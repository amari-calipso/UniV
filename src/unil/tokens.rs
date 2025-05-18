use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
pub enum TokenType {
    LeftParen, RightParen,
    LeftBrace, RightBrace,
    LeftSquare, RightSquare,

    Comma, Dot, Minus, Plus, Semicolon,
    Slash, Star, Colon, ShiftLeft,
    ShiftRight, Mod, Tilde,

    PlusPlus, MinusMinus,
    PlusEquals, MinusEquals,
    StarEquals, SlashEquals,
    OrEquals, AndEquals,
    XorEquals, ModEquals,
    ShiftLeftEquals, ShiftRightEquals,

    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual, Walrus,

    LogicAnd, LogicOr,
    BitwiseAnd, BitwiseOr, BitwiseXor,

    Question, Hash, At, Dollar,

    Identifier,
    String, Int, Float,

    Else, Fn, For, If, Return,
    While, Switch, Continue,
    Break, Do, Default, Throw,
    Foreach, Null, Or, Try, Catch, Drop,
    True, False,

    EOF
}

impl TokenType {
    pub fn stringify(&self) -> &str {
        match self {
            TokenType::LeftParen => "(",
            TokenType::RightParen => ")",
            TokenType::LeftBrace => "{",
            TokenType::RightBrace => "}",
            TokenType::LeftSquare => "[",
            TokenType::RightSquare => "]",
            TokenType::Comma => ",",
            TokenType::Dot => ".",
            TokenType::Minus => "-",
            TokenType::Plus => "+",
            TokenType::Semicolon => ";",
            TokenType::Slash => "/",
            TokenType::Star => "*",
            TokenType::Colon => ":",
            TokenType::ShiftLeft => "<<",
            TokenType::ShiftRight => ">>",
            TokenType::Mod => "%",
            TokenType::Tilde => "~",
            TokenType::PlusPlus => "++",
            TokenType::MinusMinus => "--",
            TokenType::PlusEquals => "+=",
            TokenType::MinusEquals => "-=",
            TokenType::StarEquals => "*=",
            TokenType::SlashEquals => "/=",
            TokenType::OrEquals => "|=",
            TokenType::AndEquals => "&=",
            TokenType::XorEquals => "^=",
            TokenType::ModEquals => "%=",
            TokenType::ShiftLeftEquals => ">>=",
            TokenType::ShiftRightEquals => "<<=",
            TokenType::Bang => "!",
            TokenType::BangEqual => "!=",
            TokenType::Equal => "=",
            TokenType::Walrus => ":=",
            TokenType::EqualEqual => "==",
            TokenType::Greater => ">",
            TokenType::GreaterEqual => ">=",
            TokenType::Less => "<",
            TokenType::LessEqual => "<=",
            TokenType::LogicAnd => "&&",
            TokenType::LogicOr => "||",
            TokenType::BitwiseAnd => "&",
            TokenType::BitwiseOr => "|",
            TokenType::BitwiseXor => "^",
            TokenType::Question => "?",
            TokenType::Hash => "#",
            TokenType::At => "@",
            TokenType::Dollar => "$",
            TokenType::Else => "else",
            TokenType::Fn => "fn",
            TokenType::For => "for",
            TokenType::If => "if",
            TokenType::Return => "return",
            TokenType::While => "while",
            TokenType::Switch => "switch",
            TokenType::Continue => "continue",
            TokenType::Break => "break",
            TokenType::Do => "do",
            TokenType::Default => "default",
            TokenType::Throw => "throw",
            TokenType::Foreach => "foreach",
            TokenType::Null => "null",
            TokenType::Or => "or",
            TokenType::Try => "try",
            TokenType::Catch => "catch",
            TokenType::Drop => "drop",
            TokenType::True => "true",
            TokenType::False => "false",
            _ => ""
        }
    }
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

    pub fn dummy(lexeme: Rc<str>) -> Self {
        Token { source: Rc::from(""), filename: Rc::from(""), type_: TokenType::Identifier, lexeme, pos: 0, end: 1, line: 0 }
    }

    pub fn empty() -> Self {
        Token::dummy(Rc::from(""))
    }

    pub fn set_type(&mut self, type_: TokenType) {
        self.type_ = type_;
    }

    pub fn set_lexeme(&mut self, lexeme: &str) {
        self.lexeme = Rc::from(lexeme);
    }
}