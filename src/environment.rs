use std::{collections::HashMap, rc::Rc};

use crate::{
    interpreter::{RuntimeError, RuntimeValue},
    token::Token,
};

pub struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, Rc<RuntimeValue>>,
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

    pub fn define<'a>(
        &mut self,
        name: &str,
        value: Rc<RuntimeValue>,
    ) -> Result<(), RuntimeError<'a>> {
        self.values.insert(name.to_string(), value);
        Ok(())
    }

    pub fn assign<'a>(
        &mut self,
        name: &'a Token<'a>,
        value: Rc<RuntimeValue>,
    ) -> Result<(), RuntimeError<'a>> {
        if self.values.contains_key(name.lexeme) {
            self.values.insert(name.lexeme.to_string(), value);
            Ok(())
        } else {
            match &mut self.enclosing {
                Some(enclosing) => enclosing.assign(name, value),
                None => Err(RuntimeError {
                    message: format!("Cannot assign to undefined variable '{}'.", name.lexeme),
                    token: name,
                }),
            }
        }
    }

    pub fn get<'a>(&self, name: &'a Token<'a>) -> Result<Rc<RuntimeValue>, RuntimeError<'a>> {
        match self.values.get(name.lexeme) {
            Some(value) => return Ok(value.clone()),
            None => match &self.enclosing {
                Some(enclosing) => enclosing.get(name),
                None => Err(RuntimeError {
                    message: format!("Variable '{}' is not defined.", name.lexeme),
                    token: name,
                }),
            },
        }
    }
}
