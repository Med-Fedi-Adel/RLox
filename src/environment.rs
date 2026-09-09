use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::environment;
use crate::interpreter::RuntimeError;
use crate::token::{Token, Value};

pub type EnvironmentRef = Rc<RefCell<Environment>>;

pub struct Environment {
    values: HashMap<String, Option<Value>>,
    enclosing: Option<EnvironmentRef>,
}

impl Environment {
    pub fn new() -> EnvironmentRef {
        Rc::new(RefCell::new(Self {
            values: HashMap::new(),
            enclosing: None,
        }))
    }

    pub fn from(enclosing: EnvironmentRef) -> EnvironmentRef {
        Rc::new(RefCell::new(Self {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }))
    }

    pub fn ancestor(environment: EnvironmentRef, distance: usize) -> EnvironmentRef {
        let mut current = environment;

        for _ in 0..distance {
            let enclosing = current
                .borrow()
                .enclosing
                .clone()
                .expect("Resolver produced an invalid environment distance");

            current = enclosing;
        }

        current
    }

    pub fn define(&mut self, name: String, value: Option<Value>) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Value, RuntimeError> {
        match self.values.get(&name.lexeme) {
            Some(Some(value)) => Ok(value.clone()),

            Some(None) => Err(RuntimeError::new(
                name.clone(),
                format!("Uninitialized variable '{}'.", name.lexeme),
            )),

            None => {
                if let Some(enclosing) = &self.enclosing {
                    return enclosing.borrow().get(name);
                }

                Err(RuntimeError::new(
                    name.clone(),
                    format!("Undefined variable '{}'.", name.lexeme),
                ))
            }
        }
    }

    pub fn get_at(
        environment: EnvironmentRef,
        distance: usize,
        name: &Token,
    ) -> Result<Value, RuntimeError> {
        let ancestor = Self::ancestor(environment, distance);

        let env = ancestor.borrow();

        match env.values.get(&name.lexeme) {
            Some(Some(value)) => Ok(value.clone()),
            Some(None) => Err(RuntimeError::new(
                name.clone(),
                format!("Uninitialized variable '{}'.", name.lexeme),
            )),

            None => Err(RuntimeError::new(
                name.clone(),
                format!("Undefined variable '{}'.", name.lexeme),
            )),
        }
    }

    pub fn assign(&mut self, name: &Token, value: Value) -> Result<(), RuntimeError> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), Some(value));
            return Ok(());
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow_mut().assign(name, value);
        }

        Err(RuntimeError::new(
            name.clone(),
            format!("Undefined variable '{}'.", name.lexeme),
        ))
    }

    pub fn assign_at(
        environment: EnvironmentRef,
        distance: usize,
        name: &Token,
        value: Value,
    ) -> Result<(), RuntimeError> {
        let ancestor = Self::ancestor(environment, distance);

        let mut env = ancestor.borrow_mut();

        if env.values.contains_key(&name.lexeme) {
            env.values.insert(name.lexeme.clone(), Some(value));
            Ok(())
        } else {
            Err(RuntimeError::new(
                name.clone(),
                format!("Undefined variable '{}'.", name.lexeme),
            ))
        }
    }
}
