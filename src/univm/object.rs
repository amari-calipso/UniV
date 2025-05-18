use core::str;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use bincode::{decode_from_reader, encode_into_writer, impl_borrow_decode, Decode, Encode};
use enum_dispatch::enum_dispatch;

use crate::{compiler::type_system::UniLType, univm::{environment::Environment, Task, VM_TASKS}, UniV};

#[derive(Clone, Debug)]
pub enum ExecutionInterrupt {
    Exception {
        value: UniLValue,
        traceback: String,
        thread: usize,
    },
    Quit,
    StopAlgorithm
}

#[derive(Clone, Debug)]
pub enum UniLValue {
    Null,
    Int(i64),
    Value {
        value: i64,
        idx: usize
    },
    Float(f64),
    String(Rc<str>),
    Object(Rc<RefCell<AnyObject>>)
}

impl UniLValue {
    pub fn is_truthy(&self) -> bool {
        match self {
            UniLValue::Null => false,
            UniLValue::String(_) | UniLValue::Object(_) | UniLValue::Float(_) => true,
            UniLValue::Int(x) | UniLValue::Value { value: x, .. } => *x != 0,
        }
    }

    pub fn equals(&self, other: &Self) -> bool {
        match self {
            UniLValue::Null => matches!(other, UniLValue::Null),
            UniLValue::Int(x) | UniLValue::Value { value: x, .. } => {
                match other {
                    UniLValue::Int(y) | UniLValue::Value { value: y, .. } => *x == *y,
                    UniLValue::Float(y) => (*x as f64) == *y,
                    UniLValue::Null | UniLValue::Object(_) | UniLValue::String(_) => false
                }
            }
            UniLValue::Float(x) => {
                match other {
                    UniLValue::Int(y) | UniLValue::Value { value: y, .. } => *x == (*y as f64),
                    UniLValue::Float(y) => *x == *y,
                    UniLValue::Null | UniLValue::Object(_) | UniLValue::String(_) => false
                }
            }
            UniLValue::String(x) => {
                if let UniLValue::String(y) = other {
                    x == y
                } else {
                    false
                }
            }
            UniLValue::Object(x) => {
                if let UniLValue::Object(y) = other {
                    x.as_ptr() == y.as_ptr()
                } else {
                    false
                }
            }
        }
    }

    pub fn stringify(&self) -> Rc<str> {
        match self {
            UniLValue::Null                               => Rc::from("null"),
            UniLValue::Float(x)                     => format!("{}", *x).into(),
            UniLValue::String(x)                => Rc::clone(x),
            UniLValue::Object(x) => x.borrow().stringify(),
            UniLValue::Int(x) |
            UniLValue::Value { value: x, .. } => format!("{}", *x).into(),
        }
    }

    pub fn get_type(&self) -> UniLType {
        match self {
            UniLValue::Null => UniLType::Null,
            UniLValue::Int(_) => UniLType::Int,
            UniLValue::Value { .. } => UniLType::Value,
            UniLValue::Float(_) => UniLType::Float,
            UniLValue::String(_) => UniLType::String,
            UniLValue::Object(obj) => obj.borrow().get_type(),
        }
    }

    pub fn stringify_type(&self) -> Rc<str> {
        if let UniLValue::Object(obj) = self {
            if matches!(&*obj.borrow(), AnyObject::AnyCallable(_)) {
                return Rc::from("Callable");
            }
        }

        self.get_type().stringify()
    }
}

impl Encode for UniLValue {
    fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        let mut writer = encoder.writer();
        match self {
            UniLValue::Null => {
                encode_into_writer(0u8, &mut writer, bincode::config::standard())?;
            }
            UniLValue::Int(x) => {
                encode_into_writer(1u8, &mut writer, bincode::config::standard())?;
                encode_into_writer(x, &mut writer, bincode::config::standard())?;
            }
            UniLValue::Float(x) => {
                encode_into_writer(2u8, &mut writer, bincode::config::standard())?;
                encode_into_writer(x, &mut writer, bincode::config::standard())?;
            }
            UniLValue::String(x) => {
                encode_into_writer(3u8, &mut writer, bincode::config::standard())?;
                encode_into_writer(x, &mut writer, bincode::config::standard())?;
            }
            _ => panic!("Can only encode primitive types")
        }

        Ok(())
    }
}

impl<C> Decode<C> for UniLValue {
    fn decode<D: bincode::de::Decoder<Context = C>>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
        let mut reader = decoder.reader();
        let type_: u8 = decode_from_reader(&mut reader, bincode::config::standard())?;

        match type_ {
            0 => Ok(UniLValue::Null),
            1 => Ok(UniLValue::Int(decode_from_reader(&mut reader, bincode::config::standard())?)),
            2 => Ok(UniLValue::Float(decode_from_reader(&mut reader, bincode::config::standard())?)),
            3 => Ok(UniLValue::String(decode_from_reader(&mut reader, bincode::config::standard())?)),
            _ => Err(bincode::error::DecodeError::Other("Invalid value type"))
        }
    }
}

impl_borrow_decode!(UniLValue);

#[enum_dispatch(AnyCallable)]
pub trait Callable {
    fn arity(&self) -> u8;
    fn stringify(&self) -> Rc<str>;
    fn call(&self, univ: &mut UniV, args: Vec<UniLValue>, task: &mut Task) -> Result<UniLValue, ExecutionInterrupt>;
}

#[derive(Clone)]
pub struct NativeCallable {
    call:  Rc<dyn Fn(&mut UniV, Vec<UniLValue>, &mut Task) -> Result<UniLValue, ExecutionInterrupt>>,
    arity: u8
}

impl NativeCallable {
    pub fn new(call: Rc<dyn Fn(&mut UniV, Vec<UniLValue>, &mut Task) -> Result<UniLValue, ExecutionInterrupt>>, arity: u8) -> Self {
        NativeCallable { call, arity }
    }
}

impl std::fmt::Debug for NativeCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeCallable").field("arity", &self.arity).finish()
    }
}

impl Callable for NativeCallable {
    fn arity(&self) -> u8 {
        self.arity
    }

    fn call(&self, univ: &mut UniV, args: Vec<UniLValue>, task: &mut Task) -> Result<UniLValue, ExecutionInterrupt> {
        (self.call)(univ, args, task)
    }

    fn stringify(&self) -> Rc<str> {
        Rc::from("<native fn>")
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    pub address: u64,
    pub params:  Rc<Vec<Rc<str>>>
}

impl Function {
    pub fn new(address: u64, params: Vec<Rc<str>>) -> Self {
        Function { address, params: Rc::new(params) }
    }
}

impl Callable for Function {
    fn arity(&self) -> u8 {
        self.params.len() as u8
    }

    fn stringify(&self) -> Rc<str> {
        format!("<function at {:#018x}>", self.address).into()
    }

    fn call(&self, univ: &mut UniV, mut args: Vec<UniLValue>, task: &mut Task) -> Result<UniLValue, ExecutionInterrupt> {
        if VM_TASKS.with(|x| x.try_borrow().is_ok()) { // if vm is not active
            let mut env = Environment::with_enclosing(Rc::clone(&univ.vm.globals));
            for i in 0 .. args.len() {
                env.define(&self.params[i], args[i].clone());
            }

            let mut task = Task::new(self.address, &Rc::new(RefCell::new(env)));
            task.started = true;

            univ.vm.schedule(task);
            let ret = univ.execute();
            univ.vm.reset();
            ret
        } else {
            let mut env = Environment::with_enclosing(Rc::clone(&univ.vm.globals));
            for param_name in self.params.iter().rev() {
                env.define(param_name, args.pop().unwrap());
            }

            task.previous_environments.push(Rc::clone(&task.environment));
            task.environment = Rc::new(RefCell::new(env));
            task.call_stack.push(task.ip);
            task.ip = self.address;
            Ok(UniLValue::Null)
        }
    }
}

#[enum_dispatch(AnyObject)]
pub trait Object {
    fn stringify(&self) -> Rc<str>;
    fn stringify_type(&self) -> Rc<str>;
    fn get_type(&self) -> UniLType;
}

#[enum_dispatch]
#[derive(Clone, Debug)]
pub enum AnyCallable {
    NativeCallable,
    Function,
}

impl Object for AnyCallable {
    fn stringify(&self) -> Rc<str> {
        Callable::stringify(self)
    }

    fn stringify_type(&self) -> Rc<str> {
        Rc::from("Callable")
    }

    fn get_type(&self) -> UniLType {
        let mut args = Vec::with_capacity(self.arity() as usize);
        for _ in 0 .. self.arity() {
            args.push(UniLType::Any)
        }

        UniLType::Callable { 
            args, 
            return_type: Box::new(UniLType::Any) 
        }
    }
}

#[derive(Clone, Debug)]
pub struct AnonObject {
    pub fields: HashMap<Rc<str>, UniLValue>
}

impl AnonObject {
    pub fn new() -> Self {
        AnonObject { fields: HashMap::new() }
    }

    pub fn set(&mut self, name: &Rc<str>, value: UniLValue) {
        self.fields.insert(Rc::clone(name), value);
    }
    
    pub fn get(&self, name: &str) -> Option<UniLValue> {
        self.fields.get(name).cloned()
    }
}

impl Object for AnonObject {
    fn stringify(&self) -> Rc<str> {
        let mut output = String::from("#{");
        if self.fields.len() != 0 {
            output.push(' ');
        }

        for (idx, (name, value)) in self.fields.iter().enumerate() {
            output.push_str(name);
            output.push_str(": ");
            output.push_str(&value.stringify());

            if idx != self.fields.len() - 1 {
                output.push_str(", ");
            }
        }

        output.push('}');
        output.into()
    }

    fn stringify_type(&self) -> Rc<str> {
        Rc::from("Object")
    }

    fn get_type(&self) -> UniLType {
        UniLType::Object { 
            fields: Rc::new(RefCell::new(
                self.fields.iter()
                    .map(|(k, v)| (Rc::clone(k), v.get_type()))
                    .collect() 
            ))
        }
    }
}

#[derive(Clone)]
pub struct List {
    pub items: Vec<UniLValue>
}

impl std::fmt::Debug for List {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("List").field("length", &self.items.len()).finish()
    }
}

impl List {
    pub fn new() -> Self {
        List { items: Vec::new() }
    }

    pub fn from(items: Vec<UniLValue>) -> Self {
        List { items }
    }

    pub fn convert_index_for_len(len: usize, x: i64) -> usize {
        (
            if x < 0 {
                len as i64 + x
            } else {
                x
            }
        ) as usize
    }

    pub fn convert_index(&self, x: i64) -> usize {
        Self::convert_index_for_len(self.items.len(), x)
    }

    pub fn contains(&self, value: &UniLValue) -> bool {
        for item in &self.items {
            if item.equals(value) {
                return true;
            }
        }

        false
    }

    pub fn get(&self, idx: i64) -> Option<UniLValue> {
        self.items.get(self.convert_index(idx)).cloned()
    }

    pub fn get_with_exception(&self, idx: i64, univ: &mut UniV) -> Result<UniLValue, ExecutionInterrupt> {
        self.get(idx).ok_or_else(|| univ.vm.create_exception(
            UniLValue::String(format!("List index {} out of range for length {}", idx, self.items.len()).into())
        ))
    }

    pub fn set(&mut self, idx: i64, value: UniLValue) -> Result<(), ()> {
        let real_idx = self.convert_index(idx);

        if real_idx < self.items.len() {
            // if this spot contains a value that encodes index information, we don't want to lose it
            if matches!(self.items[real_idx], UniLValue::Value { .. }) {
                match value {
                    UniLValue::Int(x) => {
                        if let UniLValue::Value { value, .. } = unsafe { self.items.get_unchecked_mut(real_idx) } {
                            *value = x;
                        } else {
                            unreachable!()
                        }
                    }
                    UniLValue::Float(x) => {
                        if let UniLValue::Value { value, .. } = unsafe { self.items.get_unchecked_mut(real_idx) } {
                            *value = x as i64;
                        } else {
                            unreachable!()
                        }
                    }
                    _ => self.items[real_idx] = value
                }
            } else {
                self.items[real_idx] = value;
            }

            Ok(())
        } else {
            Err(())
        }
    }

    pub fn set_with_exception(&mut self, idx: i64, value: UniLValue, univ: &mut UniV) -> Result<UniLValue, ExecutionInterrupt> {
        if self.set(idx, value.clone()).is_ok() {
            Ok(value)
        } else {
            Err(univ.vm.create_exception(
                UniLValue::String(format!("List index {} out of range for length {}", idx, self.items.len()).into())
            ))
        }
    }
}

impl Object for List {
    fn stringify(&self) -> Rc<str> {
        let mut output = String::from("[");

        for (idx, value) in self.items.iter().enumerate() {
            output.push_str(&value.stringify());

            if idx != self.items.len() - 1 {
                output.push_str(", ");
            }
        }

        output.push(']');
        output.into()
    }

    fn stringify_type(&self) -> Rc<str> {
        Rc::from("List")
    }

    fn get_type(&self) -> UniLType {
        UniLType::List
    }
}

#[enum_dispatch]
#[derive(Clone, Debug)]
pub enum AnyObject {
    AnyCallable,
    AnonObject,
    List
}