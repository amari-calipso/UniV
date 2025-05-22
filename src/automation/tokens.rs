use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
pub enum TokenType {
    LeftBrace, RightBrace,

    Identifier,
    String, Int, Float,

    Set, Push, Pop, Reset, Define, Run,
    Shuffle, Distribution, With, And,
    Unique, Sort, In, Length, Speed, 
    Scaled, By, All, Sorts, Shuffles,
    Visual, Queue, Describe, Max,

    EOF
}

#[derive(Clone, PartialEq, PartialOrd, Hash, Eq)]
pub struct Token {
    pub source: Rc<str>,
    pub filename: Rc<str>,
    pub type_: TokenType,
    pub lexeme: Rc<str>,
    pub pos: usize,
    pub end: usize,
    pub line: usize
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Token").field("type_", &self.type_).field("lexeme", &self.lexeme).field("pos", &self.pos).field("end", &self.end).field("line", &self.line).finish()
    }
}

impl Token {
    pub fn new(source: Rc<str>, filename: Rc<str>, type_: TokenType, lexeme: Rc<str>, pos: usize, end: usize, line: usize) -> Self {
        Token { source, filename, type_, lexeme, pos, end, line }
    }
}