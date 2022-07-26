mod ast;
mod environment;
mod interpreter;
mod lox;
mod parser;
mod resolver;
mod scanner;
mod token;
mod utils;

fn main() {
    lox::Lox::new().main();
}
