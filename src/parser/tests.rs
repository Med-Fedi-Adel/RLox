use crate::{
    expr::{AstPrinter, Expr},
    lox::Lox,
    parser::Parser,
    scanner::Scanner,
    token::{Literal, Token, TokenType},
};

fn parse(source: &str) -> Option<Expr> {
    let mut lox = Lox::new();

    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();

    let mut parser = Parser::new(tokens, &mut lox);

    parser.parse()
}

fn token(token_type: TokenType, lexeme: &str) -> Token {
    Token::new(token_type, lexeme.to_string(), None, 1)
}

#[test]
fn parses_number_literal() {
    let expr = parse("123").unwrap();

    assert_eq!(
        expr,
        Expr::Literal {
            value: Literal::Number(123.0)
        }
    );
}

#[test]
fn parses_string_literal() {
    let expr = parse("\"hello\"").unwrap();

    assert_eq!(
        expr,
        Expr::Literal {
            value: Literal::String("hello".to_string())
        }
    );
}

#[test]
fn parses_true() {
    let expr = parse("true").unwrap();

    assert_eq!(
        expr,
        Expr::Literal {
            value: Literal::Boolean(true)
        }
    );
}

#[test]
fn parses_false() {
    let expr = parse("false").unwrap();

    assert_eq!(
        expr,
        Expr::Literal {
            value: Literal::Boolean(false)
        }
    );
}

#[test]
fn parses_nil() {
    let expr = parse("nil").unwrap();

    assert_eq!(
        expr,
        Expr::Literal {
            value: Literal::Nil
        }
    );
}

#[test]
fn parses_unary_expression() {
    let expr = parse("-123").unwrap();

    assert_eq!(
        expr,
        Expr::Unary {
            operator: crate::token::Token::new(TokenType::Minus, "-".to_string(), None, 1),
            right: Box::new(Expr::Literal {
                value: Literal::Number(123.0)
            }),
        }
    );
}

#[test]
fn parses_binary_expression() {
    let expr = parse("1 + 2").unwrap();

    assert_eq!(
        expr,
        Expr::Binary {
            left: Box::new(Expr::Literal {
                value: Literal::Number(1.0)
            }),
            operator: crate::token::Token::new(TokenType::Plus, "+".to_string(), None, 1),
            right: Box::new(Expr::Literal {
                value: Literal::Number(2.0)
            }),
        }
    );
}

#[test]
fn respects_multiplication_precedence() {
    let expr = parse("1 + 2 * 3").unwrap();

    assert_eq!(
        expr,
        Expr::Binary {
            left: Box::new(Expr::Literal {
                value: Literal::Number(1.0)
            }),
            operator: crate::token::Token::new(TokenType::Plus, "+".to_string(), None, 1),
            right: Box::new(Expr::Binary {
                left: Box::new(Expr::Literal {
                    value: Literal::Number(2.0)
                }),
                operator: crate::token::Token::new(TokenType::Star, "*".to_string(), None, 1),
                right: Box::new(Expr::Literal {
                    value: Literal::Number(3.0)
                }),
            }),
        }
    );
}

#[test]
fn respects_parentheses() {
    let expr = parse("(1 + 2) * 3").unwrap();

    assert_eq!(
        expr,
        Expr::Binary {
            left: Box::new(Expr::Grouping {
                expression: Box::new(Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: Literal::Number(1.0)
                    }),
                    operator: crate::token::Token::new(TokenType::Plus, "+".to_string(), None, 1),
                    right: Box::new(Expr::Literal {
                        value: Literal::Number(2.0)
                    }),
                }),
            }),
            operator: crate::token::Token::new(TokenType::Star, "*".to_string(), None, 1),
            right: Box::new(Expr::Literal {
                value: Literal::Number(3.0)
            }),
        }
    );
}

#[test]
fn unary_has_higher_precedence_than_binary() {
    let expr = parse("-1 + 2").unwrap();

    assert_eq!(
        expr,
        Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: crate::token::Token::new(TokenType::Minus, "-".to_string(), None, 1),
                right: Box::new(Expr::Literal {
                    value: Literal::Number(1.0)
                }),
            }),
            operator: crate::token::Token::new(TokenType::Plus, "+".to_string(), None, 1),
            right: Box::new(Expr::Literal {
                value: Literal::Number(2.0)
            }),
        }
    );
}

#[test]
fn binary_operators_are_left_associative() {
    let expr = parse("1 - 2 - 3").unwrap();

    assert_eq!(
        expr,
        Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Expr::Literal {
                    value: Literal::Number(1.0)
                }),
                operator: crate::token::Token::new(TokenType::Minus, "-".to_string(), None, 1),
                right: Box::new(Expr::Literal {
                    value: Literal::Number(2.0)
                }),
            }),
            operator: crate::token::Token::new(TokenType::Minus, "-".to_string(), None, 1),
            right: Box::new(Expr::Literal {
                value: Literal::Number(3.0)
            }),
        }
    );
}

#[test]
fn parses_equality() {
    let expr = parse("1 == 2").unwrap();

    assert_eq!(
        expr,
        Expr::Binary {
            left: Box::new(Expr::Literal {
                value: Literal::Number(1.0)
            }),
            operator: crate::token::Token::new(TokenType::EqualEqual, "==".to_string(), None, 1),
            right: Box::new(Expr::Literal {
                value: Literal::Number(2.0)
            }),
        }
    );
}

#[test]
fn parses_comparison() {
    let expr = parse("1 < 2").unwrap();

    assert_eq!(
        expr,
        Expr::Binary {
            left: Box::new(Expr::Literal {
                value: Literal::Number(1.0)
            }),
            operator: crate::token::Token::new(TokenType::Less, "<".to_string(), None, 1),
            right: Box::new(Expr::Literal {
                value: Literal::Number(2.0)
            }),
        }
    );
}

#[test]
fn respects_comparison_over_equality() {
    let expr = parse("1 < 2 == true").unwrap();

    assert_eq!(
        expr,
        Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Expr::Literal {
                    value: Literal::Number(1.0)
                }),
                operator: crate::token::Token::new(TokenType::Less, "<".to_string(), None, 1),
                right: Box::new(Expr::Literal {
                    value: Literal::Number(2.0)
                }),
            }),
            operator: crate::token::Token::new(TokenType::EqualEqual, "==".to_string(), None, 1),
            right: Box::new(Expr::Literal {
                value: Literal::Boolean(true)
            }),
        }
    );
}

#[test]
fn parses_nested_expression() {
    let expr = parse("-123 * (45.67)").unwrap();

    assert_eq!(
        expr,
        Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: crate::token::Token::new(TokenType::Minus, "-".to_string(), None, 1),
                right: Box::new(Expr::Literal {
                    value: Literal::Number(123.0)
                }),
            }),
            operator: crate::token::Token::new(TokenType::Star, "*".to_string(), None, 1),
            right: Box::new(Expr::Grouping {
                expression: Box::new(Expr::Literal {
                    value: Literal::Number(45.67)
                }),
            }),
        }
    );
}

#[test]
fn rejects_missing_expression() {
    assert!(parse("+").is_none());
}

#[test]
fn rejects_missing_closing_parenthesis() {
    assert!(parse("(1 + 2").is_none());
}

#[test]
fn rejects_empty_parentheses() {
    assert!(parse("()").is_none());
}

#[test]
fn rejects_invalid_operator_sequence() {
    assert!(parse("1 + * 2").is_none());
}

#[test]
fn respects_all_precedence_levels() {
    let expr = parse("1 + 2 * 3 == 7").unwrap();

    assert_eq!(
        expr,
        Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Expr::Literal {
                    value: Literal::Number(1.0),
                }),
                operator: token(TokenType::Plus, "+"),
                right: Box::new(Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: Literal::Number(2.0),
                    }),
                    operator: token(TokenType::Star, "*"),
                    right: Box::new(Expr::Literal {
                        value: Literal::Number(3.0),
                    }),
                }),
            }),
            operator: token(TokenType::EqualEqual, "=="),
            right: Box::new(Expr::Literal {
                value: Literal::Number(7.0),
            }),
        }
    );
}

#[test]
fn scanner_parser_printer_integration() {
    let source = "-123 * (45.67)";

    let mut lox = Lox::new();

    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();

    let mut parser = Parser::new(tokens, &mut lox);
    let expression = parser.parse().unwrap();

    let output = AstPrinter::new().print(&expression);

    assert_eq!(output, "(* (- 123) (group 45.67))");
}
