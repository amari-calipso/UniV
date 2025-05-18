use std::{collections::HashMap, rc::Rc};

use lazy_static::lazy_static;

use crate::utils::{lang::{error, is_alpha, is_alphanumeric, is_beginning_digit, is_bin_digit, is_digit, is_hex_digit, is_oct_digit}, substring};

use super::tokens::{Token, TokenType};

lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, TokenType> = {
        let mut keywords = HashMap::new();
        keywords.insert("in",           TokenType::In);
        keywords.insert("by",           TokenType::By);
        keywords.insert("all",          TokenType::All);
        keywords.insert("and",          TokenType::And);
        keywords.insert("pop",          TokenType::Pop);
        keywords.insert("run",          TokenType::Run);
        keywords.insert("set",          TokenType::Set);
        keywords.insert("push",         TokenType::Push);
        keywords.insert("sort",         TokenType::Sort);
        keywords.insert("with",         TokenType::With);
        keywords.insert("queue",        TokenType::Queue);
        keywords.insert("reset",        TokenType::Reset);
        keywords.insert("sorts",        TokenType::Sorts);
        keywords.insert("speed",        TokenType::Speed);
        keywords.insert("define",       TokenType::Define);
        keywords.insert("length",       TokenType::Length);
        keywords.insert("scaled",       TokenType::Scaled);
        keywords.insert("unique",       TokenType::Unique);
        keywords.insert("visual",       TokenType::Visual);
        keywords.insert("shuffle",      TokenType::Shuffle);
        keywords.insert("describe",     TokenType::Describe);
        keywords.insert("shuffles",     TokenType::Shuffles);        
        keywords.insert("distribution", TokenType::Distribution);
        keywords
    };
}

pub struct Scanner {
    source: Rc<str>,
    filename: Rc<str>,
    pub tokens: Vec<Token>,
    start_positions: Vec<usize>,

    start: usize,
    curr:  usize,
    line:  usize,

    pub errors: Vec<String>,
}

impl Scanner {
    pub fn new(source: Rc<str>, filename: Rc<str>) -> Self {
        Scanner {
            source, filename, tokens: Vec::new(), start_positions: Vec::new(),
            start: 0, curr: 0, line: 0, errors: Vec::new()
        }
    }

    fn is_at_end(&self) -> bool {
        self.curr >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.curr).unwrap();
        self.curr += 1;
        c
    }

    fn add_token(&mut self, type_: TokenType) {
        self.tokens.push(Token::new(
            Rc::from(self.source.as_ref()),
            Rc::clone(&self.filename), type_,
            substring(&self.source.to_string(), self.start, self.curr).into(),
            self.start.saturating_sub(self.start_positions[self.line]),
            self.curr.saturating_sub(self.start_positions[self.line]), self.line
        ));
    }

    fn match_(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.source.chars().nth(self.curr).unwrap() != expected {
            return false;
        }

        self.curr += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().nth(self.curr).unwrap()
        }
    }

    fn error(&mut self, msg: &str) {
        self.errors.push(error(
            &self.source, msg,
            &self.filename,
            self.start.saturating_sub(self.start_positions[self.line]),
            self.curr.saturating_sub(self.start), self.line
        ));
    }

    fn string_literal(&mut self, ch: char) {
        let mut back_slash = false;
        loop {
            let c = self.peek();
            if self.is_at_end() || (c == ch && !back_slash) {
                break;
            }

            let old_backslash = back_slash;
            back_slash = false;

            match c {
                '\n' => self.line += 1,
                '\\' => {
                    if !old_backslash {
                        back_slash = true;
                    }
                }
                _ => (),
            }

            self.advance();
        }

        if self.is_at_end() {
            self.error("Unterminated string");
            return;
        }

        self.advance();

        self.start += 1;
        self.curr -= 1;

        self.add_token(TokenType::String);

        self.start -= 1;
        self.curr += 1;
    }

    fn number(&mut self, scan_start: bool) {
        if scan_start {
            while is_digit(self.peek()) {
                self.advance();
            }
        }

        let c = self.peek();
        if c == '.' {
            self.advance();

                if is_digit(self.peek()) {
                    loop {
                        self.advance();

                        if !is_digit(self.peek()) {
                            break;
                        }
                    }
            } else {
                self.error("Expecting digits after decimal point");
            }

            self.add_token(TokenType::Float);
        } else {
            self.add_token(TokenType::Int);
        }
    }

    fn add_int_token_with_base_conversion(&mut self, base: u32) {
        self.tokens.push(Token::new(
            Rc::from(self.source.as_ref()),
            Rc::clone(&self.filename), TokenType::Int,
            format!(
                "{}",
                i64::from_str_radix(
                    substring(&self.source.to_string(), self.start + 2, self.curr).as_ref(),
                    base
                ).expect("Scanner failed scanning alt base number")
            ).into(),
            self.start.saturating_sub(self.start_positions[self.line]),
            self.curr.saturating_sub(self.start_positions[self.line]), self.line
        ));
    }

    fn alt_base_number(&mut self) {
        let c = self.peek();
        match c {
            'b' => {
                self.advance();

                while is_bin_digit(self.peek()) {
                    self.advance();
                }

                self.add_int_token_with_base_conversion(2);
            }
            'o' => {
                self.advance();

                while is_oct_digit(self.peek()) {
                    self.advance();
                }

                self.add_int_token_with_base_conversion(8);
            }
            'x' => {
                self.advance();

                while is_hex_digit(self.peek()) {
                    self.advance();
                }

                self.add_int_token_with_base_conversion(16);
            }
            _ => {
                if is_digit(c) {
                    loop {
                        self.advance();

                        if !is_digit(self.peek()) {
                            break;
                        }
                    }

                    self.number(false);
                    self.error("Base 10 numbers cannot be zero-padded");
                } else {
                    self.number(false);
                }
            }
        }
    }

    fn identifier(&mut self) {
        while is_alphanumeric(self.peek()) {
            self.advance();
        }

        let kind = KEYWORDS.get(
            substring(&self.source.to_string(), self.start, self.curr)
                .to_lowercase().as_str()
        );

        if let Some(type_) = kind {
            self.add_token(*type_);
        } else {
            self.add_token(TokenType::Identifier);
        }
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            
            '"'  => self.string_literal('"'),
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,

            '/' => {
                if self.match_('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.error("Unexpected character");
                }
            }

            '0' => self.alt_base_number(),

            _ => {
                if is_beginning_digit(c) {
                    self.number(true);
                } else if is_alpha(c) {
                    self.identifier();
                } else {
                    self.error("Unexpected character");
                }
            }
        }
    }

    fn get_start_positions(&mut self) {
        self.start_positions.push(0);
        for (i, c) in self.source.chars().enumerate() {
            if c == '\n' {
                self.start_positions.push(i + 1);
            }
        }
    }

    pub fn scan_tokens(&mut self) {
        self.get_start_positions();

        while !self.is_at_end() {
            self.start = self.curr;
            self.scan_token();
        }

        self.tokens.push(Token::new(
            Rc::from(self.source.as_ref()),
            Rc::clone(&self.filename),
            TokenType::EOF, Rc::from(""),
            0, 1, self.line
        ));
    }
}