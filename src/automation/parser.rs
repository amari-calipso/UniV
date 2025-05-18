use crate::token_error;

use super::{ast::{Expression, RunAllSortsCategory, Statement}, tokens::{Token, TokenType}};

pub struct Parser {
    tokens: Vec<Token>,
    curr: usize,
    pub errors: Vec<String>
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, curr: 0, errors: Vec::new() }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.curr]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.curr - 1]
    }

    fn is_at_end(&self) -> bool {
        self.peek().type_ == TokenType::EOF
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.curr += 1;
        }

        self.previous()
    }

    fn check(&self, type_: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().type_ == type_
    }

    fn match_(&mut self, types: &[TokenType]) -> bool {
        for type_ in types {
            if self.check(*type_) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn syncronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            match self.peek().type_ {
                TokenType::Set | TokenType::Push | TokenType::Pop |
                TokenType::Reset | TokenType::Define | TokenType::Run |
                TokenType::Shuffle | TokenType::Distribution => return,
                _ => ()
            }

            self.advance();
        }
    }

    fn consume(&mut self, type_: TokenType, msg: &str) -> Option<&Token> {
        if self.check(type_) {
            Some(self.advance())
        } else {
            let tok = self.peek();
            token_error!(self, tok, msg);
            None
        }
    }

    fn expression(&mut self) -> Option<Expression> {
        if self.match_(&[TokenType::Identifier]) {
            return Some(Expression::Identifier(self.previous().clone()));
        }

        if self.match_(&[TokenType::String]) {
            return Some(Expression::String(self.previous().clone()));
        }

        if self.match_(&[TokenType::Int]) {
            return Some(Expression::Int(self.previous().clone()));
        }

        if self.match_(&[TokenType::Float]) {
            return Some(Expression::Float(self.previous().clone()));
        }

        let last = self.peek();
        token_error!(self, last, "Expecting expression");
        None
    }

    fn set_statement(&mut self) -> Option<Statement> {
        let type_ = {
            if self.match_(&[TokenType::Visual]) {
                self.previous().clone()
            } else {
                self.consume(TokenType::Speed, "Expecting 'visual' or 'speed' after 'set'")?.clone()
            }
        };

        let value = self.expression()?;
        Some(Statement::Set { type_, value })
    }

    fn push_statement(&mut self) -> Option<Statement> {
        let value = self.expression()?;
        Some(Statement::Push { value })
    }

    fn reset_statement(&mut self) -> Option<Statement> {
        let type_ = {
            if self.match_(&[TokenType::Speed]) {
                self.previous().clone()
            } else {
                self.consume(TokenType::Queue, "Expecting 'speed' or 'queue' after 'reset'")?.clone()
            }
        };

        Some(Statement::Reset { type_ })
    }

    fn define_statement(&mut self) -> Option<Statement> {
        let name = self.consume(TokenType::Identifier, "Expecting variable name after 'define'")?.clone();
        let value = self.expression()?;
        Some(Statement::Define { name, value })
    }

    fn run_shuffle(&mut self) -> Option<Statement> {
        let kw = self.previous().clone();
        let name = self.expression()?;
        Some(Statement::RunShuffle { kw, name })
    }

    fn run_distribution(&mut self) -> Option<Statement> {
        let kw = self.previous().clone();
        let name = self.expression()?;

        let mut length = None;
        let mut unique = None;
        if self.match_(&[TokenType::With]) {
            for _ in 0 .. 2 {
                if self.match_(&[TokenType::Length]) {
                    length = Some(self.expression()?);
                } else {
                    unique = Some(self.expression()?);
                    self.consume(TokenType::Unique, "Expecting 'unique' after unique amount")?;
                }

                if !self.match_(&[TokenType::And]) {
                    break;
                }
            }
        }

        Some(Statement::RunDistribution { kw, name, length, unique })
    }

    fn run_sort(&mut self) -> Option<Statement> {
        let kw = self.previous().clone();
        let name = self.expression()?;

        let category = {
            if self.match_(&[TokenType::In]) {
                Some(self.expression()?)
            } else {
                None
            }
        };

        let mut length = None;
        let mut speed = None;
        let mut speed_scale = None;
        if self.match_(&[TokenType::With]) {
            for _ in 0 .. 2 {
                if self.match_(&[TokenType::Length]) {
                    length = Some(self.expression()?);
                } else if self.match_(&[TokenType::Speed]) {
                    speed = Some(self.expression()?);
                    
                    if self.match_(&[TokenType::Scaled]) {
                        self.consume(TokenType::By, "Expecting 'by' after 'scaled'")?;
                        speed_scale = Some(self.expression()?);
                    }
                }

                if !self.match_(&[TokenType::And]) {
                    break;
                }
            }
        }
        
        Some(Statement::RunSort { kw, name, category, length, speed, speed_scale })
    }

    fn block(&mut self) -> Option<Vec<Statement>> {
        self.consume(TokenType::LeftBrace, "Expecting '{' before block")?;

        let mut statements = Vec::new();
        while (!self.check(TokenType::RightBrace)) && (!self.is_at_end()) {
            statements.push(self.statement()?);
        }

        self.consume(TokenType::RightBrace, "Expecting '}' after block")?;
        Some(statements)
    }

    fn run_all_sorts(&mut self) -> Option<Statement> {
        let kw = self.previous().clone();

        self.consume(TokenType::LeftBrace, "Expecting '{' after 'run all sorts'")?;

        let mut categories = Vec::new();
        while (!self.check(TokenType::RightBrace)) && (!self.is_at_end()) {
            let name = self.expression()?;
            let statements = self.block()?;
            categories.push(RunAllSortsCategory { name, statements });
        }

        self.consume(TokenType::RightBrace, "Expecting '}' after block")?;
        Some(Statement::RunAllSorts { kw, categories })
    }

    fn run_all_shuffles(&mut self) -> Option<Statement> {
        let kw = self.previous().clone();
        let statements = self.block()?;
        Some(Statement::RunAllShuffles { kw, statements })
    }

    fn run_statement(&mut self) -> Option<Statement> {
        if self.match_(&[TokenType::Shuffle]) {
            return self.run_shuffle();
        }

        if self.match_(&[TokenType::Distribution]) {
            return self.run_distribution();
        }

        if self.match_(&[TokenType::Sort]) {
            return self.run_sort();
        }

        if self.match_(&[TokenType::All]) {
            if self.match_(&[TokenType::Sorts]) {
                return self.run_all_sorts();
            } else if self.match_(&[TokenType::Shuffles]) {
                return self.run_all_shuffles();
            } else {
                let last = self.peek();
                token_error!(self, last, "Expecting 'sorts' or 'shuffles' after 'run all'");
                return None;
            }
        }

        let last = self.peek();
        token_error!(self, last, "Expecting run statement");
        None
    }

    fn describe_statement(&mut self) -> Option<Statement> {
        let value = self.expression()?;
        Some(Statement::Describe { value })
    }
    
    fn statement(&mut self) -> Option<Statement> {
        if self.match_(&[TokenType::Set]) {
            return self.set_statement();
        }

        if self.match_(&[TokenType::Run]) {
            return self.run_statement();
        }

        if self.match_(&[TokenType::Push]) {
            return self.push_statement();
        }

        if self.match_(&[TokenType::Pop]) {
            return Some(Statement::Pop);
        }

        if self.match_(&[TokenType::Reset]) {
            return self.reset_statement();
        }

        if self.match_(&[TokenType::Define]) {
            return self.define_statement();
        }

        if self.match_(&[TokenType::Describe]) {
            return self.describe_statement();
        }

        let last = self.peek();
        token_error!(self, last, "Expecting statement");
        None
    }

    pub fn parse(&mut self) -> Vec<Statement> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            if let Some(expr) = self.statement() {
                statements.push(expr);
            } else {
                self.syncronize();
            }
        }

        statements
    }
}