use std::rc::Rc;

use crate::utils::lang::AstPos;

use super::tokens::Token;

pub enum Statement {
    Set { type_: Token, value: Expression },
    Push { value: Expression },
    Pop,
    Reset { type_: Token },
    Define { name: Token, value: Expression },
    RunShuffle { kw: Token, name: Expression },
    Describe { value: Expression },
    RunAllSorts { kw: Token, categories: Vec<RunAllSortsCategory> },
    RunAllShuffles { kw: Token, statements: Vec<Statement> },
    RunDistribution { 
        kw: Token, name: Expression, length: Option<Expression>, 
        unique: Option<Expression> 
    },
    RunSort { 
        kw: Token, name: Expression, category: Option<Expression>, 
        length: Option<Expression>, speed: Option<Expression>, 
        speed_scale: Option<Expression>, max_length: Option<Expression>
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

impl Expression {
    pub fn get_pos(&self) -> AstPos {
        match self {
            Expression::Identifier(token) |
            Expression::Float(token) |
            Expression::Int(token) |
            Expression::String(token) => {
                AstPos::new(
                    Rc::clone(&token.source), 
                    Rc::clone(&token.filename), 
                    token.pos, token.end, token.line
                )
            }
        }
    }
}