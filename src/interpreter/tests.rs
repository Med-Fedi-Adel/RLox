use crate::{
    expr::Expr,
    token::{Literal, Token, TokenType},
};

use super::Interpreter;

fn token(token_type: TokenType, lexeme: &str) -> Token {
    Token::new(token_type, lexeme.to_string(), None, 1)
}

fn assert_number(result: Result<Literal, super::RuntimeError>, expected: f64) {
    match result {
        Ok(Literal::Number(n)) => {
            assert!((n - expected).abs() < 1e-9, "expected {expected}, got {n}");
        }
        Ok(other) => panic!("expected Number({expected}), got {:?}", other),
        Err(e) => panic!("expected Ok(Number({expected})), got Err: {}", e.message),
    }
}

fn assert_string(result: Result<Literal, super::RuntimeError>, expected: &str) {
    match result {
        Ok(Literal::String(s)) => assert_eq!(s, expected),
        Ok(other) => panic!("expected String({expected:?}), got {:?}", other),
        Err(e) => panic!("expected Ok(String({expected:?})), got Err: {}", e.message),
    }
}

fn assert_runtime_error(result: Result<Literal, super::RuntimeError>, expected_message: &str) {
    match result {
        Err(e) => assert_eq!(e.message, expected_message),
        Ok(value) => panic!(
            "expected RuntimeError({expected_message:?}), got Ok({:?})",
            value
        ),
    }
}

#[test]
fn addition_of_numbers() {
    let expression = Expr::Binary {
        left: Box::new(Expr::Literal {
            value: Literal::Number(1.0),
        }),
        operator: token(TokenType::Plus, "+"),
        right: Box::new(Expr::Literal {
            value: Literal::Number(2.0),
        }),
    };

    let interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expression);

    assert_number(result, 3.0);
}

#[test]
fn string_concatenation() {
    let expression = Expr::Binary {
        left: Box::new(Expr::Literal {
            value: Literal::String("hello ".to_string()),
        }),
        operator: token(TokenType::Plus, "+"),
        right: Box::new(Expr::Literal {
            value: Literal::String("world".to_string()),
        }),
    };

    let interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expression);

    assert_string(result, "hello world");
}

#[test]
fn string_plus_number_is_allowed() {
    let expression = Expr::Binary {
        left: Box::new(Expr::Literal {
            value: Literal::String("value: ".to_string()),
        }),
        operator: token(TokenType::Plus, "+"),
        right: Box::new(Expr::Literal {
            value: Literal::Number(42.0),
        }),
    };

    let interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expression);

    assert_string(result, "value: 42");
}

#[test]
fn number_plus_string_is_allowed() {
    let expression = Expr::Binary {
        left: Box::new(Expr::Literal {
            value: Literal::Number(42.0),
        }),
        operator: token(TokenType::Plus, "+"),
        right: Box::new(Expr::Literal {
            value: Literal::String(" apples".to_string()),
        }),
    };

    let interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expression);

    assert_string(result, "42 apples");
}

#[test]
fn unary_minus_times_grouping() {
    // -123 * (45.67)
    let expression = Expr::Binary {
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

    let interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expression);

    assert_number(result, -5617.41);
}

#[test]
fn division_by_zero_reports_runtime_error() {
    let expression = Expr::Binary {
        left: Box::new(Expr::Literal {
            value: Literal::Number(10.0),
        }),
        operator: token(TokenType::Slash, "/"),
        right: Box::new(Expr::Literal {
            value: Literal::Number(0.0),
        }),
    };

    let interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expression);

    assert_runtime_error(result, "Division by zero.");
}

#[test]
fn unary_minus_on_non_number_reports_runtime_error() {
    let expression = Expr::Unary {
        operator: token(TokenType::Minus, "-"),
        right: Box::new(Expr::Literal {
            value: Literal::String("not a number".to_string()),
        }),
    };

    let interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expression);

    assert_runtime_error(result, "Operand must be a number.");
}

#[test]
fn bang_negates_truthiness() {
    let expression = Expr::Unary {
        operator: token(TokenType::Bang, "!"),
        right: Box::new(Expr::Literal {
            value: Literal::Boolean(false),
        }),
    };

    let interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expression);

    match result {
        Ok(Literal::Boolean(b)) => assert!(b),
        other => panic!("expected Ok(Boolean(true)), got {:?}", other),
    }
}

#[test]
fn nil_is_falsy() {
    let expression = Expr::Unary {
        operator: token(TokenType::Bang, "!"),
        right: Box::new(Expr::Literal {
            value: Literal::Nil,
        }),
    };

    let interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expression);

    match result {
        Ok(Literal::Boolean(b)) => assert!(b),
        other => panic!("expected Ok(Boolean(true)), got {:?}", other),
    }
}

#[test]
fn equality_compares_values() {
    let expression = Expr::Binary {
        left: Box::new(Expr::Literal {
            value: Literal::Number(1.0),
        }),
        operator: token(TokenType::EqualEqual, "=="),
        right: Box::new(Expr::Literal {
            value: Literal::Number(1.0),
        }),
    };

    let interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expression);

    match result {
        Ok(Literal::Boolean(b)) => assert!(b),
        other => panic!("expected Ok(Boolean(true)), got {:?}", other),
    }
}

#[test]
fn comparison_operands_must_be_numbers() {
    let expression = Expr::Binary {
        left: Box::new(Expr::Literal {
            value: Literal::String("a".to_string()),
        }),
        operator: token(TokenType::Greater, ">"),
        right: Box::new(Expr::Literal {
            value: Literal::Number(1.0),
        }),
    };

    let interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expression);

    assert_runtime_error(result, "Operands must be numbers.");
}
