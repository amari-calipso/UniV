use std::rc::Rc;

use ast::Expression;
use parser::Parser;
use scanner::Scanner;

pub mod tokens;
pub mod ast;

pub mod scanner;
mod parser;

pub mod swap_recognition;

pub fn parse(source: String, filename: Rc<str>) -> Result<Vec<Expression>, Vec<String>> {
    let mut scanner = Scanner::new(&source, filename);
    scanner.scan_tokens();

    if !scanner.errors.is_empty() {
        return Err(scanner.errors);
    }

    let mut parser = Parser::new(scanner.tokens);
    let program = parser.parse();

    if !parser.errors.is_empty() {
        return Err(parser.errors);
    }

    Ok(program)
}