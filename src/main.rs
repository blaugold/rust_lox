mod ast;
mod environment;
mod interpreter;
mod lox;
mod parser;
mod scanner;
mod token;

fn main() {
    lox::Lox::new().main();
}
