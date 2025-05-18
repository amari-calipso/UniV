use std::{collections::HashMap, rc::Rc};

use analyzer::Analyzer;

use crate::{unil::{ast::{Expression, LiteralKind}, tokens::{Token, TokenType}}, univm::{bytecode::{Bytecode, Instruction}, environment::Environment, object::UniLValue}, utils::lang::AstPos};

pub mod analyzer;
pub mod environment;
pub mod type_system;

struct LoopInfo {
    continue_indices: Vec<usize>,
    break_indices:    Vec<usize>
}

impl LoopInfo {
    pub fn new() -> Self {
        LoopInfo { 
            continue_indices: Vec::new(), 
            break_indices: Vec::new() 
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Const {
    literal: Rc<str>,
    kind: LiteralKind
}

impl Const {
    pub fn new(literal: &Rc<str>, kind: LiteralKind) -> Self {
        Const { literal: Rc::clone(&literal), kind }
    }
}

pub struct Compiler {
    curr_loop: Option<LoopInfo>,
    names:  HashMap<Rc<str>, u64>,
    consts: HashMap<Const, u64>,
    last_algo_type: Option<u64>,
    next_task:  bool,
    pub output: Bytecode
}

impl Compiler {
    pub fn new() -> Self {
        Compiler { 
            curr_loop: None,
            names: HashMap::new(),
            consts: HashMap::new(),
            last_algo_type: None,
            next_task: false,
            output: Bytecode::new(), 
        }
    }

    fn name_idx(&mut self, name: &Rc<str>) -> u64 {
        if let Some(&idx) = self.names.get(name) {
            idx
        } else {
            let idx = self.output.names.len() as u64;
            self.output.names.push(Rc::clone(name));
            self.names.insert(Rc::clone(name), idx);
            idx
        }
    }

    fn const_idx(&mut self, literal: Const, value: UniLValue) -> u64 {
        if let Some(&idx) = self.consts.get(&literal) {
            idx
        } else {
            let idx = self.output.constants.len() as u64;
            self.output.constants.push(value);
            self.consts.insert(literal.clone(), idx);
            idx
        }
    }

    fn push_instruction_tok(&mut self, instruction: Instruction, token: &Token) {
        self.output.instructions.push(instruction);
        self.output.positions.push(AstPos::new(
            Rc::clone(&token.source), 
            Rc::clone(&token.filename), 
            token.pos, 
            token.end, token.line
        ));
    }

    fn push_instruction(&mut self, instruction: Instruction, expr: &Expression) {
        self.output.instructions.push(instruction);
        self.output.positions.push(expr.get_pos());
    }

    async fn compile_many(&mut self, expressions: &[Expression], tok: &Token, ctx: &mut reblessive::Stk) {
        if expressions.is_empty() {
            self.push_instruction_tok(Instruction::Null, tok);
            return;
        }

        let previous_next_task = self.next_task;

        for (i, expression) in expressions.iter().enumerate() {
            self.next_task = previous_next_task;
            ctx.run(|ctx| self.compile_one(&expression, ctx)).await;

            if i != expressions.len() - 1 {
                self.push_instruction(Instruction::Pop, expression);
            }

            if self.next_task {
                self.push_instruction(Instruction::NextTask, expression);
            }
        }
    }

    async fn compile_one(&mut self, expr: &Expression, ctx: &mut reblessive::Stk) {
        match expr {
            Expression::Block { opening_brace, expressions } => ctx.run(|ctx| self.compile_many(expressions, opening_brace, ctx)).await,
            Expression::Grouping { inner } => ctx.run(|ctx| self.compile_one(inner, ctx)).await,
            Expression::Variable { name } => {
                let idx = self.name_idx(&name.lexeme);
                self.push_instruction(Instruction::LoadName(idx), expr);
            }
            Expression::Drop { variable, .. } => {
                let idx = self.name_idx(&variable.lexeme);
                self.push_instruction(Instruction::DropName(idx), expr);
            }
            Expression::List { items, .. } => {
                for item in items {
                    ctx.run(|ctx| self.compile_one(item, ctx)).await;
                }

                self.push_instruction(Instruction::List(items.len() as u64), expr);
            }
            Expression::Binary { left, op, right } |
            Expression::Cmp { left, op, right } => {
                ctx.run(|ctx| self.compile_one(&left, ctx)).await;
                ctx.run(|ctx| self.compile_one(&right, ctx)).await;

                self.push_instruction_tok({
                    match op.type_ {
                        TokenType::Minus        => Instruction::Sub,
                        TokenType::Plus         => Instruction::Add,
                        TokenType::Slash        => Instruction::Div,
                        TokenType::Star         => Instruction::Mul,
                        TokenType::ShiftLeft    => Instruction::Shl,
                        TokenType::ShiftRight   => Instruction::Shr,
                        TokenType::Mod          => Instruction::Mod,
                        TokenType::BangEqual    => Instruction::Ne,
                        TokenType::EqualEqual   => Instruction::Eq,
                        TokenType::Greater      => Instruction::Gt,
                        TokenType::GreaterEqual => Instruction::Ge,
                        TokenType::Less         => Instruction::Lt,
                        TokenType::LessEqual    => Instruction::Le,
                        TokenType::BitwiseAnd   => Instruction::BitAnd,
                        TokenType::BitwiseOr    => Instruction::BitOr,
                        TokenType::BitwiseXor   => Instruction::Xor,
                        _ => unreachable!()
                    }
                }, op);
            }
            Expression::Logic { left, op, right } => {
                ctx.run(|ctx| self.compile_one(&left, ctx)).await;
                self.push_instruction_tok(Instruction::Clone, op);
                let jmp_left_idx = self.output.instructions.len();
                self.push_instruction_tok(Instruction::Null, op); // placeholder
                self.push_instruction_tok(Instruction::Pop, op);
                ctx.run(|ctx| self.compile_one(&right, ctx)).await;

                if op.type_ == TokenType::LogicAnd {
                    self.output.instructions[jmp_left_idx] = Instruction::IfNotJmp(self.output.instructions.len() as u64);
                } else if op.type_ == TokenType::LogicOr {
                    self.output.instructions[jmp_left_idx] = Instruction::IfJmp(self.output.instructions.len() as u64);
                } else {
                    unreachable!()
                }
            }
            Expression::AnonObject { fields, .. } => {
                self.push_instruction(Instruction::Object, expr);

                for field in fields {
                    self.push_instruction(Instruction::Clone, &field.expr); // clone object reference
                    ctx.run(|ctx| self.compile_one(&field.expr, ctx)).await; // push value of expression onto stack
                    let name_idx = self.name_idx(&field.name.lexeme);
                    self.push_instruction(Instruction::SetField(name_idx), &field.expr); // set field
                    self.push_instruction(Instruction::Pop, &field.expr); // pop the value returned from x.y = value
                }
            }
            Expression::Subscript { subscripted, index, .. } => {
                self.next_task = true;
                ctx.run(|ctx| self.compile_one(&subscripted, ctx)).await;
                ctx.run(|ctx| self.compile_one(&index, ctx)).await;
                self.push_instruction(Instruction::GetIndex, expr);
            }
            Expression::Get { object, name } => {
                ctx.run(|ctx| self.compile_one(&object, ctx)).await;
                let name_idx = self.name_idx(&name.lexeme);
                self.push_instruction(Instruction::GetField(name_idx), expr);
            } 
            Expression::ScopedBlock { dollar, expressions } => {
                self.push_instruction(Instruction::BeginScope, expr);
                ctx.run(|ctx| self.compile_many(expressions, dollar, ctx)).await;
                self.push_instruction(Instruction::EndScope, expr);
            }
            Expression::Call { callee, args, .. } => {
                self.next_task = true;

                for arg in args {
                    ctx.run(|ctx| self.compile_one(&arg, ctx)).await;
                }

                ctx.run(|ctx| self.compile_one(&callee, ctx)).await;
                self.push_instruction(Instruction::Call(args.len() as u8), expr);
            }
            Expression::Return { value, .. } => {
                if let Some(value) = value {
                    ctx.run(|ctx| self.compile_one(&value, ctx)).await;
                } else {
                    self.push_instruction(Instruction::Null, expr);
                }

                self.push_instruction(Instruction::Return, expr);
            }
            Expression::Throw { value, .. } => {
                if let Some(value) = value {
                    ctx.run(|ctx| self.compile_one(&value, ctx)).await;
                } else {
                    self.push_instruction(Instruction::Null, expr);
                }

                self.push_instruction(Instruction::Throw, expr);
            }
            Expression::Literal { value, kind, .. } => {
                match kind {
                    LiteralKind::Null => self.push_instruction(Instruction::Null, expr),
                    LiteralKind::Int => {
                        if value.as_ref() == "0" {
                            self.push_instruction(Instruction::Zero, expr);
                        } else if value.as_ref() == "1" {
                            self.push_instruction(Instruction::One, expr);
                        } else {
                            let parsed = UniLValue::Int(value.parse().expect("Analyzer skipped literal check"));
                            let idx = self.const_idx(Const::new(value, LiteralKind::Int), parsed);
                            self.push_instruction(Instruction::LoadConst(idx), expr);
                        }
                    }
                    LiteralKind::Float => {
                        let parsed = UniLValue::Float(value.parse().expect("Analyzer skipped literal check"));
                        let idx = self.const_idx(Const::new(value, LiteralKind::Float), parsed);
                        self.push_instruction(Instruction::LoadConst(idx), expr);
                    }
                    LiteralKind::String => {
                        let parsed = UniLValue::String(Rc::clone(value));
                        let idx = self.const_idx(Const::new(value, LiteralKind::String), parsed);
                        self.push_instruction(Instruction::LoadConst(idx), expr);
                    }
                }
            }
            Expression::If { condition, then_branch, else_branch, .. } => {
                ctx.run(|ctx| self.compile_one(&condition, ctx)).await;
                let condition_jmp_idx = self.output.instructions.len();
                self.push_instruction(Instruction::Null, &condition); // placeholder
                ctx.run(|ctx| self.compile_one(&then_branch, ctx)).await;

                let else_jmp_idx = self.output.instructions.len();
                
                if let Some(else_branch) = else_branch {
                    self.push_instruction(Instruction::Null, &else_branch); // placeholder
                    self.output.instructions[condition_jmp_idx] = Instruction::IfNotJmp(self.output.instructions.len() as u64);
                    ctx.run(|ctx| self.compile_one(&else_branch, ctx)).await;
                } else {
                    self.push_instruction(Instruction::Null, &expr); // placeholder
                    self.output.instructions[condition_jmp_idx] = Instruction::IfNotJmp(self.output.instructions.len() as u64);
                    self.push_instruction(Instruction::Null, &then_branch); // returns null if there is no else branch
                }

                self.output.instructions[else_jmp_idx] = Instruction::Jmp(self.output.instructions.len() as u64);
            }
            Expression::Ternary { condition, then_expr, else_expr, .. } => {
                ctx.run(|ctx| self.compile_one(&condition, ctx)).await;
                let condition_jmp_idx = self.output.instructions.len();
                self.push_instruction(Instruction::Null, &condition); // placeholder
                ctx.run(|ctx| self.compile_one(&then_expr, ctx)).await;
                let else_jmp_idx = self.output.instructions.len();
                self.push_instruction(Instruction::Null, &then_expr); // placeholder
                self.output.instructions[condition_jmp_idx] = Instruction::IfNotJmp(self.output.instructions.len() as u64);
                ctx.run(|ctx| self.compile_one(&else_expr, ctx)).await;
                self.output.instructions[else_jmp_idx] = Instruction::Jmp(self.output.instructions.len() as u64);
            }
            Expression::While { condition, body, increment, .. } => {
                self.push_instruction(Instruction::Null, expr); // on first iteration, there is no value, so push null

                let start_of_while_idx = self.output.instructions.len() as u64;
                ctx.run(|ctx| self.compile_one(&condition, ctx)).await;
                let condition_jmp_idx = self.output.instructions.len();
                self.push_instruction(Instruction::Null, &condition); // placeholder
                self.push_instruction(Instruction::Pop, &condition); // gets rid of the value from last iteration (only when we start a new one)
                
                let previous_loop = self.curr_loop.take();
                self.curr_loop = Some(LoopInfo::new());
                ctx.run(|ctx| self.compile_one(&body, ctx)).await;

                if let Some(increment) = increment {
                    let continue_address = self.output.instructions.len() as u64;
                    for &continue_idx in &self.curr_loop.as_ref().unwrap().continue_indices {
                        self.output.instructions[continue_idx] = Instruction::Jmp(continue_address);
                    }

                    ctx.run(|ctx| self.compile_one(&increment, ctx)).await;
                    self.push_instruction(Instruction::Pop, &condition); // remove value from increment, preserve value from body instead
                } else {
                    for &continue_idx in &self.curr_loop.as_ref().unwrap().continue_indices {
                        self.output.instructions[continue_idx] = Instruction::Jmp(start_of_while_idx);
                    }
                }

                self.push_instruction(Instruction::Jmp(start_of_while_idx), expr);
                let end_of_while_idx = self.output.instructions.len() as u64;
                self.output.instructions[condition_jmp_idx] = Instruction::IfNotJmp(end_of_while_idx);

                for &break_idx in &self.curr_loop.as_ref().unwrap().break_indices {
                    self.output.instructions[break_idx] = Instruction::Jmp(end_of_while_idx);
                }

                self.curr_loop = previous_loop;
            }
            Expression::Break { value, .. } => {
                if let Some(value) = value {
                    ctx.run(|ctx| self.compile_one(&value, ctx)).await;
                } else {
                    self.push_instruction(Instruction::Null, expr);
                }

                self.curr_loop.as_mut().expect("Analyzer did not check for breaks outside of a loop")
                    .break_indices.push(self.output.instructions.len());
                self.push_instruction(Instruction::Null, expr); // placeholder
            }
            Expression::Continue { value, .. } => {
                if let Some(value) = value {
                    ctx.run(|ctx| self.compile_one(&value, ctx)).await;
                } else {
                    self.push_instruction(Instruction::Null, expr);
                }

                self.curr_loop.as_mut().expect("Analyzer did not check for continues outside of a loop")
                    .continue_indices.push(self.output.instructions.len());
                self.push_instruction(Instruction::Null, expr); // placeholder
            }
            Expression::DoWhile { condition, body, .. } => {
                let previous_loop = self.curr_loop.take();
                self.curr_loop = Some(LoopInfo::new());

                let start_of_while_idx = self.output.instructions.len() as u64;
                ctx.run(|ctx| self.compile_one(&body, ctx)).await;

                ctx.run(|ctx| self.compile_one(&condition, ctx)).await;
                self.push_instruction(Instruction::Null, &condition); // placeholder
                self.push_instruction(Instruction::Jmp(start_of_while_idx), expr);
                let end_of_while_idx = self.output.instructions.len() as u64;
                self.output.instructions[end_of_while_idx as usize - 2] = Instruction::IfNotJmp(end_of_while_idx);

                for &break_idx in &self.curr_loop.as_ref().unwrap().break_indices {
                    self.output.instructions[break_idx] = Instruction::Jmp(end_of_while_idx);
                }

                for &continue_idx in &self.curr_loop.as_ref().unwrap().continue_indices {
                    self.output.instructions[continue_idx] = Instruction::Jmp(start_of_while_idx);
                }

                self.curr_loop = previous_loop;
            }
            Expression::Foreach { variable, iterator, body, .. } => {
                ctx.run(|ctx| self.compile_one(&iterator, ctx)).await;

                // if the expression is a list, automatically convert it to an iterator
                self.push_instruction(Instruction::Clone, &iterator);
                let builtin_idx = self.name_idx(&Rc::from("__builtin_isList"));
                self.push_instruction(Instruction::LoadName(builtin_idx), &iterator);
                self.push_instruction(Instruction::Call(1), &iterator);
                let list_jump_idx = self.output.instructions.len();
                self.push_instruction(Instruction::Null, &iterator); // placeholder
                let iterator_idx = self.name_idx(&Rc::from("ListIterator"));
                self.push_instruction(Instruction::LoadName(iterator_idx), &iterator);
                self.push_instruction(Instruction::Call(1), &iterator);
                self.output.instructions[list_jump_idx] = Instruction::IfNotJmp(self.output.instructions.len() as u64);

                self.push_instruction(Instruction::Null, expr); // on first iteration, there is no value, so push null

                let start_of_for_idx = self.output.instructions.len() as u64;

                // move the value from the previous iteration back to put the iterator on top
                self.push_instruction(Instruction::Insert(1), &iterator);

                // call next method on iterator
                self.push_instruction(Instruction::Clone, &iterator);
                self.push_instruction(Instruction::Clone, &iterator);
                let next_idx = self.name_idx(&Rc::from("next"));
                self.push_instruction(Instruction::GetField(next_idx), &iterator);
                self.push_instruction(Instruction::Call(1), &iterator);
                
                // if the output of `next` is null, jump out of the loop
                self.push_instruction(Instruction::Clone, expr);
                self.push_instruction(Instruction::Null, expr);
                self.push_instruction(Instruction::Eq, expr);
                let condition_jmp_idx = self.output.instructions.len();
                self.push_instruction(Instruction::Null, expr); // placeholder

                // bind the output of `next` to the variable
                let variable_name_idx = self.name_idx(&variable.lexeme);
                self.push_instruction_tok(Instruction::DefineName(variable_name_idx), variable);

                self.push_instruction(Instruction::Pop, &iterator); // remove the value of the variable from the stack
                self.push_instruction(Instruction::Insert(1), &iterator); // move the iterator back down
                self.push_instruction(Instruction::Pop, &iterator); // gets rid of the value from last iteration (only when we start a new one)

                let previous_loop = self.curr_loop.take();
                self.curr_loop = Some(LoopInfo::new());
                ctx.run(|ctx| self.compile_one(&body, ctx)).await;
                self.push_instruction(Instruction::Jmp(start_of_for_idx), expr);
                let end_of_for_with_pop_idx = self.output.instructions.len() as u64;
                self.output.instructions[condition_jmp_idx] = Instruction::IfJmp(end_of_for_with_pop_idx);
                self.push_instruction(Instruction::Pop, &expr); // remove the null from the iterator exiting out of the loop

                let end_of_for_idx = self.output.instructions.len() as u64;
                self.push_instruction(Instruction::Pop, &expr); // remove the iterator

                for &break_idx in &self.curr_loop.as_ref().unwrap().break_indices {
                    self.output.instructions[break_idx] = Instruction::Jmp(end_of_for_idx);
                }

                for &continue_idx in &self.curr_loop.as_ref().unwrap().continue_indices {
                    self.output.instructions[continue_idx] = Instruction::Jmp(start_of_for_idx);
                }

                self.curr_loop = previous_loop;
            }
            Expression::Try { try_branch, catch_branch, catch_var, .. } => {
                let catch_handler_idx = self.output.instructions.len();
                self.push_instruction(Instruction::Null, expr); // placeholder
                ctx.run(|ctx| self.compile_one(&try_branch, ctx)).await;
                self.push_instruction(Instruction::PopCatch, expr);
                let end_of_try_idx = self.output.instructions.len();
                self.push_instruction(Instruction::Null, expr); // placeholder

                self.output.instructions[catch_handler_idx] = Instruction::Catch(self.output.instructions.len() as u64);    
                if let Some(catch) = catch_branch {
                    if let Some(catch_var) = catch_var {
                        let name_idx = self.name_idx(&catch_var.lexeme);
                        self.push_instruction_tok(Instruction::DefineName(name_idx), catch_var);
                    } 

                    self.push_instruction(Instruction::Pop, expr);
                    ctx.run(|ctx| self.compile_one(&catch, ctx)).await;
                } else {
                    self.push_instruction(Instruction::Pop, expr);
                }

                self.output.instructions[end_of_try_idx] = Instruction::Jmp(self.output.instructions.len() as u64);
            }
            Expression::Switch { kw, expr, cases } => {
                ctx.run(|ctx| self.compile_one(&expr, ctx)).await;

                let mut branches_jmp_indices = Vec::new();
                let mut default = None;
                for branch in cases {
                    if let Some(cases) = &branch.cases {
                        let mut case_jmp_indices = Vec::new();
                        for case in cases {
                            self.push_instruction(Instruction::Clone, case);
                            ctx.run(|ctx| self.compile_one(&case, ctx)).await;
                            self.push_instruction(Instruction::Eq, case);
                            case_jmp_indices.push(self.output.instructions.len());
                            self.push_instruction(Instruction::Null, case); // placeholder
                        }

                        branches_jmp_indices.push(Some(case_jmp_indices));
                    } else {
                        default = Some(branch.clone());
                        branches_jmp_indices.push(None);
                    }
                }

                let mut default_jmp_idx = None;
                if let Some(default) = default {
                    self.push_instruction(Instruction::Pop, expr); // removes switch expression
                    ctx.run(|ctx| self.compile_many(&default.code, kw, ctx)).await;
                    default_jmp_idx = Some(self.output.instructions.len());
                    self.push_instruction(Instruction::Null, expr); // placeholder
                }

                let mut branch_exit_indices = Vec::new();
                for (i, branch) in cases.iter().enumerate() {
                    if branch.cases.is_some() {
                        for &case_jmp_idx in branches_jmp_indices[i].as_ref().unwrap() {
                            self.output.instructions[case_jmp_idx] = Instruction::IfJmp(self.output.instructions.len() as u64);
                        }

                        self.push_instruction(Instruction::Pop, branch.cases.as_ref().unwrap().first().unwrap()); // removes switch expression
                        ctx.run(|ctx| self.compile_many(&branch.code, kw, ctx)).await;
                        let branch_exit_idx = self.output.instructions.len();
                        branch_exit_indices.push(branch_exit_idx);
                        self.push_instruction(Instruction::Null, branch.cases.as_ref().unwrap().first().unwrap()); // placeholder
                    }
                }

                for branch_exit_idx in branch_exit_indices {
                    self.output.instructions[branch_exit_idx] = Instruction::Jmp(self.output.instructions.len() as u64);
                }

                if let Some(default_jmp_idx) = default_jmp_idx {
                    self.output.instructions[default_jmp_idx] = Instruction::Jmp(self.output.instructions.len() as u64);
                }
            }
            Expression::Assign { target, op, value, .. } => {
                if matches!(op.type_, TokenType::Walrus | TokenType::Equal) {
                    if let Some(Expression::Subscript { subscripted, index, .. }) = target.get_subscript() {
                        ctx.run(|ctx| self.compile_one(&subscripted, ctx)).await;
                        ctx.run(|ctx| self.compile_one(&index, ctx)).await;
                    } else if let Some(Expression::Get { object, .. }) = target.get_get() {
                        ctx.run(|ctx| self.compile_one(&object, ctx)).await;
                    }

                    ctx.run(|ctx| self.compile_one(&value, ctx)).await;
                } else {
                    // if we're working with an access operation, evaluate the accessed expression first, to avoid evaluating twice
                    if let Some(Expression::Subscript { subscripted, index, .. }) = target.get_subscript() {
                        ctx.run(|ctx| self.compile_one(&subscripted, ctx)).await;
                        ctx.run(|ctx| self.compile_one(&index, ctx)).await;
                        self.push_instruction(Instruction::Clone2, &target);
                        self.push_instruction(Instruction::GetIndex, &target);
                    } else if let Some(Expression::Get { object, name }) = target.get_get() {
                        ctx.run(|ctx| self.compile_one(&object, ctx)).await;
                        self.push_instruction(Instruction::Clone, &target);
                        let name_idx = self.name_idx(&name.lexeme);
                        self.push_instruction(Instruction::GetField(name_idx), &target);
                    } else {
                        ctx.run(|ctx| self.compile_one(&target, ctx)).await;
                    }

                    ctx.run(|ctx| self.compile_one(&value, ctx)).await;

                    self.push_instruction_tok({
                        match op.type_ {
                            TokenType::PlusEquals       => Instruction::Add,
                            TokenType::MinusEquals      => Instruction::Sub,
                            TokenType::StarEquals       => Instruction::Mul,
                            TokenType::SlashEquals      => Instruction::Div,
                            TokenType::ModEquals        => Instruction::Mod,
                            TokenType::ShiftLeftEquals  => Instruction::Shl,
                            TokenType::ShiftRightEquals => Instruction::Shr,
                            TokenType::AndEquals        => Instruction::BitAnd,
                            TokenType::XorEquals        => Instruction::Xor,
                            TokenType::OrEquals         => Instruction::BitOr,
                            _ => unreachable!()
                        }
                    }, op);
                }

                if target.get_subscript().is_some() {
                    self.next_task = true;
                    self.push_instruction(Instruction::SetIndex, &target);
                } else if let Some(Expression::Get { name, .. }) = target.get_get() {
                    let name_idx = self.name_idx(&name.lexeme);
                    self.push_instruction(Instruction::SetField(name_idx), &target);
                } else if let Some(Expression::Variable { name }) = target.get_variable() {
                    let name_idx = self.name_idx(&name.lexeme);
                        if op.type_ == TokenType::Walrus {
                            self.push_instruction(Instruction::DefineName(name_idx), &target);
                        } else {
                            self.push_instruction(Instruction::StoreName(name_idx), &target);
                        }
                } else {
                    unreachable!()
                }
            }
            Expression::Unary { op, expr, is_prefix } => {
                if matches!(op.type_, TokenType::PlusPlus | TokenType::MinusMinus) {
                    if *is_prefix {
                        let mut custom_op = op.clone();
                        custom_op.set_type({
                            match op.type_ {
                                TokenType::PlusPlus   => TokenType::Plus,
                                TokenType::MinusMinus => TokenType::Minus,
                                _ => unreachable!()
                            }
                        });

                        let inc_dec = Expression::Assign { 
                            target: expr.clone(), 
                            op: custom_op, value: Box::new(Expression::Literal { 
                                value: Rc::from("1"), 
                                tok: Token::empty(), 
                                kind: LiteralKind::Int
                            }), 
                            type_spec: None 
                        };

                        ctx.run(|ctx| self.compile_one(&inc_dec, ctx)).await;
                        return;
                    } 

                    if let Some(Expression::Subscript { subscripted, index, .. }) = expr.get_subscript() {
                        ctx.run(|ctx| self.compile_one(&subscripted, ctx)).await;
                        ctx.run(|ctx| self.compile_one(&index, ctx)).await;
                        // [subscripted index]
                        self.push_instruction(Instruction::Clone2, expr);
                        // [subscripted index subscripted index]
                        self.push_instruction(Instruction::GetIndex, expr);
                        // [subscripted index result]
                        self.push_instruction(Instruction::Clone, expr);
                        // [subscripted index result result]
                        self.push_instruction(Instruction::Insert(3), expr);
                        // [result subscripted index result]
                    } else if let Some(Expression::Get { object, name }) = expr.get_get() {
                        ctx.run(|ctx| self.compile_one(&object, ctx)).await;
                        // [object]
                        self.push_instruction(Instruction::Clone, expr);
                        // [object object]
                        let name_idx = self.name_idx(&name.lexeme);
                        self.push_instruction(Instruction::GetField(name_idx), expr);
                        // [object result]
                        self.push_instruction(Instruction::Clone, expr);
                        // [object result result]
                        self.push_instruction(Instruction::Insert(2), expr);
                        // [result object result]
                    } else {
                        ctx.run(|ctx| self.compile_one(&expr, ctx)).await;
                        self.push_instruction(Instruction::Clone, expr);
                    }

                    self.push_instruction_tok(Instruction::One, op);
                    self.push_instruction_tok({
                        match op.type_ {
                            TokenType::PlusPlus   => Instruction::Add,
                            TokenType::MinusMinus => Instruction::Sub,
                            _ => unreachable!()
                        }
                    }, op);

                    if expr.get_subscript().is_some() {
                        self.next_task = true;
                        self.push_instruction(Instruction::SetIndex, &expr);
                    } else if let Some(Expression::Get { name, .. }) = expr.get_get() {
                        let name_idx = self.name_idx(&name.lexeme);
                        self.push_instruction(Instruction::SetField(name_idx), &expr);
                    } else if let Some(Expression::Variable { name }) = expr.get_variable() {
                        let name_idx = self.name_idx(&name.lexeme);
                        self.push_instruction(Instruction::StoreName(name_idx), &expr);
                    } else {
                        unreachable!()
                    }

                    self.push_instruction_tok(Instruction::Pop, op); 
                    return;
                }

                ctx.run(|ctx| self.compile_one(&expr, ctx)).await;
                self.push_instruction_tok({
                    match op.type_ {
                        TokenType::Bang  => Instruction::Not,
                        TokenType::Minus => Instruction::Neg,
                        TokenType::Tilde => Instruction::Inv,
                        _ => unreachable!()
                    }
                }, op);
            }
            Expression::Function { name, params, body, .. } => {
                let function_decl_idx = self.output.instructions.len();
                self.push_instruction(Instruction::Null, expr); // placeholder
                let skip_function_jmp_idx = self.output.instructions.len();
                self.push_instruction(Instruction::Null, expr); // placeholder
                let function_idx = self.output.instructions.len();
                self.push_instruction(Instruction::Null, expr); // placeholder

                ctx.run(|ctx| self.compile_many(&body, name, ctx)).await;

                if !matches!(self.output.instructions.last().unwrap(), Instruction::Return) {
                    self.push_instruction(Instruction::Return, expr);
                }

                let name_idx = self.name_idx(&name.lexeme);
                self.output.instructions[function_decl_idx] = Instruction::FunctionDecl { 
                    address: function_idx as u64, name_idx, 
                    parameters: params.iter().map(|param| self.name_idx(&param.name.lexeme)).collect(), 
                    algorithm_type: self.last_algo_type.take()
                };

                self.output.instructions[function_idx] = Instruction::Catch(self.output.instructions.len() as u64);
                self.push_instruction(Instruction::ThrowReturn, expr);

                self.output.instructions[skip_function_jmp_idx] = Instruction::Jmp(self.output.instructions.len() as u64);
            }
            Expression::AlgoDecl { name, object, function } => {
                ctx.run(|ctx| self.compile_one(&object, ctx)).await;

                let type_idx = self.name_idx(&name.lexeme);
                self.last_algo_type = Some(type_idx);
                ctx.run(|ctx| self.compile_one(&function, ctx)).await;
            }
        }
    }

    pub fn compile(&mut self, expressions: &Vec<Expression>) {
        let mut stack = reblessive::Stack::new();

        for (i, expression) in expressions.iter().enumerate() {
            self.next_task = false;
            stack.enter(|ctx| self.compile_one(&expression, ctx)).finish();

            if i != expressions.len() - 1 {
                self.push_instruction(Instruction::Pop, expression);
            }

            if self.next_task {
                self.push_instruction(Instruction::NextTask, expression);
            }
        }

        self.push_instruction_tok(Instruction::End, &Token::empty());
    }
}

pub fn compile(expressions: &Vec<Expression>, globals: &Environment) -> Result<Bytecode, Vec<String>> {
    let mut analyzer = Analyzer::new(globals);
    analyzer.analyze(expressions);

    if !analyzer.errors.is_empty() {
        return Err(analyzer.errors);
    }

    let mut compiler = Compiler::new();
    compiler.compile(expressions);
    Ok(compiler.output)
}