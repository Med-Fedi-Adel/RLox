use std::rc::Rc;

use crate::{
    environment::{self, Environment, EnvironmentRef},
    interpreter::{ExecutionResult, LoxCallable, RuntimeError, lox_instance::LoxInstanceRef},
    stmt::Stmt,
    token::{Literal, Token, TokenType, Value},
};

#[derive(Clone)]
pub struct LoxFunction {
    name: Option<Token>,
    params: Rc<Vec<Token>>,
    body: Rc<Vec<Stmt>>,
    closure: EnvironmentRef,
    is_initializer: bool,
}

impl LoxFunction {
    pub fn new(
        name: Option<Token>,
        params: Rc<Vec<Token>>,
        body: Rc<Vec<Stmt>>,
        closure: EnvironmentRef,
        is_initializer: bool,
    ) -> Self {
        Self {
            name,
            params,
            body,
            closure,
            is_initializer,
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
            self.is_initializer,
        ))
    }

    fn get_bound_this(&self) -> Result<Value, RuntimeError> {
        let this = Token::new(TokenType::This, "this".to_string(), None, 1);

        Environment::get_at(self.closure.clone(), 0, 0, &this)
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

        let result = match interpreter.execute_block(&self.body, environment) {
            ExecutionResult::Success => Ok(Value::Literal(Literal::Nil)),
            ExecutionResult::RuntimeError(error) => Err(error),
            ExecutionResult::Break => Ok(Value::Literal(Literal::Nil)),
            ExecutionResult::Return(value) => Ok(value),
        }?;

        if self.is_initializer {
            self.get_bound_this()
        } else {
            Ok(result)
        }
    }

    fn name(&self) -> String {
        self.name
            .as_ref()
            .map(|t| t.lexeme.clone())
            .unwrap_or_else(|| "anonymous".to_string())
    }
}
