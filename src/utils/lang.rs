use std::rc::Rc;

use crate::unil::{ast::{Expression, LiteralKind}, scanner::KEYWORDS, tokens::Token};

const INDENT_SIZE: usize = 4;

#[cfg(not(feature = "lite"))]
pub trait Transform {
    type Transformer;
    fn to_ast(&self, t: &mut Self::Transformer) -> Expression;
}

pub fn traceback_part(source: &str, filename: &str, pos: usize, len: usize, line: usize) -> String {
    let mut output = String::new();
    output.push_str(format!("  File '{}': line {}, pos {}\n", filename, line + 1, pos).as_ref());

    if let Some(source_line) = source.lines().nth(line) {
        let orig = source_line.trim_end();
        let trimmed = orig.trim_start();
        output.push_str(format!("    {}\n", trimmed).as_ref());
        output.push_str(format!("    {}{}\n", " ".repeat(pos - (orig.len() - trimmed.len())), "^".repeat(len)).as_ref());
    }

    output
}

#[macro_export]
macro_rules! error {
    ($slf: expr, $e: expr, $msg: expr) => {
        {
            $slf.errors.push(alanglib::report::error(&$e, $msg));
        }
    };
}

#[allow(dead_code)]
pub fn make_null() -> Expression {
    Expression::Literal {
        value: Rc::from(""),
        tok: Token::empty(),
        kind: LiteralKind::Null
    }
}

pub struct BaseASTTransformer {
    pub source:   String,
    pub filename: Rc<str>,

    pub curr_name:   String,
    pub tmp_var_cnt: usize,
    pub depth:       usize,

    pub errors: Vec<String>
}

impl BaseASTTransformer {
    pub fn new(source: String, filename: Rc<str>) -> Self {
        BaseASTTransformer {
            source, filename,
            curr_name: String::from(""),
            tmp_var_cnt: 0,
            depth: 0,
            errors: Vec::new()
        }
    }

    pub fn tmp_var(&mut self) -> Rc<str> {
        let tmp = format!("__tmp_{}", self.tmp_var_cnt).into();
        self.tmp_var_cnt += 1;
        tmp
    }

    pub fn get_fn_name(&self, name: &str) -> Rc<str> {
        let name = self.get_var_name(name);

        if self.curr_name == "" {
            name
        } else {
            format!("{}__{}", self.curr_name, name).into()
        }
    }

    pub fn get_var_name(&self, name: &str) -> Rc<str> {        
        if KEYWORDS.contains_key(name) {
            format!("__reserved_{}", name).into()
        } else {
            Rc::from(name)
        }
    }

    pub fn inc_depth(&mut self) {
        self.depth += 1;
    }

    pub fn dec_depth(&mut self) {
        self.depth -= 1;
    }
}

pub fn get_token_from_variable(expr: Expression) -> Token {
    if let Expression::Variable { name } = expr {
        name
    } else {
        panic!("Expression was not Variable")
    }
}

pub fn get_vec_of_expr_from_block(expr: Expression) -> Vec<Expression> {
    if let Expression::Block { expressions, .. } = expr {
        expressions
    } else {
        panic!("Expression was not Block")
    }
}

#[allow(dead_code)]
pub fn push_indent(str: &mut String, level: usize) {
    for _ in 0 .. level * INDENT_SIZE {
        str.push(' ');
    }
}