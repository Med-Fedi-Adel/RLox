use std::rc::Rc;

use crate::{
    environment::{self, Environment, EnvironmentRef},
    expr::Expr,
    interpreter::{self, lox_function::LoxFunction},
    stmt::Stmt,
    token::{Literal, Token, TokenType, Value},
};

pub struct Interpreter {
    globals: EnvironmentRef,
    environment: EnvironmentRef,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Environment::new();

        globals.borrow_mut().define(
            "clock".to_string(),
            Some(Value::Callable(Rc::new(NativeClock))),
        );

        Self {
            globals: globals.clone(),
            environment: globals,
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) -> ExecutionResult {
        for statement in statements {
            match self.execute(statement) {
                ExecutionResult::Success => {}
                result => return result,
            }
        }

        ExecutionResult::Success
    }

    pub fn evaluate_expression(&mut self, expression: &Expr) -> Result<Value, RuntimeError> {
        self.evaluate(expression)
    }

    fn execute(&mut self, statement: &Stmt) -> ExecutionResult {
        match statement {
            Stmt::Expression { expression } => match self.evaluate(expression) {
                Ok(_) => ExecutionResult::Success,
                Err(error) => ExecutionResult::RuntimeError(error),
            },

            Stmt::Print { expression } => match self.evaluate(expression) {
                Ok(value) => {
                    println!("{}", self.stringify(&value));
                    ExecutionResult::Success
                }
                Err(error) => ExecutionResult::RuntimeError(error),
            },
            Stmt::Var { name, initializer } => {
                let value = match initializer {
                    Some(expression) => match self.evaluate(expression) {
                        Ok(value) => Some(value),
                        Err(error) => return ExecutionResult::RuntimeError(error),
                    },
                    None => None,
                };

                self.environment
                    .borrow_mut()
                    .define(name.lexeme.clone(), value);

                ExecutionResult::Success
            }

            Stmt::Block { statements } => {
                let environment = Environment::from(self.environment.clone());

                self.execute_block(statements, environment)
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = match self.evaluate(condition) {
                    Ok(value) => value,
                    Err(error) => return ExecutionResult::RuntimeError(error),
                };

                if self.is_truthy(&condition) {
                    self.execute(then_branch)
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch)
                } else {
                    ExecutionResult::Success
                }
            }

            Stmt::While { condition, body } => {
                loop {
                    let condition_value = match self.evaluate(condition) {
                        Ok(value) => value,
                        Err(error) => return ExecutionResult::RuntimeError(error),
                    };

                    if !self.is_truthy(&condition_value) {
                        break;
                    }

                    match self.execute(body) {
                        ExecutionResult::Success => {}
                        ExecutionResult::Break => break,
                        ExecutionResult::RuntimeError(error) => {
                            return ExecutionResult::RuntimeError(error);
                        }
                        result @ ExecutionResult::Return(_) => return result,
                    }
                }

                ExecutionResult::Success
            }

            Stmt::Break => ExecutionResult::Break,

            Stmt::Function {
                name,
                parameters,
                body,
            } => {
                let function = LoxFunction::new(
                    name.clone(),
                    parameters.clone(),
                    body.clone(),
                    self.environment.clone(),
                );

                self.environment.borrow_mut().define(
                    name.lexeme.clone(),
                    Some(Value::Callable(Rc::new(function))),
                );

                ExecutionResult::Success
            }

            Stmt::Return { keyword, value } => {
                let return_value = match value {
                    Some(expression) => match self.evaluate(expression) {
                        Ok(value) => value,
                        Err(error) => return ExecutionResult::RuntimeError(error),
                    },
                    None => Value::Literal(Literal::Nil),
                };

                ExecutionResult::Return(return_value)
            }
        }
    }

    fn execute_block(
        &mut self,
        statements: &[Stmt],
        environment: EnvironmentRef,
    ) -> ExecutionResult {
        let previous = self.environment.clone();

        self.environment = environment;

        let result = (|| {
            for statement in statements {
                match self.execute(statement) {
                    ExecutionResult::Success => {}
                    result => return result,
                }
            }

            ExecutionResult::Success
        })();

        self.environment = previous;

        result
    }

    fn evaluate(&mut self, expression: &Expr) -> Result<Value, RuntimeError> {
        match expression {
            Expr::Literal { value } => Ok(Value::Literal(value.clone())),

            Expr::Grouping { expression } => self.evaluate(expression),

            Expr::Variable { name } => self.environment.borrow().get(name),

            Expr::Assign { name, value } => {
                let value = self.evaluate(value)?;

                self.environment.borrow_mut().assign(name, value.clone())?;

                Ok(value)
            }

            Expr::Unary { operator, right } => {
                let right = self.evaluate(right)?;

                match operator.token_type {
                    TokenType::Minus => match right {
                        Value::Literal(Literal::Number(value)) => {
                            Ok(Value::Literal(Literal::Number(-value)))
                        }

                        _ => Err(RuntimeError::new(
                            operator.clone(),
                            "Operand must be a number.",
                        )),
                    },

                    TokenType::Bang => {
                        Ok(Value::Literal(Literal::Boolean(!self.is_truthy(&right))))
                    }

                    _ => unreachable!("Invalid unary operator"),
                }
            }

            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;

                self.evaluate_binary(&left, operator, &right)
            }

            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;

                if operator.token_type == TokenType::Or {
                    if self.is_truthy(&left) {
                        return Ok(left);
                    }
                } else {
                    if !self.is_truthy(&left) {
                        return Ok(left);
                    }
                }

                self.evaluate(right)
            }

            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let callee = self.evaluate(callee)?;

                let mut arguments_values = Vec::new();

                for argument in arguments {
                    arguments_values.push(self.evaluate(argument)?);
                }

                match callee {
                    Value::Callable(function) => {
                        if arguments_values.len() != function.arity() {
                            return Err(RuntimeError::new(
                                paren.clone(),
                                format!(
                                    "Expected {} arguments but got {}.",
                                    function.arity(),
                                    arguments_values.len()
                                ),
                            ));
                        }

                        function.call(self, arguments_values)
                    }

                    _ => Err(RuntimeError::new(
                        paren.clone(),
                        "Can only call functions and classes.",
                    )),
                }
            }
        }
    }

    fn evaluate_binary(
        &self,
        left: &Value,
        operator: &Token,
        right: &Value,
    ) -> Result<Value, RuntimeError> {
        match operator.token_type {
            TokenType::Minus => self.number_binary(left, operator, right, |a, b| a - b),

            TokenType::Slash => {
                match right {
                    Value::Literal(Literal::Number(value)) => {
                        if *value == 0.0 {
                            return Err(RuntimeError::new(operator.clone(), "Division by zero."));
                        }
                    }
                    _ => {
                        return Err(RuntimeError::new(
                            operator.clone(),
                            "Operands must be numbers.",
                        ));
                    }
                };

                self.number_binary(left, operator, right, |a, b| a / b)
            }

            TokenType::Star => self.number_binary(left, operator, right, |a, b| a * b),

            TokenType::Plus => self.add(left, right, operator),

            TokenType::Greater => self.number_comparison(left, operator, right, |a, b| a > b),

            TokenType::GreaterEqual => self.number_comparison(left, operator, right, |a, b| a >= b),

            TokenType::Less => self.number_comparison(left, operator, right, |a, b| a < b),

            TokenType::LessEqual => self.number_comparison(left, operator, right, |a, b| a <= b),

            TokenType::EqualEqual => {
                Ok(Value::Literal(Literal::Boolean(self.is_equal(left, right))))
            }

            TokenType::BangEqual => Ok(Value::Literal(Literal::Boolean(
                !self.is_equal(left, right),
            ))),

            _ => unreachable!("Invalid binary operator"),
        }
    }

    fn add(&self, left: &Value, right: &Value, operator: &Token) -> Result<Value, RuntimeError> {
        match (left, right) {
            // Number + Number
            (Value::Literal(Literal::Number(left)), Value::Literal(Literal::Number(right))) => {
                Ok(Value::Literal(Literal::Number(left + right)))
            }

            // String + String
            (Value::Literal(Literal::String(left)), Value::Literal(Literal::String(right))) => Ok(
                Value::Literal(Literal::String(format!("{}{}", left, right))),
            ),

            // String + anything
            (Value::Literal(Literal::String(left)), right) => Ok(Value::Literal(Literal::String(
                format!("{}{}", left, self.stringify(right)),
            ))),

            // Anything + String
            (left, Value::Literal(Literal::String(right))) => Ok(Value::Literal(Literal::String(
                format!("{}{}", self.stringify(left), right),
            ))),

            _ => Err(RuntimeError::new(
                operator.clone(),
                "Operands must be two numbers or at least one string.",
            )),
        }
    }

    fn number_binary<F>(
        &self,
        left: &Value,
        operator: &Token,
        right: &Value,
        operation: F,
    ) -> Result<Value, RuntimeError>
    where
        F: Fn(f64, f64) -> f64,
    {
        match (left, right) {
            (Value::Literal(Literal::Number(a)), Value::Literal(Literal::Number(b))) => {
                Ok(Value::Literal(Literal::Number(operation(*a, *b))))
            }

            _ => Err(RuntimeError::new(
                operator.clone(),
                "Operands must be numbers.",
            )),
        }
    }

    fn number_comparison<F>(
        &self,
        left: &Value,
        operator: &Token,
        right: &Value,
        operation: F,
    ) -> Result<Value, RuntimeError>
    where
        F: Fn(f64, f64) -> bool,
    {
        match (left, right) {
            (Value::Literal(Literal::Number(a)), Value::Literal(Literal::Number(b))) => {
                Ok(Value::Literal(Literal::Boolean(operation(*a, *b))))
            }

            _ => Err(RuntimeError::new(
                operator.clone(),
                "Operands must be numbers.",
            )),
        }
    }

    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Literal(Literal::Nil) => false,
            Value::Literal(Literal::Boolean(value)) => *value,
            _ => true,
        }
    }

    fn is_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Literal(Literal::Nil), Value::Literal(Literal::Nil)) => true,

            (Value::Literal(Literal::Nil), _) => false,

            (_, Value::Literal(Literal::Nil)) => false,

            (Value::Literal(left), Value::Literal(right)) => left == right,

            _ => false,
        }
    }

    pub fn stringify(&self, value: &Value) -> String {
        match value {
            Value::Literal(Literal::Number(value)) => {
                if value.fract() == 0.0 {
                    format!("{:.0}", value)
                } else {
                    value.to_string()
                }
            }

            Value::Literal(Literal::String(value)) => value.clone(),

            Value::Literal(Literal::Boolean(value)) => value.to_string(),

            Value::Literal(Literal::Nil) => "nil".to_string(),

            Value::Callable(callable) => format!("<fn {}>", callable.name()),
        }
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

impl RuntimeError {
    pub fn new(token: Token, message: impl Into<String>) -> Self {
        Self {
            token,
            message: message.into(),
        }
    }
}

pub enum ExecutionResult {
    Success,
    RuntimeError(RuntimeError),
    Break,
    Return(Value),
}

pub trait LoxCallable {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError>;

    fn arity(&self) -> usize;

    fn name(&self) -> String;
}

struct NativeClock;

impl LoxCallable for NativeClock {
    fn arity(&self) -> usize {
        0
    }

    fn call(
        &self,
        _interpreter: &mut Interpreter,
        _arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        Ok(Value::Literal(Literal::Number(time)))
    }

    fn name(&self) -> String {
        "clock".to_string()
    }
}

#[cfg(test)]
mod tests;

mod lox_function;
