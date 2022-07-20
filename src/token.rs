#[derive(Debug)]
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

#[derive(Debug)]
pub enum LiteralValue<'a> {
    Nil,
    Bool(bool),
    Number(f64),
    String(&'a str),
}

#[derive(Debug)]
pub struct Token<'a> {
    pub token_type: TokenType,
    pub lexeme: &'a str,
    pub line: usize,
    pub literal: Option<LiteralValue<'a>>,
}
