use std::{collections::HashMap, rc::Rc};

use alanglib::{ast::SourcePos, report::error_pos, scanner::{is_str_alpha, is_str_alphanumeric, is_str_beginning_digit, is_str_bin_digit, is_str_digit, is_str_hex_digit, is_str_oct_digit, substring}};
use lazy_static::lazy_static;
use unicode_segmentation::UnicodeSegmentation;

use super::tokens::{Token, TokenType};

lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, TokenType> = {
        let mut keywords = HashMap::new();
        keywords.insert("in",           TokenType::In);
        keywords.insert("by",           TokenType::By);
        keywords.insert("all",          TokenType::All);
        keywords.insert("and",          TokenType::And);
        keywords.insert("max",          TokenType::Max);
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
        keywords.insert("maximum",      TokenType::Max);
        keywords.insert("shuffle",      TokenType::Shuffle);
        keywords.insert("describe",     TokenType::Describe);
        keywords.insert("shuffles",     TokenType::Shuffles);
        keywords.insert("timestamp",    TokenType::Timestamp);      
        keywords.insert("distribution", TokenType::Distribution);
        keywords
    };
}

pub struct Scanner {
    source: Rc<str>,
    filename: Rc<str>,
    pub tokens: Vec<Token>,

    start:      usize,
    curr:       usize,
    line:       usize,
    start_line: usize,
    max_pos:    usize,

    pub errors: Vec<String>,
}

impl Scanner {
    pub fn new(source: Rc<str>, filename: Rc<str>) -> Self {
        Scanner {
            max_pos: source.graphemes(true).count(),
            source, 
            filename, 
            tokens: Vec::new(), 
            start: 0, 
            curr: 0, 
            line: 0,
            start_line: 1,
            errors: Vec::new()
        }
    }

    fn is_at_end(&self) -> bool {
        self.curr >= self.max_pos
    }

    fn advance(&mut self) -> &str {
        let c = self.source.graphemes(true).nth(self.curr).unwrap();
        self.curr += 1;
        c
    }

    fn add_token(&mut self, type_: TokenType) {
        self.tokens.push(Token::new(
            Rc::from(self.source.as_ref()),
            Rc::clone(&self.filename), type_,
            substring(&self.source.to_string(), self.start, self.curr).into(),
            self.start_line - 1,
            self.start_line - 1 + self.curr.saturating_sub(self.start), 
            self.line
        ));
    }

    fn match_(&mut self, expected: &str) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.source.graphemes(true).nth(self.curr).unwrap() != expected {
            return false;
        }

        self.curr += 1;
        true
    }

    fn peek(&self) -> &str {
        if self.is_at_end() {
            "\0"
        } else {
            self.source.graphemes(true).nth(self.curr).unwrap()
        }
    }

    fn newline(&mut self) {
        self.line += 1;
        self.start_line = 0;
    }

    fn error(&mut self, msg: &str) {
        self.errors.push(error_pos(
            &SourcePos::new(
                Rc::clone(&self.source),
                Rc::clone(&self.filename),
                self.start_line - 1,
                self.start_line - 1 + self.curr.saturating_sub(self.start), 
                self.line
            ),
            msg
        ));
    }

    fn string_literal(&mut self, ch: &str) {
        let mut back_slash = false;
        loop {
            let c = self.peek();
            if self.is_at_end() || (c == ch && !back_slash) {
                break;
            }

            let old_backslash = back_slash;
            back_slash = false;

            match c {
                "\n" => self.newline(),
                "\\" => {
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
            while is_str_digit(self.peek()) {
                self.advance();
            }
        }

        let c = self.peek();
        if c == "." {
            self.advance();

                if is_str_digit(self.peek()) {
                    loop {
                        self.advance();

                        if !is_str_digit(self.peek()) {
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
            Rc::clone(&self.source),
            Rc::clone(&self.filename), TokenType::Int,
            format!(
                "{}",
                i64::from_str_radix(
                    substring(&self.source.to_string(), self.start + 2, self.curr).as_ref(),
                    base
                ).expect("Scanner failed scanning alt base number")
            ).into(),
            self.start_line - 1,
            self.start_line - 1 + self.curr.saturating_sub(self.start), 
            self.line
        ));
    }

    fn alt_base_number(&mut self) {
        let c = self.peek();
        match c {
            "b" => {
                self.advance();

                while is_str_bin_digit(self.peek()) {
                    self.advance();
                }

                self.add_int_token_with_base_conversion(2);
            }
            "o" => {
                self.advance();

                while is_str_oct_digit(self.peek()) {
                    self.advance();
                }

                self.add_int_token_with_base_conversion(8);
            }
            "x" => {
                self.advance();

                while is_str_hex_digit(self.peek()) {
                    self.advance();
                }

                self.add_int_token_with_base_conversion(16);
            }
            _ => {
                if is_str_digit(c) {
                    loop {
                        self.advance();

                        if !is_str_digit(self.peek()) {
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
        while is_str_alphanumeric(self.peek()) {
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
        let c: Rc<str> = Rc::from(self.advance());
        match c.as_ref() {
            "{" => self.add_token(TokenType::LeftBrace),
            "}" => self.add_token(TokenType::RightBrace),
            
            "\""  => self.string_literal("\""),
            " " | "\r" | "\t" => (),
            "\n" => self.newline(),

            "/" => {
                if self.match_("/") {
                    while self.peek() != "\n" && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.error(format!("Unexpected character '{}'", c).as_str());
                }
            }

            "0" => self.alt_base_number(),

            _ => {
                if is_str_beginning_digit(c.as_ref()) {
                    self.number(true);
                } else if is_str_alpha(c.as_ref()) {
                    self.identifier();
                } else {
                    self.error(format!("Unexpected character '{}'", c).as_str());
                }
            }
        }
    }

    pub fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.start_line += self.curr - self.start;
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