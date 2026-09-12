use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    interpreter::{
        Interpreter, LoxCallable, RuntimeError, lox_function::LoxFunction, lox_instance::LoxInstance,
    },
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
        match self.find_method("init") {
            Some(initializer) => initializer.arity(),
            None => 0,
        }
    }

    pub fn call(
        self: &Rc<Self>,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let instance = Rc::new(RefCell::new(LoxInstance::new(self.clone())));

        if let Some(initializer) = self.find_method("init") {
            initializer
                .bind(instance.clone())
                .call(interpreter, arguments)?;
        }

        Ok(Value::Instance(instance))
    }
}
