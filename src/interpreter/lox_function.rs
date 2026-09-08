use std::rc::Rc;

use crate::{
    environment::{self, Environment, EnvironmentRef},
    interpreter::{ExecutionResult, LoxCallable},
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

        for (param, argument) in self.params.iter().zip(arguments.into_iter()) {
            environment
                .borrow_mut()
                .define(param.lexeme.clone(), Some(argument));
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
