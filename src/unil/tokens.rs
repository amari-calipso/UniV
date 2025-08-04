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

impl Default for TokenType {
    fn default() -> Self {
        TokenType::Identifier
    }
}

pub type Token = alanglib::token::Token<TokenType>;