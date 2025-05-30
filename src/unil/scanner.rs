use std::collections::HashMap;
use std::rc::Rc;
use lazy_static::lazy_static;

use crate::unil::tokens::{Token, TokenType};
use crate::utils::{substring, lang::{error, is_alpha, is_alphanumeric, is_beginning_digit, is_bin_digit, is_digit, is_hex_digit, is_oct_digit}};

lazy_static! {
    pub static ref KEYWORDS: HashMap<&'static str, TokenType> = {
        let mut keywords = HashMap::new();
        keywords.insert(      "do", TokenType::Do);
        keywords.insert(      "fn", TokenType::Fn);
        keywords.insert(      "if", TokenType::If);
        keywords.insert(      "or", TokenType::Or);
        keywords.insert(     "for", TokenType::For);
        keywords.insert(     "try", TokenType::Try);
        keywords.insert(    "drop", TokenType::Drop);
        keywords.insert(    "else", TokenType::Else);
        keywords.insert(    "null", TokenType::Null);
        keywords.insert(    "true", TokenType::True);
        keywords.insert(   "break", TokenType::Break);
        keywords.insert(   "catch", TokenType::Catch);
        keywords.insert(   "false", TokenType::False);
        keywords.insert(   "throw", TokenType::Throw);
        keywords.insert(   "while", TokenType::While);
        keywords.insert(  "return", TokenType::Return);
        keywords.insert(  "switch", TokenType::Switch);
        keywords.insert( "default", TokenType::Default);
        keywords.insert( "foreach", TokenType::Foreach);
        keywords.insert("continue", TokenType::Continue);
        keywords
    };
}

pub struct Scanner<'a> {
    source: &'a String,
    filename: Rc<str>,
    pub tokens: Vec<Token>,
    start_positions: Vec<usize>,

    start: usize,
    curr:  usize,
    line:  usize,

    pub errors: Vec<String>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a String, filename: Rc<str>) -> Self {
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
            substring(&self.source, self.start, self.curr).into(),
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

    fn peek_next(&self) -> char {
        if self.curr + 1 >= self.source.len() {
            '\0'
        } else {
            self.source.chars().nth(self.curr + 1).unwrap()
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
                    substring(&self.source, self.start + 2, self.curr).as_ref(),
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
            substring(&self.source, self.start, self.curr).as_str()
        );

        if let Some(type_) = kind {
            self.add_token(*type_)
        } else {
            self.add_token(TokenType::Identifier);
        }
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '[' => self.add_token(TokenType::LeftSquare),
            ']' => self.add_token(TokenType::RightSquare),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            ';' => self.add_token(TokenType::Semicolon),
            '~' => self.add_token(TokenType::Tilde),
            '#' => self.add_token(TokenType::Hash),
            '@' => self.add_token(TokenType::At),
            '$' => self.add_token(TokenType::Dollar),
            '?' => self.add_token(TokenType::Question),

            '!' => {
                if self.match_('=') {
                    self.add_token(TokenType::BangEqual);
                } else {
                    self.add_token(TokenType::Bang);
                }
            }
            ':' => {
                if self.match_('=') {
                    self.add_token(TokenType::Walrus);
                } else {
                    self.add_token(TokenType::Colon);
                }
            }
            '=' => {
                if self.match_('=') {
                    self.add_token(TokenType::EqualEqual);
                } else {
                    self.add_token(TokenType::Equal);
                }
            }
            '*' => {
                if self.match_('=') {
                    self.add_token(TokenType::StarEquals);
                } else {
                    self.add_token(TokenType::Star);
                }
            }
            '%' => {
                if self.match_('=') {
                    self.add_token(TokenType::ModEquals);
                } else {
                    self.add_token(TokenType::Mod);
                }
            }
            '^' => {
                if self.match_('=') {
                    self.add_token(TokenType::XorEquals);
                } else {
                    self.add_token(TokenType::BitwiseXor);
                }
            }

            '+' => {
                if self.match_('+') {
                    self.add_token(TokenType::PlusPlus);
                } else if self.match_('=') {
                    self.add_token(TokenType::PlusEquals);
                } else {
                    self.add_token(TokenType::Plus);
                }
            }
            '|' => {
                if self.match_('|') {
                    self.add_token(TokenType::LogicOr);
                } else if self.match_('=') {
                    self.add_token(TokenType::OrEquals);
                } else {
                    self.add_token(TokenType::BitwiseOr);
                }
            }
            '&' => {
                if self.match_('&') {
                    self.add_token(TokenType::LogicAnd);
                } else if self.match_('=') {
                    self.add_token(TokenType::AndEquals);
                } else {
                    self.add_token(TokenType::BitwiseAnd);
                }
            }
            '-' => {
                if self.match_('-') {
                    self.add_token(TokenType::MinusMinus);
                } else if self.match_('=') {
                    self.add_token(TokenType::MinusEquals);
                } else {
                    self.add_token(TokenType::Minus);
                }
            }
            '<' => {
                if self.match_('<') {
                    if self.match_('=') {
                        self.add_token(TokenType::ShiftLeftEquals);
                    } else {
                        self.add_token(TokenType::ShiftLeft);
                    }
                } else if self.match_('=') {
                    self.add_token(TokenType::LessEqual);
                } else {
                    self.add_token(TokenType::Less);
                }
            }
            '>' => {
                if self.match_('>') {
                    if self.match_('=') {
                        self.add_token(TokenType::ShiftRightEquals);
                    } else {
                        self.add_token(TokenType::ShiftRight);
                    }
                } else if self.match_('=') {
                    self.add_token(TokenType::GreaterEqual);
                } else {
                    self.add_token(TokenType::Greater);
                }
            }

            '/' => {
                if self.match_('=') {
                    self.add_token(TokenType::SlashEquals);
                } else if self.match_('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.match_('*') {
                    while !(
                        (self.peek() == '*' && self.peek_next() == '/') ||
                        self.is_at_end()
                    ) {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            }

            '"'  => self.string_literal('"'),
            '\'' => self.string_literal('\''),
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,

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