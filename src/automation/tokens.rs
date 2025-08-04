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
    Timestamp,

    EOF
}

impl Default for TokenType {
    fn default() -> Self {
        TokenType::Identifier
    }
}

pub type Token = alanglib::token::Token<TokenType>;