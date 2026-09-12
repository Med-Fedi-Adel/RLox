use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::{
    interpreter::RuntimeError,
    interpreter::lox_class::LoxClass,
    token::{Token, Value},
};

pub type LoxInstanceRef = Rc<RefCell<LoxInstance>>;

pub struct LoxInstance {
    klass: Rc<LoxClass>,
    fields: HashMap<String, Value>,
}

impl LoxInstance {
    pub fn new(klass: Rc<LoxClass>) -> Self {
        Self {
            klass,
            fields: HashMap::new(),
        }
    }

    pub fn klass_name(&self) -> &str {
        self.klass.name()
    }

    pub fn get(&self, name: &Token) -> Result<Value, RuntimeError> {
        match self.fields.get(&name.lexeme) {
            Some(value) => Ok(value.clone()),

            None => Err(RuntimeError::new(
                name.clone(),
                format!("Undefined property '{}'.", name.lexeme),
            )),
        }
    }

    pub fn set(&mut self, name: &Token, value: Value) {
        self.fields.insert(name.lexeme.clone(), value);
    }
}
