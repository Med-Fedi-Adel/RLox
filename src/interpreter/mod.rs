use crate::{
    environment::Environment,
    expr::Expr,
    stmt::Stmt,
    token::{Literal, Token, TokenType},
};

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) -> Result<(), RuntimeError> {
        for statement in statements {
            self.execute(statement)?;
        }

        Ok(())
    }

    fn execute(&mut self, statement: &Stmt) -> Result<(), RuntimeError> {
        match statement {
            Stmt::Expression { expression } => {
                self.evaluate(expression)?;
                Ok(())
            }

            Stmt::Print { expression } => {
                let value = self.evaluate(expression)?;
                println!("{}", self.stringify(&value));
                Ok(())
            }

            Stmt::Var { name, initializer } => {
                let value = match initializer {
                    Some(expression) => self.evaluate(expression)?,
                    None => Literal::Nil,
                };

                self.environment.define(name.lexeme.clone(), value);

                Ok(())
            }
        }
    }

    fn evaluate(&mut self, expression: &Expr) -> Result<Literal, RuntimeError> {
        match expression {
            Expr::Literal { value } => Ok(value.clone()),

            Expr::Grouping { expression } => self.evaluate(expression),

            Expr::Variable { name } => self.environment.get(name),

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

    fn stringify(&self, value: &Literal) -> String {
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

#[cfg(test)]
mod tests;
