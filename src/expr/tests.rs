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

#[test]
fn prints_assignment() {
    let expr = Expr::Assign {
        name: token(TokenType::Identifier, "a"),
        value: Box::new(Expr::Literal {
            value: Literal::Number(42.0),
        }),
    };

    assert_eq!(AstPrinter::new().print(&expr), "(= a 42)");
}

#[test]
fn prints_logical_and() {
    let expr = Expr::Logical {
        left: Box::new(Expr::Variable {
            name: token(TokenType::Identifier, "a"),
        }),
        operator: token(TokenType::And, "and"),
        right: Box::new(Expr::Variable {
            name: token(TokenType::Identifier, "b"),
        }),
    };

    assert_eq!(AstPrinter::new().print(&expr), "(and a b)");
}

#[test]
fn prints_logical_or() {
    let expr = Expr::Logical {
        left: Box::new(Expr::Variable {
            name: token(TokenType::Identifier, "a"),
        }),
        operator: token(TokenType::Or, "or"),
        right: Box::new(Expr::Variable {
            name: token(TokenType::Identifier, "b"),
        }),
    };

    assert_eq!(AstPrinter::new().print(&expr), "(or a b)");
}

#[test]
fn prints_nested_logical_expression() {
    let expr = Expr::Logical {
        left: Box::new(Expr::Variable {
            name: token(TokenType::Identifier, "a"),
        }),
        operator: token(TokenType::Or, "or"),
        right: Box::new(Expr::Logical {
            left: Box::new(Expr::Variable {
                name: token(TokenType::Identifier, "b"),
            }),
            operator: token(TokenType::And, "and"),
            right: Box::new(Expr::Variable {
                name: token(TokenType::Identifier, "c"),
            }),
        }),
    };

    assert_eq!(AstPrinter::new().print(&expr), "(or a (and b c))");
}

#[test]
fn prints_call_without_arguments() {
    let expr = Expr::Call {
        callee: Box::new(Expr::Variable {
            name: token(TokenType::Identifier, "foo"),
        }),
        paren: token(TokenType::RightParen, ")"),
        arguments: vec![],
    };

    assert_eq!(AstPrinter::new().print(&expr), "(call foo)");
}

#[test]
fn prints_call_with_arguments() {
    let expr = Expr::Call {
        callee: Box::new(Expr::Variable {
            name: token(TokenType::Identifier, "foo"),
        }),
        paren: token(TokenType::RightParen, ")"),
        arguments: vec![
            Expr::Literal {
                value: Literal::Number(1.0),
            },
            Expr::Literal {
                value: Literal::Number(2.0),
            },
        ],
    };

    assert_eq!(AstPrinter::new().print(&expr), "(call foo 1 2)");
}

#[test]
fn prints_call_with_expression_arguments() {
    let expr = Expr::Call {
        callee: Box::new(Expr::Variable {
            name: token(TokenType::Identifier, "foo"),
        }),
        paren: token(TokenType::RightParen, ")"),
        arguments: vec![
            Expr::Binary {
                left: Box::new(Expr::Literal {
                    value: Literal::Number(1.0),
                }),
                operator: token(TokenType::Plus, "+"),
                right: Box::new(Expr::Literal {
                    value: Literal::Number(2.0),
                }),
            },
            Expr::Unary {
                operator: token(TokenType::Minus, "-"),
                right: Box::new(Expr::Literal {
                    value: Literal::Number(3.0),
                }),
            },
        ],
    };

    assert_eq!(AstPrinter::new().print(&expr), "(call foo (+ 1 2) (- 3))");
}

#[test]
fn prints_nested_calls() {
    let inner_call = Expr::Call {
        callee: Box::new(Expr::Variable {
            name: token(TokenType::Identifier, "foo"),
        }),
        paren: token(TokenType::RightParen, ")"),
        arguments: vec![],
    };

    let expr = Expr::Call {
        callee: Box::new(inner_call),
        paren: token(TokenType::RightParen, ")"),
        arguments: vec![Expr::Literal {
            value: Literal::Number(42.0),
        }],
    };

    assert_eq!(AstPrinter::new().print(&expr), "(call (call foo) 42)");
}
