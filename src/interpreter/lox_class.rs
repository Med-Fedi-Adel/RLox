use std::{cell::RefCell, rc::Rc};

use crate::{
    interpreter::lox_instance::LoxInstance,
    token::Value,
};

#[derive(Debug)]
pub struct LoxClass {
    name: String,
}

impl LoxClass {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arity(&self) -> usize {
        0
    }

    pub fn call(self: &Rc<Self>) -> Value {
        Value::Instance(Rc::new(RefCell::new(LoxInstance::new(self.clone()))))
    }
}
