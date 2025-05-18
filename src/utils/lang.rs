use std::{cmp::max, rc::Rc};

use bincode::{Decode, Encode};

use crate::unil::{ast::{Expression, LiteralKind}, scanner::KEYWORDS, tokens::Token};

#[cfg(not(feature = "lite"))]
pub trait Transform {
    type Transformer;
    fn to_ast(&self, t: &mut Self::Transformer) -> Expression;
}

pub fn report(
    source: &str, msg: &str, type_: &str, filename: &str,
    pos: usize, len: usize, line: usize
) -> String {
    let lines = source.lines().collect::<Vec<&str>>();

    let mut output = format!("{} ({}: line {}, pos {}): {}\n", type_, filename, line + 1, pos, msg);

    let iter_range = {
        if lines.len() < 5 {
            0..lines.len()
        } else {
            if line <= 2 {
                0..5
            } else if line >= lines.len() - 3 {
                (lines.len() - 5)..lines.len()
            } else {
                (line - 2)..(line + 3)
            }
        }
    };

    let linelen = max((iter_range.end as f64).log10().ceil() as usize, 1);

    for l in iter_range {
        output.push_str(format!("{:linelen$} | {}\n", l + 1, lines[l].trim_end()).as_ref());

        if l == line {
            output.push_str(format!("{} | {}{}\n", " ".repeat(linelen), " ".repeat(pos), "^".repeat(len)).as_ref());
        }
    }

    output
}

#[inline]
pub fn error(source: &str, msg: &str, filename: &str, pos: usize, len: usize, line: usize) -> String {
    report(source, msg, "error", filename, pos, len, line)
}

#[allow(dead_code)]
#[inline]
pub fn warning(source: &str, msg: &str, filename: &str, pos: usize, len: usize, line: usize) -> String {
    report(source, msg, "warning", filename, pos, len, line)
}

#[allow(dead_code)]
#[inline]
pub fn note(source: &str, msg: &str, filename: &str, pos: usize, len: usize, line: usize) -> String {
    report(source, msg, "note", filename, pos, len, line)
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

#[derive(Clone, Encode, Decode)]
pub struct AstPos {
    pub source: Rc<str>,
    pub filename: Rc<str>,
    pub start: usize,
    pub end: usize,
    pub line: usize
}

impl std::fmt::Debug for AstPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AstPos").field("filename", &self.filename).field("start", &self.start).field("end", &self.end).field("line", &self.line).finish()
    }
}

impl AstPos {
    pub fn new(source: Rc<str>, filename: Rc<str>, start: usize, end: usize, line: usize) -> Self {
        AstPos { source, filename, start, end, line }
    }
}

#[macro_export]
macro_rules! token_error {
    ($slf: expr, $token: expr, $msg: expr) => {
        {
            $slf.errors.push($crate::utils::lang::error(&$token.source, $msg, &$token.filename, $token.pos, $token.end.saturating_sub($token.pos), $token.line));
        }
    };
}

#[macro_export]
macro_rules! token_warning {
    ($token: expr, $msg: expr) => {
        println!("{}", $crate::utils::lang::warning(&$token.source, $msg, &$token.filename, $token.pos, $token.end.saturating_sub($token.pos), $token.line));
    };
}

#[macro_export]
macro_rules! token_note {
    ($token: expr, $msg: expr) => {
        println!("{}", $crate::utils::lang::note(&$token.source, $msg, &$token.filename, $token.pos, $token.end.saturating_sub($token.pos), $token.line));
    };
}

#[macro_export]
macro_rules! ast_error {
    ($slf: expr, $e: expr, $msg: expr) => {
        {
            let pos: $crate::utils::lang::AstPos = $e.get_pos();
            $slf.errors.push($crate::utils::lang::error(&pos.source, $msg, &pos.filename, pos.start, pos.end.saturating_sub(pos.start), pos.line));
        }
    };
}

#[macro_export]
macro_rules! ast_warning {
    ($s: expr, $msg: expr) => {
        {
            let pos: $crate::utils::lang::AstPos = $s.get_pos();
            println!("{}", $crate::utils::lang::warning(&pos.source, $msg, &pos.filename, pos.start, pos.end.saturating_sub(pos.start), pos.line));
        }
    };
}

#[macro_export]
macro_rules! ast_note {
    ($e: expr, $msg: expr) => {
        {
            let pos: $crate::utils::lang::AstPos = $e.get_pos();
            println!("{}", $crate::utils::lang::note(&pos.source, $msg, &pos.filename, pos.start, pos.end.saturating_sub(pos.start), pos.line));
        }
    };
}

pub fn is_beginning_digit(c: char) -> bool {
    c >= '1' && c <= '9'
}

pub fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

pub fn is_bin_digit(c: char) -> bool {
    c == '0' || c == '1'
}

pub fn is_oct_digit(c: char) -> bool {
    c >= '0' && c <= '7'
}

pub fn is_hex_digit(c: char) -> bool {
    is_digit(c) || (c >= 'A' && c <= 'F') || (c >= 'a' && c <= 'f')
}

pub fn is_alpha(c: char) -> bool {
    (c >= 'a' && c <= 'z') ||
    (c >= 'A' && c <= 'Z') ||
    c == '_'
}

pub fn is_alphanumeric(c: char) -> bool {
    is_alpha(c) || is_digit(c)
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