use std::slice;

use crate::{
    chunk::{Chunk, Op},
    debug::DEBUG_TRACE_EXECUTION,
    value::Value,
};

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

const INITIAL_STACK_CAPACITY: usize = 256;

pub struct VM {
    stack: Vec<Value>,
}

impl VM {
    pub fn new() -> VM {
        VM {
            stack: Vec::with_capacity(INITIAL_STACK_CAPACITY),
        }
    }

    pub fn interpret(&mut self, chunk: &Chunk) -> InterpretResult {
        self.stack.clear();

        Runner::new(&mut self.stack, chunk).run()
    }
}

struct Runner<'a> {
    stack: &'a mut Vec<Value>,
    chunk: &'a Chunk,
    ip: slice::Iter<'a, u8>,
}

impl<'a> Runner<'a> {
    fn new(stack: &'a mut Vec<Value>, chunk: &'a Chunk) -> Self {
        Self {
            stack,
            chunk,
            ip: chunk.code().iter(),
        }
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            if DEBUG_TRACE_EXECUTION {
                print!(" ");
                for value in self.stack.iter() {
                    print!("[ ");
                    value.print();
                    print!(" ]");
                }
                println!();

                self.chunk
                    .disassemble_instruction(self.instruction_offset());
            }

            let instruction = self.read_byte();
            let op: Result<Op, ()> = instruction.try_into();
            let op = unsafe { op.unwrap_unchecked() };
            match op {
                Op::Constant => {
                    let constant = self.read_constant();
                    self.push(constant);
                }
                Op::Add => self.binary_op(|a, b| a + b),
                Op::Subtract => self.binary_op(|a, b| a - b),
                Op::Multiply => self.binary_op(|a, b| a * b),
                Op::Divide => self.binary_op(|a, b| a / b),
                Op::Negate => self.unary_op(|x| -x),
                Op::Return => {
                    self.pop().print();
                    println!();
                    return InterpretResult::Ok;
                }
            }
        }
    }

    fn read_byte(&mut self) -> u8 {
        unsafe { *self.ip.next().unwrap_unchecked() }
    }

    fn read_constant(&mut self) -> Value {
        self.chunk.constants()[self.read_byte() as usize]
    }

    fn instruction_offset(&self) -> usize {
        self.chunk.code().len() - self.ip.as_slice().len()
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value)
    }

    fn pop(&mut self) -> Value {
        unsafe { self.stack.pop().unwrap_unchecked() }
    }

    fn unary_op(&mut self, op: fn(f64) -> f64) {
        let last = unsafe { self.stack.last_mut().unwrap_unchecked() };
        *last = Value(op(last.0));
    }

    fn binary_op(&mut self, op: fn(f64, f64) -> f64) {
        let b = self.pop();
        let a = self.pop();
        let result = Value(op(a.0, b.0));
        self.push(result);
    }
}
