use chunk::{Chunk, Op};
use value::Value;
use vm::VM;

mod array;
mod chunk;
mod debug;
mod memory;
mod value;
mod vm;

fn main() {
    let mut vm = VM::new();

    let mut chunk = Chunk::new();

    let a = chunk.add_constant(Value(1.2));
    chunk.write_op(Op::Constant, 123);
    chunk.write(a as u8, 123);

    let b = chunk.add_constant(Value(3.0));
    chunk.write_op(Op::Constant, 123);
    chunk.write(b as u8, 123);

    chunk.write_op(Op::Negate, 123);

    chunk.write_op(Op::Add, 123);

    chunk.write_op(Op::Return, 123);

    chunk.disassemble("test chunk");

    vm.interpret(&chunk);
}
