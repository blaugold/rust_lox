use std::{
    env,
    fs::File,
    io::{self, Read, Write},
    process::exit,
};

use crate::vm::{InterpretResult, VM};

pub struct Lox {
    vm: VM,
}

impl Lox {
    pub fn new() -> Lox {
        Lox { vm: VM::new() }
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
                    self.interpret(&line.unwrap());
                }
                None => {
                    return;
                }
            }
        }
    }

    fn run_file(&mut self, path: &str) {
        let mut file = File::open(path).expect("Could not open file to run.");
        let mut source = String::new();
        file.read_to_string(&mut source)
            .expect("Could not read file to run.");

        let result = self.interpret(&source);

        match result {
            InterpretResult::Ok => {}
            InterpretResult::CompileError => exit(65),
            InterpretResult::RuntimeError => exit(70),
        }
    }

    fn interpret(&mut self, source: &str) -> InterpretResult {
        self.vm.interpret(source)
    }
}
