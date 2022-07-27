use std::{
    cell::RefCell,
    env,
    fs::File,
    io::{self, Read, Write},
    process::exit,
    rc::Rc,
};

use crate::{
    interpreter::{Interpreter, RuntimeError},
    parser::Parser,
    resolver::Resolver,
    scanner::Scanner,
    token::{Token, TokenType},
};

pub struct Lox {
    error_collector: Rc<RefCell<ErrorCollector>>,
    interpreter: Interpreter,
}

impl Lox {
    pub fn new() -> Lox {
        let error_collector = Rc::new(RefCell::new(ErrorCollector::new()));

        Lox {
            error_collector: error_collector.clone(),
            interpreter: Interpreter::new(error_collector.clone()),
        }
    }

    pub fn main(&mut self) {
        let args = Vec::from_iter(env::args().skip(1));

        match args.len() {
            0 => self.run_prompt(),
            1 => self.run_file(&args[0]),
            _ => {
                print!("Usage: lox [<file>]");
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
                    self.error_collector.borrow_mut().reset();
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

        if self.error_collector.borrow().had_error {
            exit(1);
        }
        if self.error_collector.borrow().had_runtime_error {
            exit(1);
        }
    }

    fn run(&mut self, source: &str) {
        let mut error_collector = self.error_collector.borrow_mut();
        let scanner = Scanner::new(&mut error_collector, source);
        let tokens = scanner.scan_tokens();
        let parser = Parser::new(&mut error_collector, tokens);
        let statements = parser.parse();

        if error_collector.had_error {
            return;
        }

        let resolver = Resolver::new(&mut error_collector);
        resolver.resolve(&statements);

        if error_collector.had_error {
            return;
        }

        drop(error_collector);

        self.interpreter.interpret(&statements);
    }
}

pub struct ErrorCollector {
    had_error: bool,
    had_runtime_error: bool,
}

impl ErrorCollector {
    fn new() -> ErrorCollector {
        ErrorCollector {
            had_error: false,
            had_runtime_error: false,
        }
    }

    pub fn scanner_error(&mut self, line: usize, message: &str) {
        self.report_static_error(line, "", message);
    }

    pub fn parser_error(&mut self, token: &Token, message: &str) {
        self.report_static_error_for_token(token, message);
    }

    pub fn resolver_error(&mut self, token: &Token, message: &str) {
        self.report_static_error_for_token(token, message);
    }

    pub fn runtime_error(&mut self, err: RuntimeError) {
        println!("{} [line {}]", err.message, err.token.line);
        self.had_runtime_error = true;
    }

    fn reset(&mut self) {
        self.had_error = false;
        self.had_runtime_error = false;
    }

    fn report_static_error_for_token(&mut self, token: &Token, message: &str) {
        if token.token_type == TokenType::Eof {
            self.report_static_error(token.line, " at end", message);
        } else {
            self.report_static_error(token.line, &format!(" at '{}'", token.lexeme), message);
        }
    }

    fn report_static_error(&mut self, line: usize, at: &str, message: &str) {
        println!("[line {}] Error{}: {}", line, at, message);
        self.had_error = true;
    }
}
