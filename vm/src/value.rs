#[derive(Clone, Copy)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
}

impl Value {
    pub fn print(&self) {
        match self {
            Value::Nil => print!("nil"),
            Value::Bool(value) => print!("{}", value),
            Value::Number(value) => print!("{}", value),
        }
    }

    pub fn is_falsy(&self) -> bool {
        match self {
            Value::Bool(value) => !value,
            _ => true,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use Value::*;
        match (self, other) {
            (Nil, Nil) => true,
            (Bool(l), Bool(r)) => l == r,
            (Number(l), Number(r)) => l == r,
            _ => false,
        }
    }
}

impl Eq for Value {}
