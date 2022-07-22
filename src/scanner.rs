use crate::token::LiteralValue;

use super::lox::Lox;
use super::token::{Token, TokenType};

pub struct Scanner<'a> {
    lox: &'a mut Lox,
    source: &'a str,
    bytes: &'a [u8],
    line: usize,
    start: usize,
    current: usize,
    tokens: Vec<Token>,
}

impl<'a> Scanner<'a> {
    pub fn new(lox: &'a mut Lox, source: &'a str) -> Scanner<'a> {
        Scanner {
            lox,
            source,
            bytes: source.as_bytes(),
            line: 1,
            start: 0,
            current: 0,
            tokens: Vec::new(),
        }
    }

    pub fn scan_tokens(mut self) -> (Vec<Token>, &'a mut Lox) {
        while !self.is_at_end() {
            self.scan_token();
            self.start = self.current;
        }

        self.add_token(TokenType::Eof);

        (self.tokens, self.lox)
    }

    fn scan_token(&mut self) {
        let character = self.advance();

        match character {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            '.' => self.add_token(TokenType::Dot),
            ',' => self.add_token(TokenType::Comma),
            ';' => self.add_token(TokenType::Semicolon),
            '+' => self.add_token(TokenType::Plus),
            '-' => self.add_token(TokenType::Minus),
            '/' => self.add_token(TokenType::Slash),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                let token_type = match self.match_char('=') {
                    true => TokenType::BangEqual,
                    false => TokenType::Bang,
                };
                self.add_token(token_type)
            }
            '=' => {
                let token_type = match self.match_char('=') {
                    true => TokenType::EqualEqual,
                    false => TokenType::Equal,
                };
                self.add_token(token_type)
            }
            '<' => {
                let token_type = match self.match_char('=') {
                    true => TokenType::LessEqual,
                    false => TokenType::Less,
                };
                self.add_token(token_type)
            }
            '>' => {
                let token_type = match self.match_char('=') {
                    true => TokenType::GreaterEqual,
                    false => TokenType::Greater,
                };
                self.add_token(token_type)
            }
            '"' => self.string(),
            ' ' | '\t' => {}
            '\n' => {
                self.line += 1;
            }
            _ => {
                if is_digit(character) {
                    self.number();
                } else if is_alpha(character) {
                    self.identifier();
                } else {
                    let message = format!("Unexpected character '{}'.", character);
                    self.lox.scanner_error(self.line, &message);
                }
            }
        }
    }

    fn string(&mut self) {
        loop {
            if self.is_at_end() {
                break;
            }

            match self.peek() {
                '\n' => {
                    self.line += 1;
                }
                '"' => {
                    break;
                }
                _ => {}
            }

            self.advance();
        }

        if !self.match_char('"') {
            self.lox.scanner_error(self.line, "Unterminated string.");
            return;
        }

        let lexeme = self.lexeme();
        let value = lexeme[1..(lexeme.len() - 1)].to_string();
        self.add_full_token(TokenType::String, Some(LiteralValue::String(value)));
    }

    fn number(&mut self) {
        while !self.is_at_end() && is_digit(self.peek()) {
            self.advance();
        }

        if self.match_char('.') {
            while !self.is_at_end() && is_digit(self.peek()) {
                self.advance();
            }
        }

        let value = self.lexeme().parse::<f64>().unwrap();
        self.add_full_token(TokenType::Number, Some(LiteralValue::Number(value)));
    }

    fn identifier(&mut self) {
        while !self.is_at_end() && is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let type_ = resolve_keyword_type(self.lexeme()).unwrap_or(TokenType::Identifier);
        self.add_token(type_)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn peek(&self) -> char {
        self.bytes[self.current] as char
    }

    fn advance(&mut self) -> char {
        let current = self.peek();
        self.current += 1;
        current
    }

    fn match_char(&mut self, character: char) -> bool {
        if !self.is_at_end() && self.peek() == character {
            self.advance();
            return true;
        }

        false
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_full_token(token_type, Option::None)
    }

    fn add_full_token(&mut self, token_type: TokenType, literal: Option<LiteralValue>) {
        self.tokens.push(Token {
            token_type: token_type,
            line: self.line,
            literal,
            lexeme: self.lexeme().to_string(),
        })
    }

    fn lexeme(&self) -> &'a str {
        &self.source[self.start..self.current]
    }
}

fn is_digit(character: char) -> bool {
    character >= '0' && character <= '9'
}

fn is_alpha(character: char) -> bool {
    (character >= 'A' && character <= 'Z') || (character >= 'a' && character <= 'z')
}

fn is_alpha_numeric(character: char) -> bool {
    is_digit(character) || is_alpha(character)
}

fn resolve_keyword_type(lexeme: &str) -> Option<TokenType> {
    match lexeme {
        "var" => Some(TokenType::Var),
        "fun" => Some(TokenType::Fun),
        "class" => Some(TokenType::Class),
        "this" => Some(TokenType::This),
        "super" => Some(TokenType::Super),
        "if" => Some(TokenType::If),
        "else" => Some(TokenType::Else),
        "for" => Some(TokenType::For),
        "while" => Some(TokenType::While),
        "return" => Some(TokenType::Return),
        "print" => Some(TokenType::Print),
        "and" => Some(TokenType::And),
        "or" => Some(TokenType::Or),
        "true" => Some(TokenType::True),
        "false" => Some(TokenType::False),
        "nil" => Some(TokenType::Nil),
        _ => None,
    }
}
