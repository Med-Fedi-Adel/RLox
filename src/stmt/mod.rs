use crate::expr::Expr;

pub enum Stmt {
    Expression { expression: Expr },

    Print { expression: Expr },
}
