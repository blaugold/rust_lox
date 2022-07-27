use crate::chunk::{Chunk, Op};

pub static DEBUG_TRACE_EXECUTION: bool = true;

impl Chunk {
    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        let mut offset = 0;
        while offset < self.count() {
            offset = self.disassemble_instruction(offset);
        }
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:04} ", offset);

        if offset > 0 && self.lines()[offset] == self.lines()[offset - 1] {
            print!("   | ");
        } else {
            print!("{:>4} ", self.lines()[offset]);
        }

        let instruction = self.code()[offset];
        let op_code: Result<Op, ()> = instruction.try_into();
        match op_code {
            Ok(op_code) => match op_code {
                Op::Constant => self.constant_instruction("OP_CONSTANT", offset),
                Op::Add => self.simple_instruction("OP_ADD", offset),
                Op::Subtract => self.simple_instruction("OP_SUBTRACT", offset),
                Op::Multiply => self.simple_instruction("OP_MULTIPLY", offset),
                Op::Divide => self.simple_instruction("OP_DIVIDE", offset),
                Op::Negate => self.simple_instruction("OP_NEGATE", offset),
                Op::Return => self.simple_instruction("OP_RETURN", offset),
            },
            _ => {
                println!("Unknown opcode {}", instruction);
                offset + 1
            }
        }
    }

    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{}", name);
        offset + 1
    }

    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        let constant = self.code()[offset + 1];
        print!("{:<16} {:4} '", name, constant);
        self.constants()[constant as usize].print();
        println!("'");
        offset + 2
    }
}
