use std::rc::Rc;

use crate::{expr::Expr, token::Token};

#[derive(Clone, Debug, PartialEq)]
pub enum ClassMember {
    Method {
        name: Token,
        parameters: Rc<Vec<Token>>,
        body: Rc<Vec<Stmt>>,
    },

    StaticMethod {
        name: Token,
        parameters: Rc<Vec<Token>>,
        body: Rc<Vec<Stmt>>,
    },

    Getter {
        name: Token,
        body: Rc<Vec<Stmt>>,
    },
}

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

    Class {
        name: Token,
        members: Vec<ClassMember>,
    },

    Return {
        keyword: Token,
        value: Option<Expr>,
    },
}
