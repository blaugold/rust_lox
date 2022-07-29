use std::str::Chars;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

    // Scanner error.
    Error,
}

#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
    pub token_type: TokenType,
    pub lexeme: &'a str,
    pub line: usize,
}

pub struct Scanner<'a> {
    start: Chars<'a>,
    current: Chars<'a>,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Scanner {
        Scanner {
            start: source.chars(),
            current: source.chars(),
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Token<'a> {
        self.skip_whitespace();
        self.start = self.current.clone();

        if self.is_at_end() {
            return self.make_token(TokenType::Eof);
        }

        let c = self.advance();

        if is_alpha(c) {
            return self.identifier();
        }
        if is_digit(c) {
            return self.number();
        }

        match c {
            '(' => return self.make_token(TokenType::LeftParen),
            ')' => return self.make_token(TokenType::RightParen),
            '{' => return self.make_token(TokenType::LeftBrace),
            '}' => return self.make_token(TokenType::RightBrace),
            ';' => return self.make_token(TokenType::Semicolon),
            ',' => return self.make_token(TokenType::Comma),
            '.' => return self.make_token(TokenType::Dot),
            '-' => return self.make_token(TokenType::Minus),
            '+' => return self.make_token(TokenType::Plus),
            '/' => return self.make_token(TokenType::Slash),
            '*' => return self.make_token(TokenType::Star),
            '!' => {
                let token_type = match self.match_char('=') {
                    true => TokenType::BangEqual,
                    false => TokenType::Bang,
                };
                return self.make_token(token_type);
            }
            '=' => {
                let token_type = match self.match_char('=') {
                    true => TokenType::EqualEqual,
                    false => TokenType::Equal,
                };
                return self.make_token(token_type);
            }
            '<' => {
                let token_type = match self.match_char('=') {
                    true => TokenType::LessEqual,
                    false => TokenType::Less,
                };
                return self.make_token(token_type);
            }
            '>' => {
                let token_type = match self.match_char('=') {
                    true => TokenType::GreaterEqual,
                    false => TokenType::Greater,
                };
                return self.make_token(token_type);
            }
            '"' => return self.string(),
            _ => {}
        }

        return self.error_token("Unexpected character.");
    }

    fn is_at_end(&self) -> bool {
        self.current.as_str().len() == 0
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.current.as_str().as_bytes()[0] as char
        }
    }

    fn peek_next(&self) -> char {
        if self.current.as_str().len() < 2 {
            '\0'
        } else {
            self.current.as_str().as_bytes()[1] as char
        }
    }

    fn advance(&mut self) -> char {
        let char = self.peek();
        self.current.next();
        char
    }

    fn match_char(&mut self, char: char) -> bool {
        if !self.is_at_end() && self.peek() == char {
            self.advance();
            return true;
        }

        false
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                '/' => {
                    if self.peek_next() == '/' {
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        return;
                    }
                }
                _ => return,
            }
        }
    }

    fn make_token(&self, token_type: TokenType) -> Token<'a> {
        Token {
            token_type,
            lexeme: &self.lexeme(),
            line: self.line,
        }
    }

    fn error_token(&self, message: &'static str) -> Token<'a> {
        Token {
            token_type: TokenType::Error,
            lexeme: message,
            line: self.line,
        }
    }

    fn lexeme(&self) -> &'a str {
        let end = self.start.as_str().len() - self.current.as_str().len();
        &self.start.as_str()[0..end]
    }

    fn string(&mut self) -> Token<'a> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return self.error_token("Unterminated string.");
        }

        self.advance();
        self.make_token(TokenType::String)
    }

    fn number(&mut self) -> Token<'a> {
        while is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && is_digit(self.peek_next()) {
            self.advance();

            while is_digit(self.peek()) {
                self.advance();
            }
        };

        self.make_token(TokenType::Number)
    }

    fn identifier(&mut self) -> Token<'a> {
        while is_alpha(self.peek()) || is_digit(self.peek()) {
            self.advance();
        }

        let token_type = self.identifier_type();
        self.make_token(token_type)
    }

    fn identifier_type(&mut self) -> TokenType {
        match self.start.as_str().as_bytes()[0] as char {
            'a' => return self.check_keyword(1, "nd", TokenType::And),
            'c' => return self.check_keyword(1, "class", TokenType::Class),
            'e' => return self.check_keyword(1, "lse", TokenType::Else),
            'f' => {
                if self.start.as_str().len() > 1 {
                    match self.start.as_str().as_bytes()[1] as char {
                        'a' => return self.check_keyword(2, "lse", TokenType::False),
                        'o' => return self.check_keyword(2, "r", TokenType::For),
                        'u' => return self.check_keyword(2, "n", TokenType::Fun),
                        _ => {}
                    }
                }
            }
            'i' => return self.check_keyword(1, "f", TokenType::If),
            'n' => return self.check_keyword(1, "il", TokenType::Nil),
            'o' => return self.check_keyword(1, "r", TokenType::Or),
            'p' => return self.check_keyword(1, "rint", TokenType::Print),
            'r' => return self.check_keyword(1, "eturn", TokenType::Return),
            's' => return self.check_keyword(1, "uper", TokenType::Super),
            't' => {
                if self.start.as_str().len() > 1 {
                    match self.start.as_str().as_bytes()[1] as char {
                        'h' => return self.check_keyword(2, "is", TokenType::This),
                        'r' => return self.check_keyword(2, "ue", TokenType::True),
                        _ => {}
                    }
                }
            }
            'v' => return self.check_keyword(1, "ar", TokenType::Var),
            'w' => return self.check_keyword(1, "hile", TokenType::While),
            _ => {}
        }

        TokenType::Identifier
    }

    fn check_keyword(&mut self, start: usize, rest: &str, token_type: TokenType) -> TokenType {
        if &self.lexeme()[start..] == rest {
            return token_type;
        }

        TokenType::Identifier
    }
}

fn is_digit(char: char) -> bool {
    char >= '0' && char <= '9'
}

fn is_alpha(char: char) -> bool {
    (char >= 'A' && char <= 'Z') || (char >= 'a' && char <= 'z') || char == '_'
}
