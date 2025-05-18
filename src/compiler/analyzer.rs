use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{api_layers, ast_error, ast_note, univm::environment::Environment, token_error, unil::{ast::{Expression, LiteralKind}, tokens::{Token, TokenType}}, utils::object::object_type};

use super::{environment::{AnalyzerEnvError, AnalyzerEnvironment}, type_system::UniLType};


#[derive(Debug)]
pub enum AnalyzerInterrupt {
    BreakOrContinue,
    Throw,
    Return(UniLType)
}

#[derive(Clone)]
struct CurrFn {
    return_type: UniLType,
    definition: Expression
}

pub struct Analyzer {
    globals:     Rc<RefCell<AnalyzerEnvironment>>,
    environment: Rc<RefCell<AnalyzerEnvironment>>,

    in_loop: bool,
    curr_fn: Option<CurrFn>,

    pub errors: Vec<String>
}

impl Analyzer {
    pub fn new(globals: &Environment) -> Self {
        let globals = Rc::new(RefCell::new(globals.to_analyzer()));
        
        {
            let mut env = globals.borrow_mut();
            env.define(&Rc::from("any"),    UniLType::Type(Box::new(UniLType::Any)));
            env.define(&Rc::from("Null"),   UniLType::Type(Box::new(UniLType::Null)));
            env.define(&Rc::from("Int"),    UniLType::Type(Box::new(UniLType::Int)));
            env.define(&Rc::from("Bool"),   UniLType::Type(Box::new(UniLType::Int)));
            env.define(&Rc::from("Float"),  UniLType::Type(Box::new(UniLType::Float)));
            env.define(&Rc::from("Value"),  UniLType::Type(Box::new(UniLType::Value)));
            env.define(&Rc::from("String"), UniLType::Type(Box::new(UniLType::String)));
            env.define(&Rc::from("List"),   UniLType::Type(Box::new(UniLType::List)));
            env.define(&Rc::from("Object"), UniLType::Type(Box::new(object_type())));

            api_layers::define_analyzer(&mut env);
        }

        Analyzer { 
            environment: Rc::clone(&globals),
            globals,
            in_loop: false,
            curr_fn: None,
            errors: Vec::new() 
        }
    }

    fn op_unary_minus(&mut self, value: &UniLType, expr: &Expression) -> UniLType {
        if matches!(value, UniLType::Int | UniLType::Float | UniLType::Value | UniLType::Any) {
            value.clone()
        } else {
            ast_error!(
                self, expr, 
                format!("'-' operator cannot be used on type {}", value.stringify()).as_str()
            );

            UniLType::Any
        }
    }

    fn op_unary_tilde(&mut self, value: &UniLType, expr: &Expression) -> UniLType {
        if matches!(value, UniLType::Int | UniLType::Value | UniLType::Any) {
            value.clone()
        } else {
            ast_error!(
                self, expr, 
                format!("'~' operator cannot be used on type {}", value.stringify()).as_str()
            );
            
            UniLType::Any
        }
    }

    fn op_add(&mut self, left: &UniLType, right: &UniLType, expr: &Expression) -> UniLType {
        if matches!(left, UniLType::Any) || matches!(right, UniLType::Any) {
            return UniLType::Any;
        }

        match left {
            UniLType::Int | UniLType::Value => {
                if matches!(
                    right, 
                    UniLType::Int | UniLType::Float | UniLType::Value | 
                    UniLType::String
                ) {
                    return right.clone();
                }
            }
            UniLType::Float => {
                if matches!(
                    right,
                    UniLType::Int | UniLType::Float | UniLType::Value
                ) {
                    return UniLType::Float;
                }
            }
            UniLType::String => {
                if matches!(right, UniLType::Int | UniLType::String) {
                    return UniLType::String;
                }
            }
            _ => ()
        }

        ast_error!(
            self, expr, 
            format!(
                "'+' operator cannot be used on types {} and {}", 
                left.stringify(), right.stringify()
            ).as_str()
        );
        
        UniLType::Any
    }

    fn op_mul(&mut self, left: &UniLType, right: &UniLType, expr: &Expression) -> UniLType {
        if matches!(left, UniLType::Any) || matches!(right, UniLType::Any) {
            return UniLType::Any;
        }

        match left {
            UniLType::Int | UniLType::Value => {
                if matches!(
                    right, 
                    UniLType::Int | UniLType::Float | UniLType::Value | 
                    UniLType::String
                ) {
                    return right.clone();
                }
            }
            UniLType::Float => {
                if matches!(
                    right,
                    UniLType::Int | UniLType::Float | UniLType::Value
                ) {
                    return UniLType::Float;
                }
            }
            UniLType::String => {
                if matches!(right, UniLType::Int) {
                    return UniLType::String;
                }
            }
            _ => ()
        }

        ast_error!(
            self, expr, 
            format!(
                "'*' operator cannot be used on types {} and {}", 
                left.stringify(), right.stringify()
            ).as_str()
        );
        
        UniLType::Any
    }

   fn op_sub_div_mod(&mut self, left: &UniLType, right: &UniLType, expr: &Expression, op: &str) -> UniLType {
        if matches!(left, UniLType::Any) || matches!(right, UniLType::Any) {
            return UniLType::Any;
        }

        match left {
            UniLType::Int | UniLType::Value => {
                if matches!(
                    right, 
                    UniLType::Int | UniLType::Float | UniLType::Value | 
                    UniLType::String
                ) {
                    return right.clone();
                }
            }
            UniLType::Float => {
                if matches!(
                    right, 
                    UniLType::Int | UniLType::Float | UniLType::Value
                ) {
                    return UniLType::Float;
                }
            }
            _ => ()
        }

        ast_error!(
            self, expr, 
            format!(
                "'{}' operator cannot be used on types {} and {}", 
                op, left.stringify(), right.stringify()
            ).as_str()
        );
        
        UniLType::Any
    }

    fn op_bitwise(&self, left: &UniLType, right: &UniLType) -> UniLType {
        if matches!(left, UniLType::Value) && matches!(right, UniLType::Int | UniLType::Value) {
            UniLType::Value
        } else {
            UniLType::Int
        }
    }

    fn op_shift(&mut self, left: &UniLType, right: &UniLType, expr: &Expression, op: &str) -> UniLType {
        if matches!(left, UniLType::Any) || matches!(right, UniLType::Any) {
            return UniLType::Any;
        }

        if matches!(left, UniLType::Int | UniLType::Value) {
            return left.clone();
        }

        ast_error!(
            self, expr, 
            format!(
                "'{}' operator cannot be used on types {} and {}", 
                op, left.stringify(), right.stringify()
            ).as_str()
        );
        
        UniLType::Any
    }

    fn op_cmp(&mut self, left: &UniLType, right: &UniLType, expr: &Expression, op: &str) -> UniLType {
        if matches!(left, UniLType::Any) || matches!(right, UniLType::Any) {
            return UniLType::Int;
        }
        
        match left {
            UniLType::Int | UniLType::Value | UniLType::Float => {
                if matches!(right, UniLType::Int | UniLType::Value | UniLType::Float) {
                    return UniLType::Int;
                }
            }
            UniLType::String => {
                if matches!(right, UniLType::String) {
                    return UniLType::Int;
                }
            }
            _ => ()
        }

        ast_error!(
            self, expr, 
            format!(
                "'{}' operator cannot be used on types {} and {}", 
                op, left.stringify(), right.stringify()
            ).as_str()
        );
        
        UniLType::Int
    }

    fn variable_set(&mut self, name: &Token, value: UniLType, define: bool) {
        let mut env = self.environment.borrow_mut();
        if define {
            env.define(&name.lexeme, value.finalize());
        } else {
            if env.enclosing.is_none() { // if we're in globals
                if env.set_global(&name.lexeme, value.clone()).is_err() {
                    token_error!(
                        self, name, 
                        format!("Unknown variable '{}'", name.lexeme).as_str()
                    );
                }
            } else {
                if let Err(e) = env.set(&name.lexeme, value.clone()) {
                    match e {
                        AnalyzerEnvError::Unknown => {
                            token_error!(
                                self, name, 
                                format!("Unknown variable '{}'", name.lexeme).as_str()
                            );
                        }
                        AnalyzerEnvError::Global => {
                            token_error!(
                                self, name, 
                                format!("Cannot assign to global '{}' from local scope", name.lexeme).as_str()
                            );
                        }
                    }
                }
            }
            
        }
    }

    async fn assign(&mut self, target: &Expression, value: UniLType, expr: &Expression, define: bool, toplevel: bool, ctx: &mut reblessive::Stk) -> Result<UniLType, AnalyzerInterrupt> {
        if let Some(x) = target.get_variable() {
            if let Expression::Variable { name } = x {
                let env = self.environment.borrow();
                if let Some(type_) = env.get(&name.lexeme) {
                    drop(env);
                    if type_.equals(&value) {
                        self.variable_set(name, value.clone(), define);
                    } else {
                        ast_error!(
                            self, expr,
                            format!(
                                "Value type ({}) does not match target type ({})",
                                value.stringify(), type_.stringify()
                            ).as_ref()
                        );
                    }
                } else {
                    drop(env);
                    self.variable_set(name, value.clone(), define);
                }
            } else {
                unreachable!()
            }
        } else if let Some(x) = target.get_get() {
            if let Expression::Get { object: object_expr, name } = x {
                let from = ctx.run(|ctx| self.get_type(&object_expr, toplevel, ctx)).await?;

                match from {
                    UniLType::Any => (),
                    UniLType::Value => {
                        // handles some oSV methods
                        match name.lexeme.as_ref() {
                            "copy" | "noMark" | "read" | "getInt" | "readInt" |
                            "readNoMark" | "readDigit" => {
                                ast_error!(self, expr, "Cannot overwrite API compatibility functions");
                            }
                            _ => {
                                token_error!(
                                    self, name, 
                                    format!("Unknown property '{}'", name.lexeme).as_str()
                                );
                            }
                        }
                    }
                    UniLType::Object { fields } => {
                        let mut borrowed = fields.borrow_mut();
                        if let Some(type_) = borrowed.get(&name.lexeme) {
                            if type_.equals(&value) {
                                borrowed.insert(Rc::clone(&name.lexeme), value.clone().finalize());
                            } else {
                                ast_error!(
                                    self, expr,
                                    format!(
                                        "Value type ({}) does not match target type ({})",
                                        value.stringify(), type_.stringify()
                                    ).as_ref()
                                );
                            }
                        } else {
                            borrowed.insert(Rc::clone(&name.lexeme), value.clone().finalize());
                        }
                    }
                    _ => {
                        ast_error!(
                            self, expr, 
                            format!("Cannot write properties of type '{}'", from.stringify()).as_str()
                        );
                    }
                }
            } else {
                unreachable!()
            }
        } else if let Some(x) = target.get_subscript() {
            ctx.run(|ctx| self.get_type(x, toplevel, ctx)).await?;
        } else {
            unreachable!()
        }

        Ok(value)
    }
}

macro_rules! analyze_many {
    ($slf: ident, $expressions: ident, $expression: ident, $single: expr) => {
        {
            let mut last_type = UniLType::Null;
            for (i, $expression) in $expressions.iter().enumerate() {
                match $single {
                    Ok(t) => last_type = t,
                    Err(interrupt) => {
                        match interrupt {
                            AnalyzerInterrupt::Throw => (),
                            AnalyzerInterrupt::BreakOrContinue => {
                                if !$slf.in_loop {
                                    ast_error!($slf, $expressions[i], "Cannot use break or continue outside of a loop");
                                }
                            }
                            AnalyzerInterrupt::Return(returned) => {
                                if let Some(curr_fn) = &$slf.curr_fn {
                                    if !curr_fn.return_type.equals(&returned) {
                                        ast_error!(
                                            $slf, $expressions[i], 
                                            format!(
                                                "Returned type ({}) does not match function return type ({})",
                                                returned.stringify(), curr_fn.return_type.stringify()
                                            ).as_str()
                                        );

                                        ast_note!(curr_fn.definition, "Return type defined here");
                                    }
                                } else {
                                    ast_error!($slf, $expressions[i], "Cannot use return outside of a function");
                                }
                            }
                        }
                    }
                }
                
            }

            last_type
        }
    };
}

impl Analyzer {
    async fn get_types(&mut self, expressions: &[Expression], toplevel: bool, ctx: &mut reblessive::Stk) -> UniLType {
        analyze_many!(self, expressions, expression, ctx.run(|ctx| self.get_type(expression, toplevel, ctx)).await)
    }

    async fn get_type(&mut self, expr: &Expression, toplevel: bool, ctx: &mut reblessive::Stk) -> Result<UniLType, AnalyzerInterrupt> {
        match expr {
            Expression::Break { .. } | Expression::Continue { .. } => Err(AnalyzerInterrupt::BreakOrContinue),
            Expression::Grouping { inner, .. } => ctx.run(|ctx| self.get_type(&inner, toplevel, ctx)).await,
            Expression::Block { expressions, .. } => Ok(ctx.run(|ctx| self.get_types(&expressions, toplevel, ctx)).await),
            Expression::ScopedBlock { expressions, .. } => {
                let previous = Rc::clone(&self.environment);
                self.environment = Rc::new(RefCell::new(AnalyzerEnvironment::with_enclosing(Rc::clone(&self.globals))));
                let ret = ctx.run(|ctx| self.get_types(expressions, toplevel, ctx)).await;
                self.environment = previous;
                Ok(ret)
            }
            Expression::Literal { value, kind, .. } => {
                match kind {
                    LiteralKind::String => Ok(UniLType::String),
                    LiteralKind::Null => Ok(UniLType::Null),
                    LiteralKind::Int => {
                        if value.parse::<i64>().is_err() {
                            ast_error!(self, expr, "Invalid integer literal");
                        }

                        Ok(UniLType::Int)
                    }
                    LiteralKind::Float => {
                        if value.parse::<f64>().is_err() {
                            ast_error!(self, expr, "Invalid float literal");
                        }

                        Ok(UniLType::Float)
                    }
                }
            }
            Expression::Unary { op, expr: inner_expr, is_prefix, .. } => {
                let inner = ctx.run(|ctx| self.get_type(&inner_expr, toplevel, ctx)).await?;

                if matches!(op.type_, TokenType::PlusPlus | TokenType::MinusMinus) {
                    let new = {
                        match op.type_ {
                            TokenType::PlusPlus   => self.op_add(&inner, &UniLType::Int, expr),
                            TokenType::MinusMinus => self.op_sub_div_mod(&inner, &UniLType::Int, expr, "-"),
                            _ => unreachable!()
                        }
                    };

                    let new = ctx.run(|ctx| self.assign(&inner_expr, new, expr, false, toplevel, ctx)).await?;

                    return if *is_prefix {
                        Ok(new)
                    } else {
                        Ok(inner)
                    };
                }

                if *is_prefix {
                    match op.type_ {
                        TokenType::Bang  => Ok(UniLType::Int),
                        TokenType::Minus => Ok(self.op_unary_minus(&inner, expr)),
                        TokenType::Tilde => Ok(self.op_unary_tilde(&inner, expr)),
                        _ => {
                            token_error!(
                                self, op, 
                                format!(
                                    "Malformed AST: Unary prefix op cannot be {}",
                                    op.type_.stringify()
                                ).as_str()
                            );

                            Ok(UniLType::Any)
                        }
                    }
                } else {
                    token_error!(
                        self, op, 
                        format!(
                            "Malformed AST: Unary suffix op cannot be {}",
                            op.type_.stringify()
                        ).as_str()
                    );

                    Ok(UniLType::Any)
                }
            }
            Expression::Binary { left: left_expr, op, right: right_expr, .. } => {
                let left = ctx.run(|ctx| self.get_type(&left_expr, toplevel, ctx)).await?;
                let right = ctx.run(|ctx| self.get_type(&right_expr, toplevel, ctx)).await?;

                Ok(match op.type_ {
                    TokenType::Plus       => self.op_add(&left, &right, expr),
                    TokenType::Star       => self.op_mul(&left, &right, expr),
                    TokenType::Slash      => self.op_sub_div_mod(&left, &right, expr, "/"),
                    TokenType::Mod        => self.op_sub_div_mod(&left, &right, expr, "%"),
                    TokenType::Minus      => self.op_sub_div_mod(&left, &right, expr, "-"),
                    TokenType::ShiftLeft  => self.op_shift(&left, &right, expr, "<<"),
                    TokenType::ShiftRight => self.op_shift(&left, &right, expr, ">>"),
                    TokenType::BitwiseAnd => self.op_bitwise(&left, &right),
                    TokenType::BitwiseXor => self.op_bitwise(&left, &right),
                    TokenType::BitwiseOr  => self.op_bitwise(&left, &right),
                    _ => {
                        token_error!(
                            self, op, 
                            format!(
                                "Malformed AST: Binary op cannot be {}",
                                op.type_.stringify()
                            ).as_str()
                        );
    
                        UniLType::Any
                    }
                })
            }
            Expression::Cmp { left: left_expr, op, right: right_expr, .. } => {
                let left = ctx.run(|ctx| self.get_type(&left_expr, toplevel, ctx)).await?;
                let right = ctx.run(|ctx| self.get_type(&right_expr, toplevel, ctx)).await?;

                Ok(match op.type_ {
                    TokenType::Less         => self.op_cmp(&left, &right, expr, "<"),
                    TokenType::LessEqual    => self.op_cmp(&left, &right, expr, "<="),
                    TokenType::Greater      => self.op_cmp(&left, &right, expr, ">"),
                    TokenType::GreaterEqual => self.op_cmp(&left, &right, expr, ">="),
                    TokenType::EqualEqual | TokenType::BangEqual => UniLType::Int,
                    _ => {
                        token_error!(
                            self, op, 
                            format!(
                                "Malformed AST: Cmp op cannot be {}",
                                op.type_.stringify()
                            ).as_str()
                        );
    
                        UniLType::Any
                    }
                })
            }
            Expression::Logic { left: left_expr, right: right_expr, .. } => {
                let left = ctx.run(|ctx| self.get_type(&left_expr, toplevel, ctx)).await?;
                let right = ctx.run(|ctx| self.get_type(&right_expr, toplevel, ctx)).await?;
                Ok(left.make_group(right))
            }
            Expression::Assign { target, op, value: value_expr, type_spec } => {
                let value = {
                    if let Some(type_spec) = type_spec {
                        if !matches!(op.type_, TokenType::Walrus) {
                            token_error!(
                                self, op,
                                "Malformed AST: Type specification in assignment can only be used with ':=' operator"
                            );
                        }

                        let type_ = ctx.run(|ctx| self.get_type(&type_spec, toplevel, ctx)).await?;
                        let value = ctx.run(|ctx| self.get_type(&value_expr, toplevel, ctx)).await?;

                        if let UniLType::Type(inner) = type_ {
                            if inner.equals(&value) {
                                *inner
                            } else {
                                ast_error!(
                                    self, value_expr,
                                    format!(
                                        "Value type ({}) does not match declared type ({})",
                                        value.stringify(), inner.stringify()
                                    ).as_ref()
                                );

                                UniLType::Any
                            }
                        } else {
                            ast_error!(
                                self, value_expr,
                                format!(
                                    "Expecting type as type specifier (got {})",
                                    type_.stringify()
                                ).as_ref()
                            );

                            UniLType::Any
                        }
                    } else {
                        ctx.run(|ctx| self.get_type(&value_expr, toplevel, ctx)).await?
                    }
                };

                let mut define = false;
                let final_value = {
                    if matches!(op.type_, TokenType::Equal) {
                        value
                    } else if matches!(op.type_, TokenType::Walrus) {
                        define = true;
                        value
                    } else {
                        let before_value = ctx.run(|ctx| self.get_type(&target, toplevel, ctx)).await?;
                        
                        match op.type_ {
                            TokenType::PlusEquals       => self.op_add(&before_value, &value, expr),
                            TokenType::StarEquals       => self.op_mul(&before_value, &value, expr),
                            TokenType::MinusEquals      => self.op_sub_div_mod(&before_value, &value, expr, "-"),
                            TokenType::SlashEquals      => self.op_sub_div_mod(&before_value, &value, expr, "/"),
                            TokenType::ModEquals        => self.op_sub_div_mod(&before_value, &value, expr, "%"),
                            TokenType::ShiftLeftEquals  => self.op_shift(&before_value, &value, expr, "<<"),
                            TokenType::ShiftRightEquals => self.op_shift(&before_value, &value, expr, ">>"),
                            TokenType::AndEquals | TokenType::XorEquals | TokenType::OrEquals => {
                                self.op_bitwise(&before_value, &value)
                            }
                            _ => {
                                token_error!(
                                    self, op, 
                                    format!(
                                        "Malformed AST: Assign op cannot be {}",
                                        op.type_.stringify()
                                    ).as_str()
                                );
            
                                UniLType::Any
                            }
                        }
                    }
                };

                ctx.run(|ctx| self.assign(&target, final_value, expr, define, toplevel, ctx)).await
            }
            Expression::Variable { name, .. } => {
                if let Some(value) = self.environment.borrow().get(&name.lexeme) {
                    Ok(value)
                } else {
                    token_error!(
                        self, name, 
                        format!("Unknown variable '{}'", name.lexeme).as_str()
                    );

                    Ok(UniLType::Any)
                }
            }
            Expression::If { condition, then_branch, else_branch, .. } => {
                ctx.run(|ctx| self.get_type(&condition, toplevel, ctx)).await?;
                let then = ctx.run(|ctx| self.get_type(&then_branch, toplevel, ctx)).await?;

                if let Some(else_branch) = else_branch {
                    let else_ = ctx.run(|ctx| self.get_type(&else_branch, toplevel, ctx)).await?;
                    Ok(then.make_group(else_))
                } else {
                    Ok(then.make_group(UniLType::Null))
                }
            }
            Expression::Ternary { condition, then_expr, else_expr, .. } => {
                ctx.run(|ctx| self.get_type(&condition, toplevel, ctx)).await?;
                let then = ctx.run(|ctx| self.get_type(&then_expr, toplevel, ctx)).await?;
                let else_ = ctx.run(|ctx| self.get_type(&else_expr, toplevel, ctx)).await?;
                Ok(then.make_group(else_))
            }
            Expression::While { condition, body, increment, .. } => {
                ctx.run(|ctx| self.get_type(&condition, toplevel, ctx)).await?;

                if let Some(increment) = increment {
                    ctx.run(|ctx| self.get_type(&increment, toplevel, ctx)).await?;
                }

                let previous_in_loop = self.in_loop;
                self.in_loop = true;
                let ret = ctx.run(|ctx| self.get_type(&body, toplevel, ctx)).await;
                self.in_loop = previous_in_loop;
                ret
            }
            Expression::DoWhile { condition, body, .. } => {
                ctx.run(|ctx| self.get_type(&condition, toplevel, ctx)).await?;

                let previous_in_loop = self.in_loop;
                self.in_loop = true;
                let ret = ctx.run(|ctx| self.get_type(&body, toplevel, ctx)).await;
                self.in_loop = previous_in_loop;
                ret
            }
            Expression::Switch { expr, cases, .. } => {
                ctx.run(|ctx| self.get_type(&expr, toplevel, ctx)).await?;

                let mut type_ = UniLType::Null;
                for branch in cases {
                    if let Some(branch_cases) = &branch.cases {
                        for case in branch_cases {
                            ctx.run(|ctx| self.get_type(&case, toplevel, ctx)).await?;
                        }
                    }

                    let branch_type = ctx.run(|ctx| self.get_types(&branch.code, toplevel, ctx)).await;
                    type_ = type_.make_group(branch_type);
                }

                Ok(type_)
            }
            Expression::Call { callee: callee_expr, args: args_expr, paren } => {
                if args_expr.len() > 255 {
                    ast_error!(self, callee_expr, "Cannot call a function with more than 255 arguments");
                }

                let callee = ctx.run(|ctx| self.get_type(&callee_expr, toplevel, ctx)).await?;

                match callee {
                    UniLType::Any => Ok(UniLType::Any),
                    UniLType::Callable { args, return_type } => {
                        if args_expr.len() != args.len() {
                            token_error!(
                                self, paren, 
                                format!(
                                    "Expecting {} arguments but got {}", 
                                    args.len(), args_expr.len()
                                ).as_str()
                            );
                            
                            return Ok(UniLType::Any);
                        }

                        for i in 0 .. args.len() {
                            let arg_type = ctx.run(|ctx| self.get_type(&args_expr[i], toplevel, ctx)).await?;
                            if !args[i].equals(&arg_type) {
                                ast_error!(
                                    self, args_expr[i],
                                    format!(
                                        "Argument type does not match parameter type (expecting {} but got {})",
                                        args[i].stringify(), arg_type.stringify()
                                    ).as_str()
                                );
                            }
                        }

                        Ok(*return_type)
                    }
                    _ => {
                        ast_error!(
                            self, callee_expr,
                            format!("Cannot call type {}", callee.stringify()).as_str()
                        );

                        Ok(UniLType::Any)
                    }
                }
            }
            Expression::Function { name, params: params_expr, return_type: return_expr, body } => {
                if params_expr.len() > 255 {
                    token_error!(self, name, "Cannot define a function with more than 255 parameters");
                }

                let return_type = {
                    let type_ = ctx.run(|ctx| self.get_type(&return_expr, toplevel, ctx)).await?;
                    if let UniLType::Type(inner) = type_ {
                        *inner
                    } else {
                        ast_error!(
                            self, return_expr,
                            format!(
                                "Expecting type as return type (got {})",
                                type_.stringify()
                            ).as_str()
                        );

                        UniLType::Any
                    }
                };

                let previous_fn;
                let previous_env;
                if toplevel {
                    previous_fn = None;
                    previous_env = None;
                } else {
                    previous_fn = Some(self.curr_fn.clone());
                    self.curr_fn = Some(CurrFn { return_type: return_type.clone(), definition: *return_expr.clone() });

                    previous_env = Some(Rc::clone(&self.environment));
                    self.environment = Rc::new(RefCell::new(AnalyzerEnvironment::with_enclosing(Rc::clone(&self.environment))));
                }
                    
                let mut args = Vec::new();
                for param in params_expr {
                    if let Some(type_spec) = &param.expr {
                        let type_ = ctx.run(|ctx| self.get_type(type_spec, toplevel, ctx)).await?;
                        if let UniLType::Type(inner) = type_ {
                            if !toplevel {
                                self.environment.borrow_mut().define(&param.name.lexeme, inner.clone().finalize());
                            }
                            
                            args.push(*inner);
                        } else {
                            ast_error!(
                                self, return_expr,
                                format!(
                                    "Expecting type as parameter type (got {})",
                                    type_.stringify()
                                ).as_str()
                            );

                            if !toplevel {
                                self.environment.borrow_mut().define(&param.name.lexeme, UniLType::Any);
                            }

                            args.push(UniLType::Any);
                        }
                    } else {
                        if !toplevel {
                            self.environment.borrow_mut().define(&param.name.lexeme, UniLType::Any);
                        }

                        args.push(UniLType::Any);
                    }
                }

                let function_type = UniLType::Callable { args, return_type: Box::new(return_type) };
                self.globals.borrow_mut().define(&name.lexeme, function_type.clone());

                if !toplevel {
                    ctx.run(|ctx| self.get_types(body, toplevel, ctx)).await;

                    self.environment = previous_env.unwrap();
                    self.curr_fn = previous_fn.unwrap();
                }
                
                Ok(function_type)
            }
            Expression::Throw { value: value_expr, .. } => {
                if let Some(value) = value_expr {
                    ctx.run(|ctx| self.get_type(&value, toplevel, ctx)).await?;
                }

                Err(AnalyzerInterrupt::Throw)
            } 
            Expression::Return { value: value_expr, .. } => {
                if let Some(value) = value_expr {
                    let type_ = ctx.run(|ctx| self.get_type(&value, toplevel, ctx)).await?;
                    Err(AnalyzerInterrupt::Return(type_))
                } else {
                    Err(AnalyzerInterrupt::Return(UniLType::Null))
                }
            }
            Expression::Get { object: object_expr, name, .. } => {
                let from = ctx.run(|ctx| self.get_type(&object_expr, toplevel, ctx)).await?;

                match from {
                    UniLType::Any => Ok(UniLType::Any),
                    UniLType::Value => {
                        // handles some oSV methods
                        match name.lexeme.as_ref() {
                            "copy" | "noMark" | "read" | "getInt" | "readInt" |
                            "readNoMark" | "readDigit" => Ok(UniLType::Any),
                            _ => {
                                token_error!(
                                    self, name, 
                                    format!("Unknown property '{}'", name.lexeme).as_str()
                                );

                                Ok(UniLType::Any)
                            }
                        }
                    }
                    UniLType::Object { fields } => {
                        if let Some(field) = fields.borrow().get(&name.lexeme) {
                            Ok(field.clone())
                        } else {
                            Ok(UniLType::Any)
                        }
                    }
                    _ => {
                        ast_error!(
                            self, object_expr,
                            format!(
                                "Cannot read properties of type '{}'", 
                                from.stringify()
                            ).as_str()
                        );
                        
                        Ok(UniLType::Any)
                    }
                }
            }
            Expression::AnonObject { fields: fields_expr, .. } => {
                let mut fields = HashMap::new();

                for field in fields_expr {
                    ctx.run(|ctx| self.get_type(&field.expr, toplevel, ctx)).await?;
                    
                    let type_ = {
                        let value = ctx.run(|ctx| self.get_type(&field.expr, toplevel, ctx)).await?;

                        if let Some(type_spec) = &field.type_ {    
                            let type_ = ctx.run(|ctx| self.get_type(&type_spec, toplevel, ctx)).await?;

                            if let UniLType::Type(inner) = type_ {
                                if inner.equals(&value) {
                                    *inner
                                } else {
                                    ast_error!(
                                        self, field.expr,
                                        format!(
                                            "Value type ({}) does not match declared type ({})",
                                            value.stringify(), inner.stringify()
                                        ).as_ref()
                                    );

                                    UniLType::Any
                                }
                            } else {
                                ast_error!(
                                    self, type_spec,
                                    format!(
                                        "Expecting type as type specifier (got {})",
                                        type_.stringify()
                                    ).as_ref()
                                );

                                UniLType::Any
                            }
                        } else {
                            value
                        }
                    };

                    if fields.insert(Rc::clone(&field.name.lexeme), type_.finalize()).is_some() {
                        token_error!(
                            self, field.name,
                            format!("Field '{}' was already defined", field.name.lexeme).as_str()
                        );
                    }
                }

                Ok(UniLType::Object { fields: Rc::new(RefCell::new(fields)) })
            }
            Expression::AlgoDecl { name, object: object_expr, function } => {
                let mut expected_arity = None;
                let mut expected_fields: Option<HashMap<Rc<str>, UniLType>> = None;

                match name.lexeme.as_ref() {
                    "sort" => {
                        expected_arity = Some(1);
                        expected_fields = Some(HashMap::from([
                            (Rc::from("name"), UniLType::String),
                            (Rc::from("category"), UniLType::String),
                            (Rc::from("listName"), UniLType::String),
                        ]));
                    }
                    "shuffle" | "distribution" => {
                        expected_arity = Some(1);
                        expected_fields = Some(HashMap::from([(Rc::from("name"), UniLType::String)]));
                    } 
                    "pivotSelection" => {
                        expected_arity = Some(3);
                        expected_fields = Some(HashMap::from([(Rc::from("name"), UniLType::String)]));
                    }
                    "rotation" | "indexedRotation" | "lengthsRotation" => {
                        expected_arity = Some(4);
                        expected_fields = Some(HashMap::from([(Rc::from("name"), UniLType::String)]));
                    }
                    _ => {
                        token_error!(
                            self, name,
                            format!("Invalid algorithm type '{}'", name.lexeme).as_str()
                        );
                    }
                }

                let decl = ctx.run(|ctx| self.get_type(&object_expr, toplevel, ctx)).await?;
                if let UniLType::Object { fields } = decl {
                    if let Some(expected) = expected_fields {
                        let borrowed = fields.borrow();
                        if borrowed.len() >= expected.len() {
                            for (expected_field_name, expected_field_type) in expected {
                                if let Some(actual_field_type) = borrowed.get(&expected_field_name) {
                                    if !expected_field_type.equals(actual_field_type) {
                                        ast_error!(
                                            self, object_expr, 
                                            format!(
                                                "Expecting '{}' field of '{}' algorithm declaration to be of type {} but got {}", 
                                                expected_field_name, name.lexeme, 
                                                expected_field_type.stringify(), actual_field_type.stringify() 
                                            ).as_str()
                                        );
                                    }
                                } else {
                                    ast_error!(
                                        self, object_expr, 
                                        format!(
                                            "'{}' algorithm declaration is missing '{}' field", 
                                            name.lexeme, expected_field_name
                                        ).as_str()
                                    );
                                }
                            }
                        } else {
                            ast_error!(
                                self, object_expr, 
                                format!(
                                    "Expecting at least {} fields for '{}' algorithm declaration but got {}", 
                                    expected.len(), name.lexeme, borrowed.len()
                                ).as_str()
                            );
                        }
                    }
                } else {
                    ast_error!(self, object_expr, "Malformed AST: AlgoDecl object can only be AnonObject");
                }

                if let Expression::Function { params, name: tok, .. } = &**function {
                    if let Some(arity) = expected_arity {
                        if arity != params.len() {
                            token_error!(
                                self, tok, 
                                format!(
                                    "Expecting {} parameters for '{}' algorithm but got {}",
                                    arity, name.lexeme, params.len()
                                ).as_str()
                            )
                        }
                    }

                    ctx.run(|ctx| self.get_type(&function, toplevel, ctx)).await
                } else {
                    ast_error!(self, function, "Malformed AST: AlgoDecl function can only be Function");
                    Ok(UniLType::Any)
                }
            }
            Expression::Subscript { subscripted: subscripted_expr, index: index_expr, .. } => {
                let subscripted = ctx.run(|ctx| self.get_type(&subscripted_expr, toplevel, ctx)).await?;
                let index       = ctx.run(|ctx| self.get_type(&index_expr, toplevel, ctx)).await?;

                if !matches!(subscripted, UniLType::List | UniLType::Any) {
                    ast_error!(
                        self, subscripted_expr,
                        format!(
                            "Type {} is not indexable", 
                            subscripted.stringify()
                        ).as_str()
                    );
                }

                if !matches!(index, UniLType::Int | UniLType::Value | UniLType::Any) {
                    ast_error!(
                        self, index_expr,
                        format!(
                            "List index must be of type Int or Value, not {}", 
                            index.stringify()
                        ).as_str()
                    );
                }

                Ok(UniLType::Any)
            }
            Expression::Try { try_branch: try_expr, catch_branch, catch_var, .. } => {
                let try_branch = ctx.run(|ctx| self.get_type(&try_expr, toplevel, ctx)).await?;

                if let Some(catch_branch) = catch_branch {
                    let previous = Rc::clone(&self.environment);
                    self.environment = Rc::new(RefCell::new(AnalyzerEnvironment::with_enclosing(Rc::clone(&previous))));

                    if let Some(var_name) = catch_var {
                        self.environment.borrow_mut().define(&var_name.lexeme, UniLType::Any);
                    }

                    let catch = ctx.run(|ctx| self.get_type(&catch_branch, toplevel, ctx)).await?;
                    self.environment = previous;

                    Ok(try_branch.make_group(catch))
                } else {
                    Ok(try_branch.make_group(UniLType::Null))
                }
            }
            Expression::List { items, .. } => {
                for item in items {
                    ctx.run(|ctx| self.get_type(&item, toplevel, ctx)).await?;
                }

                Ok(UniLType::List)
            }
            Expression::Foreach { iterator: iterator_expr, body, variable, .. } => {
                let iterator = ctx.run(|ctx| self.get_type(&iterator_expr, toplevel, ctx)).await?;

                if !matches!(iterator, UniLType::Any | UniLType::List | UniLType::Object { .. }) {
                    ast_error!(
                        self, iterator_expr,
                        format!(
                            "Type {} is not iterable", 
                            iterator.stringify()
                        ).as_str()
                    );
                }

                // TODO: although a bit convoluted, it should be possible to provide some type inference for the loop variable,
                // but we would need more syntax to specify types for loop variables, since we NEED to be able to 
                // make it `any` for language layers supporting dynamic languages (note for future self: this would need a .finalize() call)
                self.environment.borrow_mut().define(&variable.lexeme, UniLType::Any);

                let previous_in_loop = self.in_loop;
                self.in_loop = true;
                let ret = ctx.run(|ctx| self.get_type(&body, toplevel, ctx)).await;
                self.in_loop = previous_in_loop;
                ret
            }
            Expression::Drop { variable, .. } => {
                let mut env = self.environment.borrow_mut();
                if env.del(&variable.lexeme).is_err() {
                    token_error!(
                        self, variable, 
                        format!("Unknown variable '{}'", variable.lexeme).as_str()
                    );
                }

                Ok(UniLType::Null)
            }
        }
    }
    
    pub fn analyze(&mut self, expressions: &[Expression]) -> UniLType {
        let mut stack = reblessive::Stack::new();
        // first, load all the toplevel definitions, so that all globals are there
        analyze_many!(self, expressions, expression, stack.enter(|ctx| self.get_type(&expression, true, ctx)).finish());
        // now analyze everything, and definition order doesn't matter... *magic*
        analyze_many!(self, expressions, expression, stack.enter(|ctx| self.get_type(&expression, false, ctx)).finish())
    }
}