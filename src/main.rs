mod ast;
mod interpreter;
mod lox;
mod scanner;
mod token;

fn main() {
    lox::Lox::new().main();
}
