use super::{AstPrinter, Expr};
use crate::token::{Literal, Token, TokenType};

fn token(token_type: TokenType, lexeme: &str) -> Token {
    Token::new(token_type, lexeme.to_string(), None, 1)
}

#[test]
fn prints_literal() {
    let expr = Expr::Literal {
        value: Literal::Number(123.0),
    };

    assert_eq!(AstPrinter::new().print(&expr), "123");
}

#[test]
fn prints_unary() {
    let expr = Expr::Unary {
        operator: token(TokenType::Minus, "-"),
        right: Box::new(Expr::Literal {
            value: Literal::Number(123.0),
        }),
    };

    assert_eq!(AstPrinter::new().print(&expr), "(- 123)");
}

#[test]
fn prints_binary() {
    let expr = Expr::Binary {
        left: Box::new(Expr::Literal {
            value: Literal::Number(1.0),
        }),
        operator: token(TokenType::Plus, "+"),
        right: Box::new(Expr::Literal {
            value: Literal::Number(2.0),
        }),
    };

    assert_eq!(AstPrinter::new().print(&expr), "(+ 1 2)");
}

#[test]
fn prints_grouping() {
    let expr = Expr::Grouping {
        expression: Box::new(Expr::Literal {
            value: Literal::Number(123.0),
        }),
    };

    assert_eq!(AstPrinter::new().print(&expr), "(group 123)");
}

#[test]
fn prints_complex_expression() {
    let expr = Expr::Binary {
        left: Box::new(Expr::Unary {
            operator: token(TokenType::Minus, "-"),
            right: Box::new(Expr::Literal {
                value: Literal::Number(123.0),
            }),
        }),
        operator: token(TokenType::Star, "*"),
        right: Box::new(Expr::Grouping {
            expression: Box::new(Expr::Literal {
                value: Literal::Number(45.67),
            }),
        }),
    };

    assert_eq!(AstPrinter::new().print(&expr), "(* (- 123) (group 45.67))");
}
