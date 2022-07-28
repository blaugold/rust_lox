mod array;
mod chunk;
mod compiler;
mod debug;
mod lox;
mod memory;
mod scanner;
mod value;
mod vm;

fn main() {
    lox::Lox::new().main();
}
