use std::slice;

use crate::{
    chunk::{Chunk, Op},
    compiler::Compiler,
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

    pub fn interpret(&mut self, source: &str) -> InterpretResult {
        let mut chunk = Chunk::new();
        let mut compiler = Compiler::new(source, &mut chunk);

        if !compiler.compile() {
            return InterpretResult::CompileError;
        }

        Runner::new(&mut self.stack, &chunk).run()
    }
}

macro_rules! binary_op {
    ($self:ident, $result_type:ident, $op:tt) => {
        {
            let b = $self.peek(0).clone();
            let a = $self.peek(1).clone();

            if let Value::Number(b) = b {
                if let Value::Number(a) = a {
                    $self.pop();
                    $self.pop();
                    $self.push(Value::$result_type(a $op b));
                    None
                } else {
                    $self.runtime_error("Operands mut be numbers.")
                }
            } else {
                $self.runtime_error("Operands mut be numbers.")
            }
        }
    };
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
        if DEBUG_TRACE_EXECUTION {
            println!("!! Begin Execution !!")
        }

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
            let result = match op {
                Op::Constant => {
                    let constant = self.read_constant();
                    self.push(constant);
                    None
                }
                Op::Nil => {
                    self.push(Value::Nil);
                    None
                }
                Op::True => {
                    self.push(Value::Bool(true));
                    None
                }
                Op::False => {
                    self.push(Value::Bool(false));
                    None
                }
                Op::Equal => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a == b));
                    None
                }
                Op::Greater => binary_op!(self, Bool, >),
                Op::Less => binary_op!(self, Bool, <),
                Op::Add => binary_op!(self, Number, +),
                Op::Subtract => binary_op!(self, Number, -),
                Op::Multiply => binary_op!(self, Number, *),
                Op::Divide => binary_op!(self, Number, /),
                Op::Negate => {
                    let value = self.peek(0);
                    match value {
                        Value::Number(value) => {
                            *value = -*value;
                            None
                        }
                        _ => self.runtime_error("Operand must be a number."),
                    }
                }
                Op::Not => {
                    let value = self.peek(0);
                    *value = Value::Bool(value.is_falsy());
                    None
                }
                Op::Return => {
                    self.pop().print();
                    println!();
                    Some(InterpretResult::Ok)
                }
            };

            if let Some(result) = result {
                if DEBUG_TRACE_EXECUTION {
                    println!("!! End Execution !!")
                }
                return result;
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

    fn peek(&mut self, index: usize) -> &mut Value {
        unsafe {
            let index = self.stack.len() - 1 - index;
            self.stack.get_unchecked_mut(index)
        }
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value)
    }

    fn pop(&mut self) -> Value {
        unsafe { self.stack.pop().unwrap_unchecked() }
    }

    fn runtime_error(&mut self, message: &str) -> Option<InterpretResult> {
        eprintln!("{}", message);

        let instruction = self.instruction_offset() - 1;
        let line = self.chunk.lines()[instruction];
        eprintln!("[line {}] in script", line);

        self.reset_stack();

        Some(InterpretResult::RuntimeError)
    }

    fn reset_stack(&mut self) {
        self.stack.clear();
    }
}
