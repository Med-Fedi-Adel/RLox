use crate::{
    expr::{AstPrinter, Expr},
    lox::Lox,
    parser::Parser,
    scanner::Scanner,
    stmt::Stmt,
    token::{Literal, Token, TokenType},
};

fn parse(source: &str) -> Vec<Stmt> {
    let mut lox = Lox::new();

    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();

    let mut parser = Parser::new(tokens, &mut lox);

    parser.parse()
}

fn parse_expression(source: &str) -> Expr {
    let statements = parse(source);

    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Expression { expression } => expression.clone(),

        _ => panic!("Expected an expression statement"),
    }
}

fn token(token_type: TokenType, lexeme: &str) -> Token {
    Token::new(token_type, lexeme.to_string(), None, 1)
}

#[test]
fn parses_number_literal() {
    let expr = parse_expression("123;");

    assert_eq!(
        expr,
        Expr::Literal {
            value: Literal::Number(123.0)
        }
    );
}

#[test]
fn parses_string_literal() {
    let expr = parse_expression("\"hello\";");

    assert_eq!(
        expr,
        Expr::Literal {
            value: Literal::String("hello".to_string())
        }
    );
}

#[test]
fn parses_true() {
    let expr = parse_expression("true;");

    assert_eq!(
        expr,
        Expr::Literal {
            value: Literal::Boolean(true)
        }
    );
}

#[test]
fn parses_false() {
    let expr = parse_expression("false;");

    assert_eq!(
        expr,
        Expr::Literal {
            value: Literal::Boolean(false)
        }
    );
}

#[test]
fn parses_nil() {
    let expr = parse_expression("nil;");

    assert_eq!(
        expr,
        Expr::Literal {
            value: Literal::Nil
        }
    );
}

#[test]
fn parses_unary_expression() {
    let expr = parse_expression("-123;");

    assert_eq!(
        expr,
        Expr::Unary {
            operator: token(TokenType::Minus, "-"),
            right: Box::new(Expr::Literal {
                value: Literal::Number(123.0)
            }),
        }
    );
}

#[test]
fn parses_binary_expression() {
    let expr = parse_expression("1 + 2;");

    assert_eq!(
        expr,
        Expr::Binary {
            left: Box::new(Expr::Literal {
                value: Literal::Number(1.0)
            }),
            operator: token(TokenType::Plus, "+"),
            right: Box::new(Expr::Literal {
                value: Literal::Number(2.0)
            }),
        }
    );
}

#[test]
fn respects_multiplication_precedence() {
    let expr = parse_expression("1 + 2 * 3;");

    assert_eq!(
        expr,
        Expr::Binary {
            left: Box::new(Expr::Literal {
                value: Literal::Number(1.0)
            }),
            operator: token(TokenType::Plus, "+"),
            right: Box::new(Expr::Binary {
                left: Box::new(Expr::Literal {
                    value: Literal::Number(2.0)
                }),
                operator: token(TokenType::Star, "*"),
                right: Box::new(Expr::Literal {
                    value: Literal::Number(3.0)
                }),
            }),
        }
    );
}

#[test]
fn respects_parentheses() {
    let expr = parse_expression("(1 + 2) * 3;");

    assert_eq!(
        expr,
        Expr::Binary {
            left: Box::new(Expr::Grouping {
                expression: Box::new(Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: Literal::Number(1.0)
                    }),
                    operator: token(TokenType::Plus, "+"),
                    right: Box::new(Expr::Literal {
                        value: Literal::Number(2.0)
                    }),
                }),
            }),
            operator: token(TokenType::Star, "*"),
            right: Box::new(Expr::Literal {
                value: Literal::Number(3.0)
            }),
        }
    );
}

#[test]
fn unary_has_higher_precedence_than_binary() {
    let expr = parse_expression("-1 + 2;");

    assert_eq!(
        expr,
        Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: token(TokenType::Minus, "-"),
                right: Box::new(Expr::Literal {
                    value: Literal::Number(1.0)
                }),
            }),
            operator: token(TokenType::Plus, "+"),
            right: Box::new(Expr::Literal {
                value: Literal::Number(2.0)
            }),
        }
    );
}

#[test]
fn binary_operators_are_left_associative() {
    let expr = parse_expression("1 - 2 - 3;");

    assert_eq!(
        expr,
        Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Expr::Literal {
                    value: Literal::Number(1.0)
                }),
                operator: token(TokenType::Minus, "-"),
                right: Box::new(Expr::Literal {
                    value: Literal::Number(2.0)
                }),
            }),
            operator: token(TokenType::Minus, "-"),
            right: Box::new(Expr::Literal {
                value: Literal::Number(3.0)
            }),
        }
    );
}

#[test]
fn parses_equality() {
    let expr = parse_expression("1 == 2;");

    assert_eq!(
        expr,
        Expr::Binary {
            left: Box::new(Expr::Literal {
                value: Literal::Number(1.0)
            }),
            operator: token(TokenType::EqualEqual, "=="),
            right: Box::new(Expr::Literal {
                value: Literal::Number(2.0)
            }),
        }
    );
}

#[test]
fn parses_comparison() {
    let expr = parse_expression("1 < 2;");

    assert_eq!(
        expr,
        Expr::Binary {
            left: Box::new(Expr::Literal {
                value: Literal::Number(1.0)
            }),
            operator: token(TokenType::Less, "<"),
            right: Box::new(Expr::Literal {
                value: Literal::Number(2.0)
            }),
        }
    );
}

#[test]
fn respects_comparison_over_equality() {
    let expr = parse_expression("1 < 2 == true;");

    assert_eq!(
        expr,
        Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Expr::Literal {
                    value: Literal::Number(1.0)
                }),
                operator: token(TokenType::Less, "<"),
                right: Box::new(Expr::Literal {
                    value: Literal::Number(2.0)
                }),
            }),
            operator: token(TokenType::EqualEqual, "=="),
            right: Box::new(Expr::Literal {
                value: Literal::Boolean(true)
            }),
        }
    );
}

#[test]
fn parses_nested_expression() {
    let expr = parse_expression("-123 * (45.67);");

    assert_eq!(
        expr,
        Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: token(TokenType::Minus, "-"),
                right: Box::new(Expr::Literal {
                    value: Literal::Number(123.0)
                }),
            }),
            operator: token(TokenType::Star, "*"),
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
    let statements = parse("+;");

    assert!(statements.is_empty());
}

#[test]
fn rejects_missing_closing_parenthesis() {
    let statements = parse("(1 + 2;");

    assert!(statements.is_empty());
}

#[test]
fn rejects_empty_parentheses() {
    let statements = parse("();");

    assert!(statements.is_empty());
}

#[test]
fn rejects_invalid_operator_sequence() {
    let statements = parse("1 + * 2;");

    assert!(statements.is_empty());
}

#[test]
fn respects_all_precedence_levels() {
    let expr = parse_expression("1 + 2 * 3 == 7;");

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
    let source = "-123 * (45.67);";

    let mut lox = Lox::new();

    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();

    let mut parser = Parser::new(tokens, &mut lox);
    let statements = parser.parse();

    assert_eq!(statements.len(), 1);

    let expression = match &statements[0] {
        Stmt::Expression { expression } => expression,
        _ => panic!("Expected an expression statement"),
    };

    let output = AstPrinter::new().print(expression);

    assert_eq!(output, "(* (- 123) (group 45.67))");
}

#[test]
fn parses_expression_statement() {
    let statements = parse("1 + 2;");

    assert_eq!(statements.len(), 1);

    assert!(matches!(&statements[0], Stmt::Expression { .. }));
}

#[test]
fn parses_print_statement() {
    let statements = parse("print 123;");

    assert_eq!(statements.len(), 1);

    assert!(matches!(&statements[0], Stmt::Print { .. }));
}

#[test]
fn parses_multiple_statements() {
    let statements = parse(
        r#"
        print "one";
        print true;
        2 + 1;
        "#,
    );

    assert_eq!(statements.len(), 3);
}

#[test]
fn requires_semicolon_after_expression() {
    let statements = parse("123");

    assert!(statements.is_empty());
}

#[test]
fn requires_semicolon_after_print() {
    let statements = parse("print 123");

    assert!(statements.is_empty());
}

#[test]
fn parses_variable_declaration() {
    let statements = parse("var beverage = \"espresso\";");

    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Var { name, initializer } => {
            assert_eq!(name.lexeme, "beverage");

            assert_eq!(
                initializer,
                &Some(Expr::Literal {
                    value: Literal::String("espresso".to_string())
                })
            );
        }

        _ => panic!("Expected variable declaration"),
    }
}

#[test]
fn parses_variable_without_initializer() {
    let statements = parse("var beverage;");

    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Var { name, initializer } => {
            assert_eq!(name.lexeme, "beverage");
            assert_eq!(initializer, &None);
        }

        _ => panic!("Expected variable declaration"),
    }
}

#[test]
fn parses_variable_expression() {
    let statements = parse("beverage;");

    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Expression {
            expression: Expr::Variable { name, .. },
        } => {
            assert_eq!(name.lexeme, "beverage");
        }

        _ => panic!("Expected variable expression"),
    }
}

#[test]
fn parses_class_declaration() {
    let statements = parse(
        r#"
        class Breakfast {
            cook() {
                print "Eggs a-fryin'!";
            }

            serve(who) {
                print who;
            }
        }
        "#,
    );

    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::Class { name, methods } => {
            assert_eq!(name.lexeme, "Breakfast");
            assert_eq!(methods.len(), 2);

            for method in methods {
                assert!(matches!(method, Stmt::Function { .. }));
            }
        }

        _ => panic!("Expected class declaration"),
    }
}
