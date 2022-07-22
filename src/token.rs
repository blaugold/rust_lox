#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Single character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Dot,
    Comma,
    Semicolon,
    Plus,
    Minus,
    Slash,
    Star,

    // One or two-character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    // Keywords.
    Var,
    Fun,
    Class,
    This,
    Super,
    If,
    Else,
    For,
    While,
    Return,
    Print,
    And,
    Or,
    True,
    False,
    Nil,

    // Literals.
    Number,
    String,
    Identifier,

    // End of file.
    Eof,
}

#[derive(Debug, Clone)]
pub enum LiteralValue {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub literal: Option<LiteralValue>,
}
