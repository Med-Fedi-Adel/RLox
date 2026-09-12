use std::rc::Rc;

use crate::{
    environment::{self, Environment, EnvironmentRef},
    interpreter::{ExecutionResult, LoxCallable, lox_instance::LoxInstanceRef},
    stmt::Stmt,
    token::{Literal, Token, Value},
};

#[derive(Clone)]
pub struct LoxFunction {
    name: Option<Token>,
    params: Rc<Vec<Token>>,
    body: Rc<Vec<Stmt>>,
    closure: EnvironmentRef,
}

impl LoxFunction {
    pub fn new(
        name: Option<Token>,
        params: Rc<Vec<Token>>,
        body: Rc<Vec<Stmt>>,
        closure: EnvironmentRef,
    ) -> Self {
        Self {
            name,
            params,
            body,
            closure,
        }
    }

    pub fn bind(self: &Rc<Self>, instance: LoxInstanceRef) -> Rc<LoxFunction> {
        let environment = Environment::from(self.closure.clone());

        environment
            .borrow_mut()
            .define_local(Some(Value::Instance(instance)));

        Rc::new(LoxFunction::new(
            self.name.clone(),
            self.params.clone(),
            self.body.clone(),
            environment,
        ))
    }
}

impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        self.params.len()
    }

    fn call(
        &self,
        interpreter: &mut super::Interpreter,
        arguments: Vec<crate::token::Value>,
    ) -> Result<crate::token::Value, super::RuntimeError> {
        let environment = Environment::from(self.closure.clone());

        for argument in arguments {
            environment.borrow_mut().define_local(Some(argument));
        }

        match interpreter.execute_block(&self.body, environment) {
            ExecutionResult::Success => Ok(Value::Literal(Literal::Nil)),
            ExecutionResult::RuntimeError(error) => Err(error),
            ExecutionResult::Break => Ok(Value::Literal(Literal::Nil)),
            ExecutionResult::Return(value) => Ok(value),
        }
    }

    fn name(&self) -> String {
        self.name
            .as_ref()
            .map(|t| t.lexeme.clone())
            .unwrap_or_else(|| "anonymous".to_string())
    }
}
