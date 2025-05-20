use std::{cell::{OnceCell, RefCell}, collections::{HashMap, HashSet}, rc::Rc};

use bytecode::{Bytecode, Instruction};
use environment::Environment;
use object::{AnonObject, AnyCallable, AnyObject, Callable, ExecutionInterrupt, Function, List, NativeCallable, UniLValue};
use raylib::ffi::TraceLogLevel;
use serde::Serializer;

use crate::{algos::{Distribution, PivotSelection, Rotation, Shuffle, Sort}, api_layers, get_expect, highlights::HighlightInfo, log, utils::{lang::{traceback_part, AstPos}, object::{expect_int_range_strict, expect_int_strict}}, with_timer, IdentityHashMap, IdentityHashSet, UniV};

pub mod object;
pub mod environment;
pub mod bytecode; 

macro_rules! op_arithmetic {
    ($name: ident, $op: tt) => {
        pub fn $name(&mut self, left: &UniLValue, right: &UniLValue) {
            match left {
                UniLValue::Int(x) => {
                    match right {
                        UniLValue::Float(y) => return self.stack.push(UniLValue::Float((*x as f64) $op (*y))),
                        UniLValue::Int(y)   => return self.stack.push(UniLValue::Int(*x $op *y)),
                        UniLValue::Value { value: y, idx } => return self.stack.push(UniLValue::Value { value: *x $op *y, idx: *idx }),
                        UniLValue::Null | UniLValue::String(_) | UniLValue::Object(_) => ()
                    }
                }
                UniLValue::Value { value: x, idx } => {
                    match right {
                        UniLValue::Float(y) => return self.stack.push(UniLValue::Float((*x as f64) $op (*y))),
                        UniLValue::Int(y)   => return self.stack.push(UniLValue::Int(*x $op *y)),
                        UniLValue::Value { value: y, .. } => return self.stack.push(UniLValue::Value { value: *x $op *y, idx: *idx }),
                        UniLValue::Null | UniLValue::String(_) | UniLValue::Object(_) => ()
                    }
                }
                UniLValue::Float(x) => {
                    match right {
                        UniLValue::Int(y) | UniLValue::Value { value: y, .. } => {
                            return self.stack.push(UniLValue::Float((*x) $op (*y as f64)));
                        }
                        UniLValue::Float(y) => return self.stack.push(UniLValue::Float((*x) $op (*y))),
                        UniLValue::Null | UniLValue::String(_) | UniLValue::Object(_) => ()
                    }
                }
                UniLValue::Null | UniLValue::String(_) | UniLValue::Object(_) => ()
            }

            self.exception = Some(UniLValue::String(format!(
                "'{}' operator cannot be used on types {} and {}",
                stringify!($op), left.stringify_type(), right.stringify_type()
            ).into()));
        }
    };
}

macro_rules! op_shift {
    ($name: ident, $op: tt) => {
        pub fn $name(&mut self, left: &UniLValue, right: &UniLValue) {
            match left {
                UniLValue::Int(x) => {
                    if let UniLValue::Int(y) | UniLValue::Value { value: y, .. }  = right {
                        return self.stack.push(UniLValue::Int((*x) $op (*y)));
                    }
                }
                UniLValue::Value { value: x, idx } => {
                    if let UniLValue::Int(y) | UniLValue::Value { value: y, .. }  = right {
                        return self.stack.push(UniLValue::Value { value: (*x) $op (*y), idx: *idx });
                    }
                }
                _ => ()
            }

            self.exception = Some(UniLValue::String(format!(
                "'{}' operator cannot be used on types {} and {}",
                stringify!($op), left.stringify_type(), right.stringify_type()
            ).into()));
        }
    };
}

macro_rules! op_cmp {
    ($name: ident, $op: tt) => {
        pub fn $name(&mut self, left: &UniLValue, right: &UniLValue) {
            match left {
                UniLValue::Int(x) | UniLValue::Value { value: x, .. } => {
                    match right {
                        UniLValue::Float(y) => return self.stack.push(UniLValue::Int(((*x as f64) $op (*y)) as i64)),
                        UniLValue::Int(y) | UniLValue::Value { value: y, .. } => {
                            return self.stack.push(UniLValue::Int(((*x) $op (*y)) as i64));
                        }
                        UniLValue::Null | UniLValue::Object(_) | UniLValue::String(_) => ()
                    }
                }
                UniLValue::Float(x) => {
                    match right {
                        UniLValue::Float(y) => return self.stack.push(UniLValue::Int(((*x) $op (*y)) as i64)),
                        UniLValue::Int(y) | UniLValue::Value { value: y, .. } => {
                            return self.stack.push(UniLValue::Int(((*x) $op (*y as f64)) as i64));
                        }
                        UniLValue::Null | UniLValue::Object(_) | UniLValue::String(_) => ()
                    }
                }
                UniLValue::String(x) => {
                    match right {
                        UniLValue::String(y) => return self.stack.push(UniLValue::Int((x $op y) as i64)),
                        UniLValue::Null | UniLValue::Int(_) | UniLValue::Float(_) | UniLValue::Object(_) |
                        UniLValue::Value { .. } => ()
                    }
                }
                UniLValue::Null | UniLValue::Object(_) => ()
            }

            self.exception = Some(UniLValue::String(format!(
                "'{}' operator cannot be used on types {} and {}",
                stringify!($op), left.stringify_type(), right.stringify_type()
            ).into()));
        }
    };
}

macro_rules! op_bitwise {
    ($name: ident, $op_int: tt, $op_other: tt) => {
        pub fn $name(&mut self, left: &UniLValue, right: &UniLValue) {
            match left {
                UniLValue::Int(x) => {
                    if let UniLValue::Int(y) | UniLValue::Value { value: y, .. }  = right {
                        return self.stack.push(UniLValue::Int((*x) $op_int (*y)));
                    }
                }
                UniLValue::Value { value: x, idx } => {
                    if let UniLValue::Int(y) | UniLValue::Value { value: y, .. }  = right {
                        return self.stack.push(UniLValue::Value { value: (*x) $op_int (*y), idx: *idx });
                    }
                }
                _ => ()
            }

            self.stack.push(UniLValue::Int((left.is_truthy() $op_other right.is_truthy()) as i64))
        }
    };
}

pub struct ExceptionHandler {
    pub address: u64,
    pub environment: Rc<RefCell<Environment>>,
    pub call_stack_len: usize,
}

impl std::fmt::Debug for ExceptionHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.serialize_u64(self.address)
    }
}

pub struct Task {
    pub ip: u64,
    pub environment: Rc<RefCell<Environment>>,
    pub previous_environments: Vec<Rc<RefCell<Environment>>>,

    pub stack: Vec<UniLValue>,
    pub call_stack: Vec<u64>,
    pub exception_handlers_stack: Vec<ExceptionHandler>,
    pub exception: Option<UniLValue>,
    pub pos_stack: Vec<AstPos>,

    pub running: bool,
    pub started: bool
}

impl Task {
    pub fn new(address: u64, environment: &Rc<RefCell<Environment>>) -> Self {
        Task {
            ip: address,
            environment: Rc::clone(environment),
            previous_environments: Vec::new(),
            stack: Vec::new(),
            call_stack: Vec::new(),
            exception_handlers_stack: Vec::new(),
            exception: None,
            pos_stack: Vec::new(),
            running: true,
            started: false,
        }
    }

    pub fn empty() -> Self {
        Task {
            ip: 0,
            environment: Rc::new(RefCell::new(Environment::new())),
            previous_environments: Vec::new(),
            stack: Vec::new(),
            call_stack: Vec::new(),
            exception_handlers_stack: Vec::new(),
            exception: None,
            pos_stack: Vec::new(),
            running: false,
            started: false,
        }
    }

    pub fn pop_stack(&mut self) -> UniLValue {
        self.stack.pop().expect("VM tried to pop empty value stack")
    }

    pub fn peek_stack(&mut self) -> &UniLValue {
        self.stack.last().expect("VM tried to peek empty value stack")
    }

    pub fn pop_environment(&mut self) {        
        let enclosing = Rc::clone(
            &self.environment.borrow().enclosing
                .as_ref().expect("VM tried to pop empty environment stack")
        );

        self.environment = enclosing;
    }

    pub fn pop_call_stack(&mut self) -> Option<u64> {
        self.pos_stack.pop().expect("Position stack wasn't syncronized with call stack");
        self.call_stack.pop()
    }

    pub fn pop_exception_stack(&mut self) -> Option<ExceptionHandler> {
        self.exception_handlers_stack.pop()
    }

    /// Pops all exception handlers defined within a function call
    pub fn pop_exception_stack_after_call(&mut self) {
        loop {
            if let Some(handler) = self.exception_handlers_stack.last() {
                if handler.call_stack_len > self.call_stack.len() {
                    self.exception_handlers_stack.pop().unwrap();
                    continue;
                } 
            }

            break;
        }
    }

    pub fn get_traceback(&mut self) -> String {
        let mut result = String::new();

        if !self.pos_stack.is_empty() {
            result.push_str("Traceback (most recent call last):\n");
        }

        for pos in &self.pos_stack {
            result.push_str(&traceback_part(&pos.source, &pos.filename, pos.start, pos.end.saturating_sub(pos.start), pos.line));
        }

        result
    }

    pub fn op_unary_minus(&mut self, value: &UniLValue) {
        match value {
            UniLValue::Int(x)   => self.stack.push(UniLValue::Int(-*x)),
            UniLValue::Float(x) => self.stack.push(UniLValue::Float(-*x)),
            UniLValue::Value { value, idx } => self.stack.push(UniLValue::Value { value: -*value, idx: *idx }),
            UniLValue::Null | UniLValue::String(_) | UniLValue::Object(_) => {
                self.exception = Some(UniLValue::String(format!(
                    "'-' operator cannot be used on type {}",
                    value.stringify_type()
                ).into()));
            }
        }
    }

    pub fn op_unary_tilde(&mut self, value: &UniLValue) {
        match value {
            UniLValue::Int(x) => self.stack.push(UniLValue::Int(!*x)),
            UniLValue::Value { value, idx } => self.stack.push(UniLValue::Value { value: !*value, idx: *idx }),
            UniLValue::Null | UniLValue::String(_) | UniLValue::Float(_) | UniLValue::Object(_) => {
                self.exception = Some(UniLValue::String(format!(
                    "'~' operator cannot be used on type {}",
                    value.stringify_type()
                ).into()))
            }
        }
    }

    pub fn op_mul(&mut self, left: &UniLValue, right: &UniLValue) {
        match left {
            UniLValue::Int(x) => {
                match right {
                    UniLValue::Int(y)        => return self.stack.push(UniLValue::Int((*x) * (*y))),
                    UniLValue::Float(y)      => return self.stack.push(UniLValue::Float((*x as f64) * (*y))),
                    UniLValue::Value { value: y, idx } => return self.stack.push(UniLValue::Value { value: (*x) * (*y), idx: *idx }),
                    UniLValue::String(y) => return self.stack.push(UniLValue::String(y.repeat(*x as usize).into())),
                    UniLValue::Null | UniLValue::Object(_) => ()
                }
            }
            UniLValue::Value { value: x, idx } => {
                match right {
                    UniLValue::Int(y)        => return self.stack.push(UniLValue::Int((*x) * (*y))),
                    UniLValue::Float(y)      => return self.stack.push(UniLValue::Float((*x as f64) * (*y))),
                    UniLValue::Value { value: y, .. } => return self.stack.push(UniLValue::Value { value: (*x) * (*y), idx: *idx }),
                    UniLValue::String(y) => return self.stack.push(UniLValue::String(y.repeat(*x as usize).into())),
                    UniLValue::Null | UniLValue::Object(_) => ()
                }
            }
            UniLValue::Float(x) => {
                match right {
                    UniLValue::Int(y) | UniLValue::Value { value: y, .. } => {
                        return self.stack.push(UniLValue::Float((*x) * (*y as f64)));
                    }
                    UniLValue::Float(y) => return self.stack.push(UniLValue::Float((*x) * (*y))),
                    UniLValue::Null | UniLValue::String(_) | UniLValue::Object(_) => ()
                }
            }
            UniLValue::String(x) => {
                if let UniLValue::Int(times) = right {
                    if *times < 0 {
                        self.exception = Some(UniLValue::String(Rc::from("Cannot repeat string a negative amount of times")));
                        return;
                    }

                    return self.stack.push(UniLValue::String(x.repeat(*times as usize).into()));
                }
            }
            UniLValue::Null | UniLValue::Object(_) => ()
        }

        self.exception = Some(UniLValue::String(format!(
            "'*' operator cannot be used on types {} and {}",
            left.stringify_type(), right.stringify_type()
        ).into()))
    }

    pub fn op_add(&mut self, left: &UniLValue, right: &UniLValue) {
        match left {
            UniLValue::Int(x) => {
                match right {
                    UniLValue::Int(y)        => return self.stack.push(UniLValue::Int((*x) + (*y))),
                    UniLValue::Float(y)      => return self.stack.push(UniLValue::Float((*x as f64) + (*y))),
                    UniLValue::Value { value: y, idx } => return self.stack.push(UniLValue::Value { value: (*x) + (*y), idx: *idx }),
                    UniLValue::String(y) => return self.stack.push(UniLValue::String(format!("{x}{y}").into())),
                    UniLValue::Null | UniLValue::Object(_) => ()
                }
            }
            UniLValue::Value { value: x, idx } => {
                match right {
                    UniLValue::Int(y)        => return self.stack.push(UniLValue::Int((*x) + (*y))),
                    UniLValue::Float(y)      => return self.stack.push(UniLValue::Float((*x as f64) + (*y))),
                    UniLValue::Value { value: y, .. } => return self.stack.push(UniLValue::Value { value: (*x) + (*y), idx: *idx }),
                    UniLValue::String(y) => return self.stack.push(UniLValue::String(format!("{x}{y}").into())),
                    UniLValue::Null | UniLValue::Object(_) => ()
                }
            }
            UniLValue::Float(x) => {
                match right {
                    UniLValue::Int(y) | UniLValue::Value { value: y, .. } => {
                        return self.stack.push(UniLValue::Float((*x) + (*y as f64)));
                    }
                    UniLValue::Float(y) => return self.stack.push(UniLValue::Float((*x) + (*y))),
                    UniLValue::Null | UniLValue::String(_) | UniLValue::Object(_) => ()
                }
            }
            UniLValue::String(x) => {
                match right {
                    UniLValue::String(y) => return self.stack.push(UniLValue::String(format!("{x}{y}").into())),
                    UniLValue::Null | UniLValue::Int(_) | UniLValue::Float(_) | UniLValue::Object(_) |
                    UniLValue::Value { .. } => ()
                }
            }
            UniLValue::Null | UniLValue::Object(_) => ()
        }

        self.exception = Some(UniLValue::String(format!(
            "'+' operator cannot be used on types {} and {}",
            left.stringify_type(), right.stringify_type()
        ).into()))
    }

    pub fn op_div(&mut self, left: &UniLValue, right: &UniLValue) {
        match right {
            UniLValue::Int(y) | UniLValue::Value { value: y, .. } => {
                if *y == 0 {
                    self.exception = Some(UniLValue::String(Rc::from("Cannot divide by zero")));
                    return;
                }
            }
            UniLValue::Float(y) => {
                if *y == 0.0 {
                    self.exception = Some(UniLValue::String(Rc::from("Cannot divide by zero")));
                    return;
                }
            }
            _ => ()
        }

        match left {
            UniLValue::Int(x) => {
                match right {
                    UniLValue::Float(y) => return self.stack.push(UniLValue::Float((*x as f64) / (*y))),
                    UniLValue::Int(y)   => return self.stack.push(UniLValue::Int(*x / *y)),
                    UniLValue::Value { value: y, idx } => return self.stack.push(UniLValue::Value { value: *x / *y, idx: *idx }),
                    UniLValue::Null | UniLValue::String(_) | UniLValue::Object(_) => ()
                }
            }
            UniLValue::Value { value: x, idx } => {
                match right {
                    UniLValue::Float(y) => return self.stack.push(UniLValue::Float((*x as f64) / (*y))),
                    UniLValue::Int(y)   => return self.stack.push(UniLValue::Int(*x / *y)),
                    UniLValue::Value { value: y, .. } => return self.stack.push(UniLValue::Value { value: *x / *y, idx: *idx }),
                    UniLValue::Null | UniLValue::String(_) | UniLValue::Object(_) => ()
                }
            }
            UniLValue::Float(x) => {
                match right {
                    UniLValue::Int(y) | UniLValue::Value { value: y, .. } => {
                        return self.stack.push(UniLValue::Float((*x) / (*y as f64)));
                    }
                    UniLValue::Float(y) => return self.stack.push(UniLValue::Float((*x) / (*y))),
                    UniLValue::Null | UniLValue::String(_) | UniLValue::Object(_) => ()
                }
            }
            UniLValue::Null | UniLValue::String(_) | UniLValue::Object(_) => ()
        }

        self.exception = Some(UniLValue::String(format!(
            "'/' operator cannot be used on types {} and {}",
            left.stringify_type(), right.stringify_type()
        ).into()))
    }

    op_arithmetic!(op_mod, %);
    op_arithmetic!(op_sub, -);

    op_bitwise!(op_and, &, &&);
    op_bitwise!(op_xor, ^, ^);
    op_bitwise!(op_or, |, ||);

    op_shift!(op_shl, <<);
    op_shift!(op_shr, >>);

    op_cmp!(op_lt, <);
    op_cmp!(op_le, <=);
    op_cmp!(op_gt, >);
    op_cmp!(op_ge, >=);
}

thread_local! {
    static VM_TASKS: RefCell<IdentityHashMap<usize, Task>> = RefCell::new(HashMap::default());
}

pub struct UniVM {
    pub globals:  Rc<RefCell<Environment>>,
    pub bytecode: OnceCell<Bytecode>,

    pub delegate_call: bool,
    
    pub return_values: IdentityHashMap<usize, UniLValue>,
    scheduled_tasks: IdentityHashMap<usize, Task>,
    started_tasks: IdentityHashSet<usize>,
    task_id: usize
}

macro_rules! load_name {
    ($slf: expr, $idx: expr) => {
        get_expect!($slf.bytecode).names.get($idx as usize)
            .expect("VM tried to access out of bounds names index")
    };
}

impl UniVM {
    const TIME_SLICE_INSTRUCTIONS: usize = 1024;

    pub fn new() -> Self {
        log!(TraceLogLevel::LOG_INFO, "Loading API layers");
        let mut globals = Environment::new();
        api_layers::define(&mut globals);
        log!(TraceLogLevel::LOG_INFO, "{} globals loaded", globals.len());

        Self {
            globals: Rc::new(RefCell::new(globals)),
            bytecode: OnceCell::new(),
            delegate_call: false,
            return_values: HashMap::default(),
            scheduled_tasks: HashMap::default(),
            started_tasks: HashSet::default(),
            task_id: 0
        }
    }

    pub fn reset(&mut self) {
        VM_TASKS.with_borrow_mut(|tasks| tasks.clear());
        self.return_values.clear();
        self.task_id = 0;
    }

    pub fn schedule(&mut self, task: Task) -> usize {
        let id = self.task_id;
        self.task_id += 1;
        self.scheduled_tasks.insert(id, task);
        id
    }

    pub fn start_task(&mut self, id: usize) {
        self.started_tasks.insert(id);
    }

    pub fn set_bytecode(&mut self, bytecode: Bytecode) {
        self.bytecode.take();
        self.bytecode.set(bytecode).unwrap();
    }

    pub fn create_exception(&mut self, value: UniLValue) -> ExecutionInterrupt {
        ExecutionInterrupt::Exception { value, traceback: String::from(""), thread: 0 }
    }

    pub fn get_global(&mut self, name: &str) -> Result<UniLValue, ExecutionInterrupt> {
        let value = self.globals.borrow().get(name);
        value.ok_or_else(|| self.create_exception(UniLValue::String(
            format!("'{}' does not exist in globals", name).into()
        )))
    }
}

impl UniV {
    pub fn execute(&mut self) -> Result<UniLValue, ExecutionInterrupt> {
        VM_TASKS.with_borrow_mut(|tasks| {
            if tasks.is_empty() && self.vm.scheduled_tasks.is_empty() {
                return Ok(UniLValue::Null);
            }

            while !(tasks.is_empty() && self.vm.scheduled_tasks.is_empty()) {
                for (task_id, task) in tasks.iter_mut() {
                    if !task.started && self.vm.started_tasks.contains(task_id) {
                        self.vm.started_tasks.remove(task_id);
                        task.started = true;
                    }

                    if !task.started {
                        continue;
                    }

                    for _ in 0 .. UniVM::TIME_SLICE_INSTRUCTIONS {
                        let instruction = get_expect!(self.vm.bytecode).instructions.get(task.ip as usize)
                            .expect("VM instruction pointer tried to access out of bounds instructions").clone();
    
                        let position = get_expect!(self.vm.bytecode).positions.get(task.ip as usize)
                            .expect("VM instruction pointer tried to access out of bounds position").clone();

                        task.pos_stack.push(position);
            
                        match instruction {
                            Instruction::Null   => task.stack.push(UniLValue::Null),
                            Instruction::One    => task.stack.push(UniLValue::Int(1)),
                            Instruction::Zero   => task.stack.push(UniLValue::Int(0)),
                            Instruction::Object => task.stack.push(UniLValue::Object(Rc::new(RefCell::new(AnonObject::new().into())))),
                            Instruction::Throw  => task.exception = Some(task.pop_stack()),
                            Instruction::EndScope => task.pop_environment(),
                            Instruction::BeginScope => {
                                let previous = Rc::clone(&task.environment);
                                task.environment = Rc::new(RefCell::new(Environment::with_enclosing(previous)));
                            }
                            Instruction::End => {
                                task.running = false; 
                                break;
                            }
                            Instruction::Pop => {
                                task.pop_stack();
                            }
                            Instruction::Clone => {
                                let tmp = task.peek_stack().clone();
                                task.stack.push(tmp);
                            }
                            Instruction::Clone2 => {
                                let tmp1 = task.pop_stack();
                                let tmp0 = task.pop_stack();
                                task.stack.push(tmp0.clone());
                                task.stack.push(tmp1.clone());
                                task.stack.push(tmp0);
                                task.stack.push(tmp1);
                            }
                            Instruction::Insert(distance) => {
                                // [a b c ...<x distance> z]
                                let tmp = task.pop_stack();
                                // [a b c ...<x distance>] [z]
                                task.stack.insert(task.stack.len() - distance as usize, tmp);
                                // [z a b c ...]
                            }
                            Instruction::NextTask => {
                                task.ip = task.ip.wrapping_add(1);
                                task.pos_stack.pop();
                                break;
                            }
                            Instruction::Jmp(address) => {
                                task.ip = address;
                                task.pos_stack.pop();
                                continue;
                            }
                            Instruction::Catch(address) => {
                                task.exception_handlers_stack.push(ExceptionHandler { 
                                    address, 
                                    environment: Rc::clone(&task.environment),
                                    call_stack_len: task.call_stack.len()
                                });
                            }
                            Instruction::PopCatch => {
                                task.pop_exception_stack();
                            }
                            Instruction::LoadConst(idx) => {
                                let value = get_expect!(self.vm.bytecode).constants.get(idx as usize)
                                    .expect("VM tried to load out of bounds constant").clone();
                                task.stack.push(value);
                            }
                            Instruction::IfNotJmp(address) => {
                                if !task.pop_stack().is_truthy() {
                                    task.ip = address;
                                    task.pos_stack.pop();
                                    continue;
                                }
                            }
                            Instruction::IfJmp(address) => {
                                if task.pop_stack().is_truthy() {
                                    task.ip = address;
                                    task.pos_stack.pop();
                                    continue;
                                }
                            }
                            Instruction::List(items_amt) => {
                                let mut items = Vec::with_capacity(items_amt as usize);
                                for _ in 0 .. items_amt {
                                    items.push(task.pop_stack());
                                }
                                items.reverse();
                                task.stack.push(UniLValue::Object(Rc::new(RefCell::new(List::from(items).into()))));
                            }
                            Instruction::Return => {
                                if let Some(address) = task.pop_call_stack() {
                                    task.environment = task.previous_environments.pop()
                                        .expect("Enviroments stack wasn't syncronized with call stack");
                                    task.pop_exception_stack_after_call();
                                    task.ip = address;
                                } else {
                                    // if you return from an empty call stack, task has finished
                                    task.running = false; 
                                    break;
                                }
                            }
                            Instruction::ThrowReturn => {
                                task.exception = Some(task.pop_stack());
                                if let Some(address) = task.pop_call_stack() {
                                    task.environment = task.previous_environments.pop()
                                        .expect("Enviroments stack wasn't syncronized with call stack");
                                    task.pop_exception_stack_after_call();
                                    task.ip = address;
                                }
                            }
                            Instruction::LoadName(name_idx) => {
                                let name = load_name!(self.vm, name_idx);
                                if let Some(value) = task.environment.borrow().get(&name) {
                                    task.stack.push(value);
                                } else {
                                    task.exception = Some(UniLValue::String(format!("Unknown variable '{}'", name).into()));
                                }
                            }
                            Instruction::DefineName(name_idx) => {
                                let name = load_name!(self.vm, name_idx);
                                let value = task.peek_stack().clone();
                                task.environment.borrow_mut().define(name, value);
                            }
                            Instruction::StoreName(name_idx) => {
                                let name = load_name!(self.vm, name_idx);
                                let value = task.peek_stack().clone();
                                if task.environment.borrow_mut().set(&name, value).is_err() {
                                    task.exception = Some(UniLValue::String(format!("Unknown variable '{}'", name).into()));
                                }
                            }
                            Instruction::DropName(name_idx) => {
                                let name = load_name!(self.vm, name_idx);
                                if task.environment.borrow_mut().del(&name).is_err() {
                                    task.exception = Some(UniLValue::String(format!("Unknown variable '{}'", name).into()));
                                } else {
                                    task.stack.push(UniLValue::Null);
                                }
                            }
                            Instruction::Neg => {
                                let value = task.pop_stack();
                                task.op_unary_minus(&value);
                            }
                            Instruction::Not => {
                                let value = task.pop_stack();
                                task.stack.push(UniLValue::Int((!value.is_truthy()) as i64))
                            }
                            Instruction::Inv => {
                                let value = task.pop_stack();
                                task.op_unary_tilde(&value);
                            }
                            Instruction::Mul | Instruction::Add | Instruction::Div | Instruction::Mod |
                            Instruction::Sub | Instruction::Xor | Instruction::Shl | Instruction::Shr |
                            Instruction::BitAnd | Instruction::BitOr | Instruction::Lt | Instruction::Le |
                            Instruction::Gt | Instruction::Ge | Instruction::Eq | Instruction::Ne => {
                                let right = task.pop_stack();
                                let left = task.pop_stack();
    
                                if matches!(left, UniLValue::Value { .. }) || matches!(right, UniLValue::Value { .. }) {
                                    self.comparisons += 1;
                                }
    
                                match instruction {
                                    Instruction::Mul => task.op_mul(&left, &right),
                                    Instruction::Add => task.op_add(&left, &right),
                                    Instruction::Div => task.op_div(&left, &right),
                                    Instruction::Mod => task.op_mod(&left, &right),
                                    Instruction::Sub => task.op_sub(&left, &right),
                                    Instruction::Shl => task.op_shl(&left, &right),
                                    Instruction::Shr => task.op_shr(&left, &right),
                                    Instruction::Xor => task.op_xor(&left, &right),
                                    Instruction::BitAnd => task.op_and(&left, &right),
                                    Instruction::BitOr  => task.op_or(&left, &right),
                                    Instruction::Lt => task.op_lt(&left, &right),
                                    Instruction::Le => task.op_le(&left, &right),
                                    Instruction::Gt => task.op_gt(&left, &right),
                                    Instruction::Ge => task.op_ge(&left, &right),
                                    Instruction::Eq => task.stack.push(UniLValue::Int( left.equals(&right) as i64)),
                                    Instruction::Ne => task.stack.push(UniLValue::Int(!left.equals(&right) as i64)),
                                    _ => unreachable!()
                                }
                            }
                            Instruction::GetIndex => {
                                let index = task.pop_stack();
                                let subscripted = task.pop_stack();
    
                                let mut error = true;
                                if let UniLValue::Object(obj) = &subscripted {
                                    if let AnyObject::List(list) = &*obj.borrow() {
                                        if let UniLValue::Int(idx) | UniLValue::Value { value: idx, .. } = index {
                                            match with_timer!(self, list.get_with_exception(idx, self)) {
                                                Ok(value) => task.stack.push(value),
                                                Err(e) => {
                                                    if let ExecutionInterrupt::Exception { value, .. } = e {
                                                        task.exception = Some(value);
                                                    } else {
                                                        unreachable!()
                                                    }
                                                }
                                            }

                                            self.reads += 1;
                                            let aux = self.get_optional_aux_id(obj.as_ptr() as *const AnyObject);
                                            self.highlights.push(HighlightInfo::from_idx_and_aux(list.convert_index(idx), aux));  
                                        } else {
                                            task.exception = Some(UniLValue::String(format!(
                                                "List index must be of type Int, not {}", 
                                                index.stringify_type()
                                            ).into()));
                                        }
    
                                        error = false;
                                    }
                                }
                
                                if error {
                                    task.exception = Some(UniLValue::String(format!("Type {} is not indexable", subscripted.stringify_type()).into()));
                                }
                            }
                            Instruction::SetIndex => {
                                let value = task.pop_stack();
                                let index = task.pop_stack();
                                let subscripted = task.pop_stack();
    
                                let mut error = true;
                                if let UniLValue::Object(obj) = &subscripted {
                                    if let AnyObject::List(list) = &mut *obj.borrow_mut() {
                                        if let UniLValue::Int(idx) | UniLValue::Value { value: idx, .. } = index {   
                                            match with_timer!(self, list.set_with_exception(idx, value, self)) {
                                                Ok(value) => task.stack.push(value),
                                                Err(e) => {
                                                    if let ExecutionInterrupt::Exception { value, .. } = e {
                                                        task.exception = Some(value);
                                                    } else {
                                                        unreachable!()
                                                    }
                                                }
                                            }

                                            self.writes += 1;
                                            let aux = self.get_optional_aux_id(obj.as_ptr() as *const AnyObject);
                                            self.highlights.push(HighlightInfo::from_idx_and_aux_write(list.convert_index(idx), aux));                     
                                        } else {
                                            task.exception = Some(UniLValue::String(format!(
                                                "List index must be of type Int, not {}", 
                                                index.stringify_type()
                                            ).into()));
                                        }
    
                                        error = false;
                                    }
                                }
                
                                if error {
                                    task.exception = Some(UniLValue::String(format!("Type {} is not indexable", subscripted.stringify_type()).into()));
                                }
                            }
                            Instruction::SetField(name_idx) => {
                                let name = load_name!(self.vm, name_idx);

                                let value = task.pop_stack();
                                let object = task.pop_stack();
    
                                let mut error = true;
                                if let UniLValue::Object(obj) = &object {
                                    if let AnyObject::AnonObject(instance) = &mut *obj.borrow_mut() {
                                        instance.set(&name, value.clone());
                                        task.stack.push(value);
                                        error = false;
                                    }
                                }
                                
                                if error {
                                    task.exception = Some(UniLValue::String(format!("Cannot write properties of type {}", object.stringify_type()).into()));
                                }
                            }
                            Instruction::GetField(name_idx) => {
                                let name = load_name!(self.vm, name_idx);
    
                                let object = task.pop_stack();
                                let cloned_object = object.clone();
                                
                                let mut error = true;
                                match object {
                                    UniLValue::Object(ref obj) => {
                                        if let AnyObject::AnonObject(instance) = &*obj.borrow() {
                                            if let Some(field) = instance.get(&name) {
                                                task.stack.push(field);
                                            } else {
                                                task.exception = Some(UniLValue::String(format!("Undefined property '{}'", name).into()));
                                            }
    
                                            error = false;
                                        }
                                    }
                                    // handles some oSV methods
                                    UniLValue::Value { value, .. } => {
                                        error = false;
                                        match name.as_ref() {
                                            "copy" | "noMark" | "read" => {
                                                task.stack.push(UniLValue::Object(Rc::new(RefCell::new(AnyObject::AnyCallable(
                                                    NativeCallable::new(
                                                        Rc::new(move |_univ, _args, _task| Ok(cloned_object.clone())), 
                                                        1
                                                    ).into()
                                                )))));
                                            }
                                            "getInt" | "readInt" => {
                                                task.stack.push(UniLValue::Object(Rc::new(RefCell::new(AnyObject::AnyCallable(
                                                    NativeCallable::new(
                                                        Rc::new(move |_univ, _args, _task| Ok(UniLValue::Int(value))), 
                                                        1
                                                    ).into()
                                                )))));
                                            }
                                            "readNoMark" => {
                                                task.stack.push(UniLValue::Object(Rc::new(RefCell::new(AnyObject::AnyCallable(
                                                    NativeCallable::new(
                                                        Rc::new(move |_univ, _args, _task| {
                                                            Ok(UniLValue::Object(Rc::new(RefCell::new(
                                                                List::from(vec![cloned_object.clone(), UniLValue::Null]).into()
                                                            ))))
                                                        }), 
                                                        1
                                                    ).into()
                                                )))));
                                            }
                                            "readDigit" => {
                                                task.stack.push(UniLValue::Object(Rc::new(RefCell::new(AnyObject::AnyCallable(
                                                    NativeCallable::new(
                                                        Rc::new(move |univ, args, _task| {
                                                            let d = expect_int_range_strict(
                                                                &args[1],
                                                                (i32::MIN as i64)..(i32::MAX as i64), 
                                                                "first argument of 'readDigit'", 
                                                                univ
                                                            )?;
                                                            let base = expect_int_strict(&args[2], "second argument of 'readDigit'", univ)?;
                
                                                            Ok(UniLValue::Int((value as f64 / (base as f64).powi(d as i32)) as i64 % base))
                                                        }), 
                                                        3
                                                    ).into()
                                                )))));
                                            }
                                            _ => {
                                                task.exception = Some(UniLValue::String(format!("Unknown property '{}'", name).into()));
                                            }
                                        }
                                    }
                                    _ => ()
                                }
                                
                                if error {
                                    task.exception = Some(UniLValue::String(format!("Cannot read properties of type '{}'", object.stringify_type()).into()));
                                }
                            }
                            Instruction::Call(args_n) => {
                                let callee = task.pop_stack();
    
                                let mut error = true;
                                if let UniLValue::Object(obj) = &callee {
                                    if let AnyObject::AnyCallable(callable) = &*obj.borrow() {
                                        if args_n != callable.arity() {
                                            task.exception = Some(UniLValue::String(format!(
                                                "Expecting {} arguments but got {}", 
                                                callable.arity(), args_n
                                            ).into()));
                                        } else {
                                            let mut args = Vec::new();
                                            for _ in 0 .. args_n {
                                                let tmp = task.pop_stack();
                                                args.push(tmp);
                                            }

                                            args.reverse(); // args are fetched backwards
                                                
                                            match callable {
                                                AnyCallable::NativeCallable(native) => {
                                                    match native.call(self, args, task) {
                                                        Ok(value) => {
                                                            if self.vm.delegate_call {
                                                                self.vm.delegate_call = false;
                                                                task.pos_stack.pop();
                                                                continue;
                                                            } else {
                                                                task.stack.push(value);
                                                            }
                                                        }
                                                        Err(e) => {
                                                            if let ExecutionInterrupt::Exception { value, .. } = e {
                                                                task.exception = Some(value);
                                                            } else {
                                                                return Err(e);
                                                            }
                                                        }
                                                    }
                                                }
                                                AnyCallable::Function(function) => {
                                                    function.call(self, args, task).unwrap();
                                                    task.pos_stack.pop();
                                                    continue;
                                                }
                                            }
                                        }
    
                                        error = false;
                                    }
                                }
                                
                                if error {
                                    task.exception = Some(UniLValue::String(format!("Cannot call type {}", callee.stringify_type()).into()));
                                }
                            }
                            Instruction::FunctionDecl { address, name_idx, parameters, algorithm_type } => {
                                let name = load_name!(self.vm, name_idx);

                                let function = Function::new(
                                    address, 
                                    parameters.iter()
                                        .map(|idx| Rc::clone(load_name!(self.vm, *idx)))
                                        .collect()
                                );

                                let callable = AnyCallable::Function(function);
                                self.vm.globals.borrow_mut().define(name, UniLValue::Object(Rc::new(RefCell::new(callable.clone().into()))));

                                if let Some(algo_idx) = algorithm_type {
                                    let algo_type = load_name!(self.vm, algo_idx);

                                    let mut object = None;
                                    if algo_type.as_ref() != "runAllShuffles" {
                                        if let UniLValue::Object(obj) = task.pop_stack() {
                                            if let AnyObject::AnonObject(instance) = &*obj.borrow() {
                                                object = Some(instance.clone());
                                            }
                                        }
                                    } 

                                    match algo_type.as_ref() {
                                        "sort" => {
                                            match Sort::new(object.unwrap(), callable.clone()) {
                                                Ok(sort) => {
                                                    let _ = self.add_sort(sort)
                                                        .map_err(|e| task.exception = Some(UniLValue::String(e)));
                                                }
                                                Err(e) => task.exception = Some(UniLValue::String(e)),
                                            }
                                        }
                                        "shuffle" => {
                                            match Shuffle::new(object.unwrap(), callable.clone()) {
                                                Ok(shuffle) => {
                                                    let _ = self.add_shuffle(shuffle)
                                                        .map_err(|e| task.exception = Some(UniLValue::String(e)));
                                                }
                                                Err(e) => task.exception = Some(UniLValue::String(e)),
                                            }
                                        }
                                        "distribution" => {
                                            match Distribution::new(object.unwrap(), callable.clone()) {
                                                Ok(distribution) => {
                                                    let _ = self.add_distribution(distribution)
                                                        .map_err(|e| task.exception = Some(UniLValue::String(e)));
                                                }
                                                Err(e) => task.exception = Some(UniLValue::String(e)),
                                            }
                                        }
                                        "pivotSelection" => {
                                            match PivotSelection::new(object.unwrap(), callable.clone()) {
                                                Ok(pivot_selection) => {
                                                    let _ = self.add_pivot_selection(pivot_selection)
                                                        .map_err(|e| task.exception = Some(UniLValue::String(e)));
                                                }
                                                Err(e) => task.exception = Some(UniLValue::String(e)),
                                            }
                                        }
                                        "rotation" | "indexedRotation" => {
                                            match Rotation::new_indexed(object.unwrap(), callable.clone()) {
                                                Ok(rotation) => {
                                                    let _ = self.add_rotation(rotation)
                                                        .map_err(|e| task.exception = Some(UniLValue::String(e)));
                                                }
                                                Err(e) => task.exception = Some(UniLValue::String(e)),
                                            }
                                        }
                                        "lengthsRotation" => {
                                            match Rotation::new_lengths(object.unwrap(), callable.clone()) {
                                                Ok(rotation) => {
                                                    let _ = self.add_rotation(rotation)
                                                        .map_err(|e| task.exception = Some(UniLValue::String(e)));
                                                }
                                                Err(e) => task.exception = Some(UniLValue::String(e)),
                                            }
                                        }
                                        _ => unreachable!()
                                    }
                                }

                                task.stack.push(UniLValue::Object(Rc::new(RefCell::new(callable.into()))))
                            }
                        }
    
                        if let Some(exception) = task.exception.take() {
                            if let Some(handler) = task.pop_exception_stack() {
                                task.environment = handler.environment;
                                task.stack.push(exception);
                                task.ip = handler.address;
                                continue;
                            } else {
                                return Err(ExecutionInterrupt::Exception { 
                                    value: exception, 
                                    traceback: task.get_traceback(),
                                    thread: *task_id
                                });
                            }
                        }
    
                        task.ip = task.ip.wrapping_add(1);
                        task.pos_stack.pop();
                    }
                }

                for (&task_id, task) in tasks.iter_mut() {
                    if !task.running {
                        if let Some(value) = task.stack.pop() {
                            self.vm.return_values.insert(task_id, value);
                        }
                    }
                }
    
                tasks.retain(|_, task| task.running);
                tasks.extend(self.vm.scheduled_tasks.drain());

                self.immediate_highlight(Vec::new())?;

                // if all tasks are not started, they will never start 
                // because no task is active to start them, so break out
                if tasks.iter().all(|(_, x)| !x.started) {
                    break;
                }
            }

            Ok(self.vm.return_values.get(&0).expect("Main task did not have ID = 0").clone())
        })
    }
}