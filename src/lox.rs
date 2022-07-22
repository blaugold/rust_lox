use std::{
    env,
    fs::File,
    io::{self, Read, Write},
    process::exit,
};

use crate::{
    interpreter::{Interpreter, RuntimeError},
    parser::Parser,
    scanner::Scanner,
    token::{Token, TokenType},
};

pub struct Lox {
    had_error: bool,
    had_runtime_error: bool,
    interpreter: Interpreter,
}

impl Lox {
    pub fn new() -> Lox {
        Lox {
            had_error: false,
            had_runtime_error: false,
            interpreter: Interpreter::new(),
        }
    }

    pub fn main(&mut self) {
        let args = Vec::from_iter(env::args().skip(1));

        match args.len() {
            0 => self.run_prompt(),
            1 => self.run_file(&args[0]),
            _ => {
                print!("Usage: rust_lox [<file>]");
                exit(1);
            }
        }
    }

    fn run_prompt(&mut self) {
        let mut lines = io::stdin().lines();

        loop {
            print!("> ");
            io::stdout().flush().unwrap();

            match lines.next() {
                Some(line) => {
                    self.run(&line.unwrap());
                    self.had_error = false;
                    self.had_runtime_error = false;
                }
                None => {
                    return;
                }
            }
        }
    }

    fn run_file(&mut self, path: &str) {
        let mut file = File::open(path).expect("Could not open file to run.");
        let mut content = String::new();
        file.read_to_string(&mut content)
            .expect("Could not read file to run.");

        self.run(&content);

        if self.had_error {
            exit(1);
        }
        if self.had_runtime_error {
            exit(1);
        }
    }

    fn run(&mut self, source: &str) {
        let scanner = Scanner::new(self, source);
        let (tokens, lox) = scanner.scan_tokens();
        let parser = Parser::new(lox, tokens);
        let (statements, lox) = parser.parse();

        if !lox.had_error {
            match lox.interpreter.interpret(&statements) {
                Err(err) => lox.runtime_error(err),
                Ok(_) => {}
            }
        }
    }

    pub fn scanner_error(&mut self, line: usize, message: &str) {
        self.report_static_error(line, "", message);
    }

    pub fn parser_error(&mut self, token: &Token, message: &str) {
        if token.token_type == TokenType::Eof {
            self.report_static_error(token.line, " at end", message);
        } else {
            self.report_static_error(token.line, &format!(" at '{}'", token.lexeme), message);
        }
    }

    pub fn runtime_error(&mut self, err: RuntimeError) {
        println!("{} [line {}]", err.message, err.token.line);
        self.had_runtime_error = true;
    }

    fn report_static_error(&mut self, line: usize, at: &str, message: &str) {
        println!("[line {}] Error{}: {}", line, at, message);
        self.had_error = true;
    }
}
