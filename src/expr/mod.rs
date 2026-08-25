use crate::token::{Literal, Token};

mod ast_printer;

pub use ast_printer::AstPrinter;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },

    Grouping {
        expression: Box<Expr>,
    },

    Literal {
        value: Literal,
    },

    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}

#[cfg(test)]
mod tests;
