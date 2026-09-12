use std::rc::Rc;

use crate::{
    expr_id::ExprId,
    stmt::Stmt,
    token::{Literal, Token},
};

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

    Variable {
        id: ExprId,
        name: Token,
    },

    Assign {
        id: ExprId,
        name: Token,
        value: Box<Expr>,
    },

    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },

    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },

    Get {
        object: Box<Expr>,
        name: Token,
    },

    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },

    Function {
        params: Rc<Vec<Token>>,
        body: Rc<Vec<Stmt>>,
    },
}

#[cfg(test)]
mod tests;
