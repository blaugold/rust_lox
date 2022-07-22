use std::collections::HashMap;

use crate::{
    interpreter::{RuntimeError, RuntimeValue},
    token::Token,
};

pub struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, RuntimeValue>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn set_enclosing(&mut self, enclosing: Box<Environment>) {
        self.enclosing = Some(enclosing)
    }

    pub fn take_enclosing(&mut self) -> Box<Environment> {
        std::mem::replace(&mut self.enclosing, None).unwrap()
    }

    pub fn define(&mut self, name: &str, value: RuntimeValue) -> Result<(), RuntimeError> {
        self.values.insert(name.to_string(), value);
        Ok(())
    }

    pub fn assign(&mut self, name: &Token, value: RuntimeValue) -> Result<(), RuntimeError> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.to_string(), value);
            Ok(())
        } else {
            match &mut self.enclosing {
                Some(enclosing) => enclosing.assign(name, value),
                None => Err(RuntimeError {
                    message: format!("Cannot assign to undefined variable '{}'.", name.lexeme),
                    token: name.clone(),
                }),
            }
        }
    }

    pub fn get(&self, name: &Token) -> Result<RuntimeValue, RuntimeError> {
        match self.values.get(&name.lexeme) {
            Some(value) => return Ok(value.clone()),
            None => match &self.enclosing {
                Some(enclosing) => enclosing.get(name),
                None => Err(RuntimeError {
                    message: format!("Variable '{}' is not defined.", name.lexeme),
                    token: name.clone(),
                }),
            },
        }
    }
}
