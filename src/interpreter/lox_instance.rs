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

    pub fn get(instance: LoxInstanceRef, name: &Token) -> Result<Value, RuntimeError> {
        {
            let instance = instance.borrow();

            if let Some(value) = instance.fields.get(&name.lexeme) {
                return Ok(value.clone());
            }
        }

        let method = instance.borrow().klass.find_method(&name.lexeme);

        if let Some(method) = method {
            return Ok(Value::Callable(method.bind(instance)));
        }

        Err(RuntimeError::new(
            name.clone(),
            format!("Undefined property '{}'.", name.lexeme),
        ))
    }

    pub fn set(&mut self, name: &Token, value: Value) {
        self.fields.insert(name.lexeme.clone(), value);
    }
}
