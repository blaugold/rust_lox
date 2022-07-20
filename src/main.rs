mod lox;
mod scanner;
mod token;
mod ast;

fn main() {
    lox::Lox::new().main();
}
