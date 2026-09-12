use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    interpreter::lox_function::LoxFunction,
    interpreter::lox_instance::LoxInstance,
    token::Value,
};

pub struct LoxClass {
    name: String,
    methods: HashMap<String, Rc<LoxFunction>>,
}

impl LoxClass {
    pub fn new(name: String, methods: HashMap<String, Rc<LoxFunction>>) -> Self {
        Self { name, methods }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn find_method(&self, name: &str) -> Option<Rc<LoxFunction>> {
        self.methods.get(name).cloned()
    }

    pub fn arity(&self) -> usize {
        0
    }

    pub fn call(self: &Rc<Self>) -> Value {
        Value::Instance(Rc::new(RefCell::new(LoxInstance::new(self.clone()))))
    }
}
