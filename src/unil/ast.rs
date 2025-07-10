use std::rc::Rc;

use crate::{unil::tokens::Token, utils::lang::AstPos};

const INDENT_SIZE: usize = 4;

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Hash)]
pub enum LiteralKind {
    Int, Float, String, Null
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct NamedExpr {
    pub name: Token,
    pub expr: Option<Expression>,
}

impl NamedExpr {
    pub fn new(name: Token, expr: Option<Expression>) -> Self {
        NamedExpr { name, expr }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ObjectField {
    pub name: Token,
    pub expr: Expression,
    pub type_: Option<Expression>,
}

impl ObjectField {
    pub fn new(name: Token, expr: Expression, type_: Option<Expression>) -> Self {
        ObjectField { name, expr, type_ }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct SwitchCase {
    pub cases: Option<Vec<Expression>>, // is none when default
    pub code: Vec<Expression>
}

impl SwitchCase {
    pub fn new(cases: Option<Vec<Expression>>, code: Vec<Expression>) -> Self {
        SwitchCase { cases, code }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Expression {
    Break { kw: Token, value: Option<Box<Expression>> },
    Continue { kw: Token, value: Option<Box<Expression>> },
    Grouping { inner: Box<Expression> },
    Variable{ name: Token },
    Binary { left: Box<Expression>, op: Token, right: Box<Expression> },
    Cmp { left: Box<Expression>, op: Token, right: Box<Expression> },
    Logic { left: Box<Expression>, op: Token, right: Box<Expression> },
    Literal { value: Rc<str>, tok: Token, kind: LiteralKind },
    Unary { op: Token, expr: Box<Expression>, is_prefix: bool },
    Assign { target: Box<Expression>, op: Token, value: Box<Expression>, type_spec: Option<Box<Expression>> },
    Call { callee: Box<Expression>, paren: Token, args: Vec<Expression> },
    Ternary { question_tok: Token, condition: Box<Expression>, then_expr: Box<Expression>, else_expr: Box<Expression> },
    Subscript { subscripted: Box<Expression>, paren: Token, index: Box<Expression> },
    Get { object: Box<Expression>, name: Token },
    List { opening_brace: Token, items: Vec<Expression> },
    Block { opening_brace: Token, expressions: Vec<Expression> },
    ScopedBlock { dollar: Token, expressions: Vec<Expression> },
    If { kw: Token, condition: Box<Expression>, then_branch: Box<Expression>, else_branch: Option<Box<Expression>> },
    While { kw: Token, condition: Box<Expression>, body: Box<Expression>, increment: Option<Box<Expression>> },
    DoWhile { kw: Token, condition: Box<Expression>, body: Box<Expression> },
    Function { name: Token, params: Vec<NamedExpr>, return_type: Box<Expression>, body: Vec<Expression> },
    Return { kw: Token, value: Option<Box<Expression>> },
    Switch { kw: Token, expr: Box<Expression>, cases: Vec<SwitchCase> },
    Foreach { kw: Token, variable: Token, iterator: Box<Expression>, body: Box<Expression> },
    AnonObject { kw: Token, fields: Vec<ObjectField> },
    AlgoDecl { name: Token, object: Box<Expression>, function: Box<Expression> },
    Throw { kw: Token, value: Option<Box<Expression>> },
    Try { kw: Token, try_branch: Box<Expression>, catch_branch: Option<Box<Expression>>, catch_var: Option<Token> },
    Drop { kw: Token, variable: Token },
}

fn get_indent(amt: usize) -> String {
    " ".repeat(amt * INDENT_SIZE)
}

fn needs_semicolon(expression: &Expression) -> bool {
    match expression {
        Expression::Block { .. } |
        Expression::Function { .. } |
        Expression::Switch { .. } | 
        Expression::AlgoDecl { .. } => false,
        Expression::While { body, .. } | 
        Expression::Foreach { body, .. } => needs_semicolon(body),
        Expression::If { then_branch, else_branch, .. } => {
            if let Some(else_) = else_branch {
                needs_semicolon(&else_)
            } else {
                needs_semicolon(&then_branch)
            }
        } 
        Expression::Try { try_branch, catch_branch, .. } => {
            if let Some(catch) = catch_branch {
                needs_semicolon(&catch)
            } else {
                needs_semicolon(&try_branch)
            }
        }
        _ => true
    }
}

fn block_codegen(expressions: &Vec<Expression>, indent: usize, needs_indent: bool, scoped: bool) -> String {
    let mut result = String::new();

    if needs_indent {
        result.push_str(&get_indent(indent));
    }

    if scoped {
        result.push('$');
    }
    
    result.push_str("{\n");

    for expression in expressions {
        result.push_str(&get_indent(indent + 1));
        result.push_str(&expression.codegen_inner(indent + 1, false));

        if needs_semicolon(expression) {
            result.push_str(";\n");
        } else {
            result.push('\n');
        }
    }

    result.push_str(&get_indent(indent));
    result.push('}');
    result
}

impl Expression {
    pub fn get_subscript(&self) -> Option<&Expression> {
        match self {
            Expression::Grouping { inner } => inner.get_subscript(),
            Expression::Subscript { .. } => Some(self),
            _ => None
        }
    }

    pub fn get_variable(&self) -> Option<&Expression> {
        match self {
            Expression::Grouping { inner } => inner.get_variable(),
            Expression::Variable { .. } => Some(self),
            _ => None
        }
    }

    pub fn get_get(&self) -> Option<&Expression> {
        match self {
            Expression::Grouping { inner } => inner.get_get(),
            Expression::Get { .. } => Some(self),
            _ => None
        }
    }

    pub fn is_valid_assignment_target(&self) -> bool {
        match self {
            Expression::Variable { .. } | Expression::Get {..} | Expression::Subscript { .. } => true,
            Expression::Grouping { inner } => inner.is_valid_assignment_target(),
            _ => false
        }
    }

    pub fn get_pos(&self) -> AstPos {
        match self {
            Expression::Grouping { inner } => inner.get_pos(),
            Expression::Literal { tok, .. } | Expression::Variable{ name: tok, ..} => {
                AstPos::new(Rc::clone(&tok.source), Rc::clone(&tok.filename), tok.pos, tok.end, tok.line)
            }
            Expression::Binary { left,  right, .. } |
            Expression::Logic { left, right, .. } |
            Expression::Cmp { left,  right, .. } => {
                let left_pos = left.get_pos();
                let right_pos = right.get_pos();

                if left_pos.line != right_pos.line || left_pos.filename != right_pos.filename {
                    left_pos
                } else {
                    AstPos::new(Rc::clone(&left_pos.source), Rc::clone(&left_pos.filename), left_pos.start, right_pos.end, left_pos.line)
                }
            }
            Expression::Unary { op, expr, is_prefix, .. } => {
                let expr_pos = expr.get_pos();

                if *is_prefix {
                    if op.line != expr_pos.line || op.filename != expr_pos.filename {
                        AstPos::new(Rc::clone(&op.source), Rc::clone(&op.filename), op.pos, op.end, op.line)
                    } else {
                        AstPos::new(Rc::clone(&op.source), Rc::clone(&op.filename), op.pos, expr.get_pos().end, op.line)
                    }
                } else {
                    if expr_pos.line != op.line || op.filename != expr_pos.filename {
                        expr_pos
                    } else {
                        AstPos::new(Rc::clone(&expr_pos.source), Rc::clone(&expr_pos.filename), expr_pos.start, op.end, expr_pos.line)
                    }
                }
            }
            Expression::Assign { target, value, .. } => {
                let target_pos = target.get_pos();
                let value_pos = value.get_pos();

                if target_pos.line != value_pos.line || target_pos.filename != value_pos.filename {
                    AstPos::new(Rc::clone(&target_pos.source), Rc::clone(&target_pos.filename), target_pos.start, target_pos.end, target_pos.line)
                } else {
                    AstPos::new(Rc::clone(&target_pos.source), Rc::clone(&target_pos.filename), target_pos.start, value_pos.end, target_pos.line)
                }
            }
            Expression::Call { callee, paren, .. } => {
                let callee_pos = callee.get_pos();

                if callee_pos.line != paren.line || callee_pos.filename != paren.filename {
                    callee_pos
                } else {
                    AstPos::new(Rc::clone(&callee_pos.source), Rc::clone(&callee_pos.filename), callee_pos.start, paren.end, callee_pos.line)
                }
            }
            Expression::Ternary { condition: cond, else_expr: else_, .. } => {
                let cond_pos = cond.get_pos();
                let else_pos = else_.get_pos();

                if cond_pos.line != else_pos.line || cond_pos.filename != else_pos.filename {
                    cond_pos
                } else {
                    AstPos::new(Rc::clone(&cond_pos.source), Rc::clone(&cond_pos.filename), cond_pos.start, else_pos.end, cond_pos.line)
                }
            }
            Expression::Subscript { subscripted, paren, .. } => {
                let subscripted_pos = subscripted.get_pos();

                if subscripted_pos.line != paren.line || subscripted_pos.filename != paren.filename {
                    subscripted_pos
                } else {
                    AstPos::new(Rc::clone(&subscripted_pos.source), Rc::clone(&subscripted_pos.filename), subscripted_pos.start, paren.end, subscripted_pos.line)
                }
            }
            Expression::Get { object, name, .. } => {
                let object_pos = object.get_pos();

                if object_pos.line != name.line || object_pos.filename != name.filename {
                    AstPos::new(Rc::clone(&name.source), Rc::clone(&name.filename), name.pos, name.end, name.line)
                } else {
                    AstPos::new(Rc::clone(&object_pos.source), Rc::clone(&object_pos.filename), object_pos.start, name.end, object_pos.line)
                }
            },
            Expression::List { opening_brace: open_paren, items: exprs, .. } => {
                match exprs.len() {
                    0 => {
                        AstPos::new(
                            Rc::clone(&open_paren.source),
                            Rc::clone(&open_paren.filename),
                            open_paren.pos,
                            open_paren.end,
                            open_paren.line
                        )
                    }
                    1 => exprs[0].get_pos(),
                    _ => {
                        let first_pos = exprs[0].get_pos();
                        let last_pos = exprs.last().unwrap().get_pos();

                        if first_pos.line != last_pos.line || first_pos.filename != last_pos.filename {
                            first_pos
                        } else {
                            AstPos::new(Rc::clone(&first_pos.source), Rc::clone(&first_pos.filename), first_pos.start, last_pos.end, first_pos.line)
                        }
                    }
                }
            }
            Expression::Block { opening_brace: tok, .. } |
            Expression::ScopedBlock { dollar: tok, .. } |
            Expression::If { kw: tok, .. } |
            Expression::While { kw: tok, .. } |
            Expression::DoWhile { kw: tok, .. } |
            Expression::Function { name: tok, .. } |
            Expression::Return { kw: tok, .. } |
            Expression::Switch { kw: tok, .. } |
            Expression::Break { kw: tok, .. } |
            Expression::Continue { kw: tok, .. } |
            Expression::Foreach { kw: tok, .. } |
            Expression::AnonObject { kw: tok, .. } |
            Expression::Throw { kw: tok, .. } |
            Expression::Drop { kw: tok, .. } |
            Expression::Try { kw: tok, .. } |
            Expression::AlgoDecl { name: tok, .. } => {
                AstPos::new(Rc::clone(&tok.source), Rc::clone(&tok.filename), tok.pos, tok.end, tok.line)
            }
        }
    }

    fn codegen_inner(&self, indent: usize, needs_indent: bool) -> String {
        match self {
            Expression::Break { .. } => String::from("break"),
            Expression::Continue { .. } => String::from("continue"),
            Expression::Grouping { inner } => format!("({})", inner.codegen_inner(indent, false)),
            Expression::Variable { name } => name.lexeme.to_string(),
            Expression::Get { object, name } => format!("{}.{}", object.codegen_inner(indent, needs_indent), name.lexeme),
            Expression::Block { expressions, .. } => block_codegen(expressions, indent, needs_indent, false),
            Expression::ScopedBlock { expressions, .. } => block_codegen(expressions, indent, needs_indent, true),
            Expression::Drop { variable, .. } => format!("drop {}", variable.lexeme),
            Expression::Subscript { subscripted, index, .. } => {
                format!("{}[{}]", subscripted.codegen_inner(indent, needs_indent), index.codegen_inner(indent, false))
            }
            Expression::Ternary { condition, then_expr, else_expr, .. } => {
                format!("({} ? {} : {})", condition.codegen_inner(indent, needs_indent), then_expr.codegen_inner(indent, needs_indent), else_expr.codegen_inner(indent, needs_indent))
            }
            Expression::DoWhile { condition, body, .. } => {
                format!("do {} while {}", body.codegen_inner(indent, false), condition.codegen_inner(indent, false))
            }
            Expression::AlgoDecl { name, object, function, .. } => {
                format!(
                    "@{} {}\n{}{}", 
                    name.lexeme, object.codegen_inner(indent, needs_indent).trim_start_matches('#'), 
                    get_indent(indent), function.codegen_inner(indent, needs_indent)
                )
            }
            Expression::Literal { value, kind, .. } => {
                match kind {
                    LiteralKind::String => format!("\"{}\"", value),
                    LiteralKind::Null => String::from("null"),
                    _ => value.to_string()
                }
            }
            Expression::Assign { target, op, value, type_spec } => {
                if let Some(type_) = type_spec {
                    format!("({}: {} {} {})", target.codegen_inner(indent, true), type_.codegen_inner(indent, false), op.type_.stringify(), value.codegen_inner(indent, false))
                } else {
                    format!("({} {} {})", target.codegen_inner(indent, true), op.type_.stringify(), value.codegen_inner(indent, false))
                }
            }
            Expression::Unary { op, expr, is_prefix } => {
                if *is_prefix {
                    format!("({}{})", op.type_.stringify(), expr.codegen_inner(indent, needs_indent))
                } else {
                    format!("({}{})", expr.codegen_inner(indent, needs_indent), op.type_.stringify())
                }
            }
            Expression::While { condition, body, increment, .. } => {
                if let Some(inc) = increment {
                    if matches!(&**body, Expression::Block { .. }) {
                        format!("for ; {}; {} {}", condition.codegen_inner(indent, false), inc.codegen_inner(indent, false), body.codegen_inner(indent, false))
                    } else {
                        format!("for (; {}; {}) {}", condition.codegen_inner(indent, false), inc.codegen_inner(indent, false), body.codegen_inner(indent, false))
                    }
                } else {
                    if matches!(&**body, Expression::Block { .. }) {
                        format!("while {} {}", condition.codegen_inner(indent, false), body.codegen_inner(indent, false))
                    } else {
                        format!("while ({}) {}", condition.codegen_inner(indent, false), body.codegen_inner(indent, false))
                    }
                }
            }
            Expression::Foreach { variable, iterator, body , ..} => {
                if matches!(&**body, Expression::Block { .. }) {
                    format!("foreach {}: {} {}", variable.lexeme, iterator.codegen_inner(indent, false), body.codegen_inner(indent, false))
                } else {
                    format!("foreach ({}: {}) {}", variable.lexeme, iterator.codegen_inner(indent, false), body.codegen_inner(indent, false))
                }
            }
            Expression::Return { value , ..} => {
                if let Some(ret) = value {
                    format!("return {}", ret.codegen_inner(indent, false))
                } else {
                    String::from("return")
                }
            }
            Expression::Throw { value, .. } => {
                if let Some(throw) = value {
                    format!("throw {}", throw.codegen_inner(indent, false))
                } else {
                    String::from("throw")
                }
            }
            Expression::Try { try_branch, catch_branch, catch_var, .. } => {
                if let Some(catch) = catch_branch {
                    if let Some(var) = catch_var {
                        format!("try {} catch ({}) {}", try_branch.codegen_inner(indent, false), var.lexeme, catch.codegen_inner(indent, false))
                    } else {
                        format!("try {} catch {}", try_branch.codegen_inner(indent, false), catch.codegen_inner(indent, false))
                    }
                } else {
                    format!("try {}", try_branch.codegen_inner(indent, false))
                }
            }
            Expression::List { items, .. } => {
                let mut result = String::new();
                result.push('[');

                for (i, item) in items.iter().enumerate() {
                    result.push_str(&item.codegen_inner(indent, false));

                    if i != items.len() - 1 {
                        result.push_str(", ")
                    }
                }

                result.push(']');
                result
            }
            Expression::Call { callee, args, .. } => {
                let mut result = String::from("(");
                result.push_str(&callee.codegen_inner(indent, needs_indent));
                result.push('(');

                for (i, arg) in args.iter().enumerate() {
                    result.push_str(&arg.codegen_inner(indent, false));

                    if i != args.len() - 1 {
                        result.push_str(", ")
                    }
                }

                result.push(')');
                result.push(')');
                result
            }
            Expression::AnonObject { fields, .. } => {
                let mut result = String::from("#{ ");

                for (i, arg) in fields.iter().enumerate() {
                    result.push_str(&arg.name.lexeme);
                    
                    if let Some(type_) = &arg.type_ {
                        result.push('(');
                        result.push_str(&type_.codegen_inner(indent, false));
                        result.push(')');
                    }

                    result.push_str(": ");
                    result.push_str(&arg.expr.codegen_inner(indent, false));

                    if i != fields.len() - 1 {
                        result.push_str(", ")
                    }
                }

                result.push_str(" }");
                result
            }
            Expression::If { condition, then_branch, else_branch, .. } => {
                if matches!(&**then_branch, Expression::Block { .. }) {
                    if let Some(else_) = else_branch {
                        format!("if {} {} else {}", condition.codegen_inner(indent, false), then_branch.codegen_inner(indent, false), else_.codegen_inner(indent, false))
                    } else {
                        format!("if {} {}", condition.codegen_inner(indent, false), then_branch.codegen_inner(indent, false))
                    }
                } else {
                    if let Some(else_) = else_branch {
                        format!("if ({}) {}; else {}", condition.codegen_inner(indent, false), then_branch.codegen_inner(indent, false), else_.codegen_inner(indent, false))
                    } else {
                        format!("if ({}) {}", condition.codegen_inner(indent, false), then_branch.codegen_inner(indent, false))
                    }
                }
            }
            Expression::Function { name, params, return_type, body } => {
                let mut result = String::from("fn ");
                result.push_str(&name.lexeme);
                result.push('(');

                for (i, arg) in params.iter().enumerate() {
                    result.push_str(&arg.name.lexeme);

                    if let Some(type_) = &arg.expr {
                        result.push_str(": ");
                        result.push_str(&type_.codegen_inner(indent, false));
                    }

                    if i != params.len() - 1 {
                        result.push_str(", ")
                    }
                }

                result.push_str(") ");
                result.push_str(&return_type.codegen_inner(indent, false));
                result.push(' ');
                result.push_str(&block_codegen(body, indent, false, false));
                result
            }
            Expression::Switch { expr, cases , ..} => {
                let mut result = String::from("switch ");
                result.push_str(&expr.codegen_inner(indent, false));
                result.push_str("{\n");

                for block in cases {
                    result.push_str(&get_indent(indent + 1));

                    if let Some(inner_cases) = &block.cases {
                        for (i, case) in inner_cases.iter().enumerate() {
                            result.push_str(&case.codegen_inner(indent, false));

                            if i != inner_cases.len() - 1 {
                                result.push_str(" | ");
                            }
                        }
                    } else {
                        result.push_str("default");
                    }

                    result.push(' ');
                    result.push_str(&block_codegen(&block.code, indent + 1, false, false));
                }

                result.push_str(&get_indent(indent));
                result.push('}');
                result
            }
            Expression::Binary { left, op, right } | 
            Expression::Cmp { left, op, right } | 
            Expression::Logic { left, op, right } => {
                format!("({} {} {})", left.codegen_inner(indent, needs_indent), op.type_.stringify(), right.codegen_inner(indent, needs_indent))
            }
        }
    }

    pub fn codegen(&self) -> String {
        self.codegen_inner(0, true)
    }

    pub fn get_inner(&self) -> Expression {
        match self {
            Expression::Grouping { inner } => inner.get_inner(),
            _ => self.clone()
        }
    }

    pub fn equals(&self, other: &Expression) -> bool {
        if std::mem::discriminant(self) != std::mem::discriminant(other) && 
           !(matches!(self, Expression::Grouping { .. }) || matches!(other, Expression::Grouping { .. })) 
        {
            return false;
        }

        let other = other.get_inner();

        match self {
            Expression::Grouping { inner: self_inner } => self_inner.equals(&other),
            Expression::Break { value: self_value, .. } |
            Expression::Continue { value: self_value, .. } |
            Expression::Return { value: self_value, .. } |
            Expression::Throw { value: self_value, .. } => {
                if let Expression::Break { value: other_value, .. } |
                       Expression::Continue { value: other_value, .. } |
                       Expression::Return { value: other_value, .. } |
                       Expression::Throw { value: other_value, .. } = other 
                {
                    if self_value.is_some() ^ other_value.is_some() {
                        false
                    } else if self_value.is_none() {
                        true
                    } else {
                        self_value.as_ref().unwrap().equals(other_value.as_ref().unwrap())
                    }
                } else {
                    unreachable!()
                }
            }
            Expression::Variable { name: self_name } |
            Expression::Drop { variable: self_name, .. } => {
                if let Expression::Variable { name: other_name } |
                       Expression::Drop { variable: other_name, .. } = other {
                    self_name.lexeme.as_ref() == other_name.lexeme.as_ref()
                } else {
                    unreachable!()
                }
            }
            Expression::Binary { left: self_left, op: self_op, right: self_right } | 
            Expression::Cmp { left: self_left, op: self_op, right: self_right } | 
            Expression::Logic { left: self_left, op: self_op, right: self_right } |
            Expression::Assign { target: self_left, op: self_op, value: self_right, .. } => {
                if let Expression::Binary { left: other_left, op: other_op, right: other_right } |
                       Expression::Cmp { left: other_left, op: other_op, right: other_right } |
                       Expression::Logic { left: other_left, op: other_op, right: other_right } |
                       Expression::Assign { target: other_left, op: other_op, value: other_right, .. } = other 
                {
                    self_op.type_ == other_op.type_ && self_left.equals(&other_left) && self_right.equals(&other_right)   
                } else {
                    unreachable!()
                }
            }
            Expression::Literal { value: self_value, kind: self_kind, .. } => {
                if let Expression::Literal { value: other_value, kind: other_kind, .. } = other {
                    *self_kind == other_kind && self_value.as_ref() == other_value.as_ref()
                } else {
                    unreachable!()
                }
            }
            Expression::Unary { op: self_op, expr: self_expr, is_prefix: self_is_prefix } => {
                if let Expression::Unary { op: other_op, expr: other_expr, is_prefix: other_is_prefix } = other {
                    *self_is_prefix == other_is_prefix && self_op.type_ == other_op.type_ && self_expr.equals(&other_expr)
                } else {
                    unreachable!()
                }
            }
            Expression::Call { callee: self_callee, args: self_args, .. } => {
                if let Expression::Call { callee: other_callee, args: other_args, .. } = other {
                    if self_args.len() != other_args.len() {
                        return false;
                    }

                    if !self_callee.equals(&other_callee) {
                        return false;
                    }

                    for i in 0 .. self_args.len() {
                        if !self_args[i].equals(&other_args[i]) {
                            return false;
                        }
                    }

                    true
                } else {
                    unreachable!()
                }
            }
            Expression::Ternary { condition: self_condition, then_expr: self_then, else_expr: self_else, .. } => {
                if let Expression::Ternary { condition: other_condition, then_expr: other_then, else_expr: other_else, .. } = other {
                    self_condition.equals(&other_condition) && self_then.equals(&other_then) && self_else.equals(&other_else)
                } else {
                    unreachable!()
                }
            }
            Expression::Subscript { subscripted: self_st, index: self_nd, .. } |
            Expression::DoWhile { condition: self_st, body: self_nd, .. } => {
                if let Expression::Subscript { subscripted: other_st, index: other_nd, .. } |
                       Expression::DoWhile { condition: other_st, body: other_nd, .. } = other 
                {
                    self_st.equals(&other_st) && self_nd.equals(&other_nd)
                } else {
                    unreachable!()
                }
            }
            Expression::Get { object: self_obj, name: self_name } => {
                if let Expression::Get { object: other_obj, name: other_name } = other {
                    self_name.lexeme.as_ref() == other_name.lexeme.as_ref() && self_obj.equals(&other_obj)
                } else {
                    unreachable!()
                }
            }
            Expression::List { items: self_items, .. } |
            Expression::Block { expressions: self_items, .. } |
            Expression::ScopedBlock { expressions: self_items, .. } => {
                if let Expression::List { items: other_items, .. } |
                       Expression::Block { expressions: other_items, .. } |
                       Expression::ScopedBlock { expressions: other_items, .. } = other 
                {
                    if self_items.len() != other_items.len() {
                        return false;
                    }

                    for i in 0 .. self_items.len() {
                        if !self_items[i].equals(&other_items[i]) {
                            return false;
                        }
                    }

                    true
                } else {
                    unreachable!()
                }
            }
            Expression::If { condition: self_st, then_branch: self_nd, else_branch: self_opt, .. } |
            Expression::While { condition: self_st, body: self_nd, increment: self_opt, .. } => {
                if let Expression::If { condition: other_st, then_branch: other_nd, else_branch: other_opt, .. } |
                       Expression::While { condition: other_st, body: other_nd, increment: other_opt, .. } = other 
                {
                    if self_opt.is_some() ^ other_opt.is_some() {
                        false
                    } else {
                        if !self_st.equals(&other_st) || !self_nd.equals(&other_nd) {
                            return false;
                        }

                        if self_opt.is_none() {
                            true
                        } else {
                            self_opt.as_ref().unwrap().equals(other_opt.as_ref().unwrap())
                        }
                    }
                } else {
                    unreachable!()
                }
            }
            Expression::Try { try_branch: self_try, catch_branch: self_catch, .. } => {
                if let Expression::Try { try_branch: other_try, catch_branch: other_catch, .. } = other {
                    if self_catch.is_some() ^ other_catch.is_some() {
                        false
                    } else {
                        if !self_try.equals(&other_try) {
                            return false;
                        }

                        if self_catch.is_none() {
                            true
                        } else {
                            self_catch.as_ref().unwrap().equals(other_catch.as_ref().unwrap())
                        }
                    }
                } else {
                    unreachable!()
                }
            }
            _ => false // TODO improve recognition
        }
    }
}