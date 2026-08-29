use crate::{expr::Expr, token::Token};

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
}
