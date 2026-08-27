use std::error::Error;

use crate::{
    expr::Expr,
    token::{Literal, Token, TokenType},
};

pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Self {
        Self
    }

    pub fn interpret(&self, expression: &Expr) -> Result<(), RuntimeError> {
        let value = self.evaluate(expression)?;

        println!("{}", self.stringify(&value));

        Ok(())
    }

    fn evaluate(&self, expression: &Expr) -> Result<Literal, RuntimeError> {
        match expression {
            Expr::Literal { value } => Ok(value.clone()),

            Expr::Grouping { expression } => self.evaluate(expression),

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

            TokenType::Slash => self.number_binary(left, operator, right, |a, b| a / b),

            TokenType::Star => self.number_binary(left, operator, right, |a, b| a * b),

            TokenType::Plus => match (left, right) {
                (Literal::Number(a), Literal::Number(b)) => Ok(Literal::Number(a + b)),

                (Literal::String(a), Literal::String(b)) => {
                    Ok(Literal::String(format!("{}{}", a, b)))
                }

                _ => Err(RuntimeError::new(
                    operator.clone(),
                    "Operands must be two numbers or two strings.",
                )),
            },

            TokenType::Greater => self.number_comparison(left, operator, right, |a, b| a > b),

            TokenType::GreaterEqual => self.number_comparison(left, operator, right, |a, b| a >= b),

            TokenType::Less => self.number_comparison(left, operator, right, |a, b| a < b),

            TokenType::LessEqual => self.number_comparison(left, operator, right, |a, b| a <= b),

            TokenType::EqualEqual => Ok(Literal::Boolean(self.is_equal(left, right))),

            TokenType::BangEqual => Ok(Literal::Boolean(!self.is_equal(left, right))),

            _ => unreachable!("Invalid binary operator"),
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
                    format!("{:.1}", value)
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
