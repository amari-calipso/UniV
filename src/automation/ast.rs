use std::rc::Rc;

use alanglib::ast::{SourcePos, WithPosition};

use super::tokens::Token;

pub enum Statement {
    Set { type_: Token, value: Expression },
    Push { value: Expression },
    Pop,
    Reset { type_: Token },
    Define { name: Token, value: Expression },
    RunShuffle { kw: Token, name: Expression, timestamp: bool },
    Describe { value: Expression },
    Timestamp { kw: Token, value: Expression },
    RunAllSorts { kw: Token, categories: Vec<RunAllSortsCategory> },
    RunAllShuffles { kw: Token, statements: Vec<Statement> },
    RunDistribution { 
        kw: Token, name: Expression, length: Option<Expression>, 
        unique: Option<Expression>, timestamp: bool 
    },
    RunSort { 
        kw: Token, name: Expression, category: Option<Expression>, 
        length: Option<Expression>, speed: Option<Expression>, 
        speed_scale: Option<Expression>, max_length: Option<Expression>,
        timestamp: bool
    }
}

pub struct RunAllSortsCategory {
    pub name: Expression,
    pub statements: Vec<Statement>
}

pub enum Expression {
    Identifier(Token),
    Float(Token),
    Int(Token),
    String(Token)
}

impl WithPosition for Expression {
    fn get_pos(&self) -> SourcePos {
        match self {
            Expression::Identifier(token) |
            Expression::Float(token) |
            Expression::Int(token) |
            Expression::String(token) => {
                SourcePos::new(
                    Rc::clone(&token.source), 
                    Rc::clone(&token.filename), 
                    token.pos, token.end, token.line
                )
            }
        }
    }
}