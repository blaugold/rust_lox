use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    interpreter::{EarlyReturn, RuntimeError, RuntimeValue},
    token::Token,
};

pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, RuntimeValue>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn new_enclosed(enclosing: &Rc<RefCell<Environment>>) -> Self {
        Self {
            enclosing: Some(enclosing.clone()),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &str, value: RuntimeValue) -> Result<(), EarlyReturn> {
        self.values.insert(name.to_string(), value);
        Ok(())
    }

    pub fn assign_at(
        &mut self,
        name: &str,
        scope_index: usize,
        value: RuntimeValue,
    ) -> Result<(), EarlyReturn> {
        self.with_scope_at(scope_index, |scope| {
            scope.insert(name.to_string(), value);
        });
        Ok(())
    }

    pub fn assign(&mut self, name: &Token, value: RuntimeValue) -> Result<(), EarlyReturn> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.to_string(), value);
            Ok(())
        } else {
            match &mut self.enclosing {
                Some(enclosing) => enclosing.borrow_mut().assign(name, value),
                None => RuntimeError {
                    message: format!("Cannot assign to undefined variable '{}'.", name.lexeme),
                    token: name.clone(),
                }
                .into(),
            }
        }
    }

    pub fn get_at(&mut self, name: &str, scope_index: usize) -> Result<RuntimeValue, EarlyReturn> {
        self.with_scope_at(scope_index, |scope| Ok(scope[name].clone()))
    }

    pub fn get(&self, name: &Token) -> Result<RuntimeValue, EarlyReturn> {
        match self.values.get(&name.lexeme) {
            Some(value) => return Ok(value.clone()),
            None => match &self.enclosing {
                Some(enclosing) => enclosing.borrow().get(name),
                None => RuntimeError {
                    message: format!("Variable '{}' is not defined.", name.lexeme),
                    token: name.clone(),
                }
                .into(),
            },
        }
    }

    fn with_scope_at<Fn, T>(&mut self, scope_index: usize, run: Fn) -> T
    where
        Fn: FnOnce(&mut HashMap<String, RuntimeValue>) -> T,
    {
        if scope_index == 0 {
            return run(&mut self.values);
        }

        self.enclosing
            .as_ref()
            .unwrap()
            .borrow_mut()
            .with_scope_at(scope_index - 1, run)
    }
}
