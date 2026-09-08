use std::rc::Rc;

use crate::{expr::Expr, token::Token};

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Expression {
        expression: Expr,
    },

    Print {
        expression: Expr,
    },

    Var {
        name: Token,
        initializer: Option<Expr>,
    },

    Block {
        statements: Vec<Stmt>,
    },

    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },

    While {
        condition: Expr,
        body: Box<Stmt>,
    },

    Break,

    Function {
        name: Token,
        parameters: Rc<Vec<Token>>,
        body: Rc<Vec<Stmt>>,
    },

    Return {
        keyword: Token,
        value: Option<Expr>,
    },
}
