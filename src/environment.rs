use std::collections::HashMap;

use crate::interpreter::RuntimeError;
use crate::token::Token;

pub struct Environment {
    values: HashMap<String, crate::token::Literal>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: crate::token::Literal) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<crate::token::Literal, RuntimeError> {
        match self.values.get(&name.lexeme) {
            Some(value) => Ok(value.clone()),

            None => Err(RuntimeError::new(
                name.clone(),
                format!("Undefined variable '{}'.", name.lexeme),
            )),
        }
    }
}
