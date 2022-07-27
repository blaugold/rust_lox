use crate::scanner::{Scanner, TokenType};

pub struct Compiler {}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {}
    }

    pub fn compile(&mut self, source: &str) {
        let mut scanner = Scanner::new(source);
        let mut line: isize = -1;
        loop {
            let token = scanner.scan_token();
            if token.line as isize == line {
                print!("{:4} ", token.line);
                line = token.line as isize;
            } else {
                print!("   | ");
            }
            println!("{:?} '{}'", token.token_type, token.lexeme);

            if token.token_type == TokenType::Eof {
                break;
            }
        }
    }
}
