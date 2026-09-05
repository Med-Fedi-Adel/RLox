use crate::{
    environment::{self, Environment, EnvironmentRef},
    expr::Expr,
    interpreter,
    stmt::Stmt,
    token::{Literal, Token, TokenType, Value},
};

pub struct Interpreter {
    environment: EnvironmentRef,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
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
                    }
                }

                ExecutionResult::Success
            }
            Stmt::Break => ExecutionResult::Break,
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
            Expr::Literal { value } => Ok(value.clone()),

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
                        Literal::Number(value) => Ok(Literal::Number(-value)),

                        _ => Err(RuntimeError::new(
                            operator.clone(),
                            "Operand must be a number.",
                        )),
                    },

                    TokenType::Bang => Ok(Literal::Boolean(!self.is_truthy(&right))),

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
                todo!()
            }
        }
    }

    fn evaluate_binary(
        &self,
        left: &Literal,
        operator: &Token,
        right: &Literal,
    ) -> Result<Literal, RuntimeError> {
        match operator.token_type {
            TokenType::Minus => self.number_binary(left, operator, right, |a, b| a - b),

            TokenType::Slash => {
                match right {
                    Literal::Number(value) => {
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

            TokenType::EqualEqual => Ok(Literal::Boolean(self.is_equal(left, right))),

            TokenType::BangEqual => Ok(Literal::Boolean(!self.is_equal(left, right))),

            _ => unreachable!("Invalid binary operator"),
        }
    }

    fn add(
        &self,
        left: &Literal,
        right: &Literal,
        operator: &Token,
    ) -> Result<Literal, RuntimeError> {
        match (left, right) {
            // Number + Number
            (Literal::Number(left), Literal::Number(right)) => Ok(Literal::Number(left + right)),

            // String + String
            (Literal::String(left), Literal::String(right)) => {
                Ok(Literal::String(format!("{}{}", left, right)))
            }

            // String + anything
            (Literal::String(left), right) => Ok(Literal::String(format!(
                "{}{}",
                left,
                self.stringify(right)
            ))),

            // Anything + String
            (left, Literal::String(right)) => Ok(Literal::String(format!(
                "{}{}",
                self.stringify(left),
                right
            ))),

            _ => Err(RuntimeError::new(
                operator.clone(),
                "Operands must be two numbers or at least one string.",
            )),
        }
    }

    fn number_binary<F>(
        &self,
        left: &Literal,
        operator: &Token,
        right: &Literal,
        operation: F,
    ) -> Result<Literal, RuntimeError>
    where
        F: Fn(f64, f64) -> f64,
    {
        match (left, right) {
            (Literal::Number(a), Literal::Number(b)) => Ok(Literal::Number(operation(*a, *b))),

            _ => Err(RuntimeError::new(
                operator.clone(),
                "Operands must be numbers.",
            )),
        }
    }

    fn number_comparison<F>(
        &self,
        left: &Literal,
        operator: &Token,
        right: &Literal,
        operation: F,
    ) -> Result<Literal, RuntimeError>
    where
        F: Fn(f64, f64) -> bool,
    {
        match (left, right) {
            (Literal::Number(a), Literal::Number(b)) => Ok(Literal::Boolean(operation(*a, *b))),

            _ => Err(RuntimeError::new(
                operator.clone(),
                "Operands must be numbers.",
            )),
        }
    }

    fn is_truthy(&self, value: &Literal) -> bool {
        match value {
            Literal::Nil => false,
            Literal::Boolean(value) => *value,
            _ => true,
        }
    }

    fn is_equal(&self, left: &Literal, right: &Literal) -> bool {
        left == right
    }

    pub fn stringify(&self, value: &Literal) -> String {
        match value {
            Literal::Number(value) => {
                if value.fract() == 0.0 {
                    format!("{:.0}", value)
                } else {
                    value.to_string()
                }
            }

            Literal::String(value) => value.clone(),

            Literal::Boolean(value) => value.to_string(),

            Literal::Nil => "nil".to_string(),
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
}

pub trait LoxCallable {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Literal>,
    ) -> Result<Literal, RuntimeError>;

    fn arity(&self) -> usize;
}

#[cfg(test)]
mod tests;
