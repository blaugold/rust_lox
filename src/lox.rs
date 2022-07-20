use std::{
    env,
    fs::File,
    io::{self, Read, Write},
    process::exit,
};

use crate::scanner::Scanner;

pub struct Lox {
    had_error: bool,
}

impl Lox {
    pub fn new() -> Lox {
        Lox { had_error: false }
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
    }

    fn run(&mut self, source: &str) {
        let scanner = Scanner::new(self, source);
        let (tokens, _lox) = scanner.scan_tokens();

        println!("Tokens: {:#?}", tokens);
    }

    pub fn scanner_error(&mut self, line: usize, message: &str) {
        Lox::report(line, "", message);
        self.had_error = true;
    }

    fn report(line: usize, at: &str, message: &str) {
        println!("[line {}] Error{}: {}", line, at, message)
    }
}
