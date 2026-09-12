use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::environment;
use crate::interpreter::RuntimeError;
use crate::token::{Token, Value};

pub type EnvironmentRef = Rc<RefCell<Environment>>;

pub struct Environment {
    values: HashMap<String, Option<Value>>,
    local_values: Vec<Option<Value>>,
    enclosing: Option<EnvironmentRef>,
}

impl Environment {
    pub fn new() -> EnvironmentRef {
        Rc::new(RefCell::new(Self {
            values: HashMap::new(),
            local_values: Vec::new(),
            enclosing: None,
        }))
    }

    pub fn from(enclosing: EnvironmentRef) -> EnvironmentRef {
        Rc::new(RefCell::new(Self {
            values: HashMap::new(),
            local_values: Vec::new(),
            enclosing: Some(enclosing),
        }))
    }

    pub fn enclosing(&self) -> Option<EnvironmentRef> {
        self.enclosing.clone()
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

    pub fn define_local(&mut self, value: Option<Value>) {
        self.local_values.push(value);
    }

    pub fn assign_last_local(&mut self, value: Option<Value>) {
        let slot = self
            .local_values
            .last_mut()
            .expect("No local slot to assign");

        *slot = value;
    }

    pub fn get(&self, name: &Token) -> Result<Value, RuntimeError> {
        match self.values.get(&name.lexeme) {
            Some(Some(value)) => Ok(value.clone()),

            Some(None) => Ok(Value::Literal(crate::token::Literal::Nil)),

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
        slot: usize,
        name: &Token,
    ) -> Result<Value, RuntimeError> {
        let ancestor = Self::ancestor(environment, distance);
        let environment = ancestor.borrow();

        let value = environment
            .local_values
            .get(slot)
            .expect("Resolver produced an invalid local slot");

        match value {
            Some(value) => Ok(value.clone()),
            None => Ok(Value::Literal(crate::token::Literal::Nil)),
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

    pub fn assign_at(environment: EnvironmentRef, distance: usize, slot: usize, value: Value) {
        let ancestor = Self::ancestor(environment, distance);
        let mut environment = ancestor.borrow_mut();

        let target = environment
            .local_values
            .get_mut(slot)
            .expect("Resolver produced an invalid local slot");

        *target = Some(value);
    }
}
