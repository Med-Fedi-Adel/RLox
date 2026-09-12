use crate::{
    expr::Expr,
    interpreter::ExecutionResult,
    lox::Lox,
    parser::Parser,
    resolver::Resolver,
    scanner::Scanner,
    token::{Literal, Token, TokenType, Value},
};

use super::Interpreter;

fn interpret(source: &str) -> ExecutionResult {
    let mut lox = Lox::new();
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens, &mut lox);
    let statements = parser.parse();
    let mut interpreter = Interpreter::new();

    let errors = {
        let mut resolver = Resolver::new(&mut interpreter);
        resolver.resolve(&statements);
        resolver.into_errors()
    };

    assert!(errors.is_empty(), "Resolution errors: {errors:?}");
    interpreter.interpret(&statements)
}

fn token(token_type: TokenType, lexeme: &str) -> Token {
    Token::new(token_type, lexeme.to_string(), None, 1)
}

fn assert_interpret_runtime_error(result: ExecutionResult, expected_message: &str) {
    match result {
        ExecutionResult::RuntimeError(error) => {
            assert_eq!(error.message, expected_message)
        }
        ExecutionResult::Success => {
            panic!("expected RuntimeError({expected_message:?}), got Success")
        }
        ExecutionResult::Break => {
            panic!("expected RuntimeError({expected_message:?}), got Break")
        }
        ExecutionResult::Return(_) => {
            panic!("expected RuntimeError({expected_message:?}), got Break")
        }
    }
}

fn assert_number(result: Result<Value, super::RuntimeError>, expected: f64) {
    match result {
        Ok(Value::Literal(Literal::Number(n))) => {
            assert!((n - expected).abs() < 1e-9, "expected {expected}, got {n}");
        }
        Ok(other) => panic!("expected Number({expected}), got {:?}", other),
        Err(e) => panic!("expected Ok(Number({expected})), got Err: {}", e.message),
    }
}

fn assert_string(result: Result<Value, super::RuntimeError>, expected: &str) {
    match result {
        Ok(Value::Literal(Literal::String(s))) => assert_eq!(s, expected),
        Ok(other) => panic!("expected String({expected:?}), got {:?}", other),
        Err(e) => panic!("expected Ok(String({expected:?})), got Err: {}", e.message),
    }
}

fn assert_runtime_error(result: Result<Value, super::RuntimeError>, expected_message: &str) {
    match result {
        Err(e) => assert_eq!(e.message, expected_message),
        Ok(value) => panic!(
            "expected RuntimeError({expected_message:?}), got Ok({:?})",
            value
        ),
    }
}

fn assert_boolean(result: Result<Value, super::RuntimeError>, expected: bool) {
    match result {
        Ok(Value::Literal(Literal::Boolean(value))) => assert_eq!(value, expected),
        Ok(other) => panic!("expected Boolean({expected}), got {:?}", other),
        Err(e) => panic!("expected Ok(Boolean({expected})), got Err: {}", e.message),
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

    let mut interpreter = Interpreter::new();
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

    let mut interpreter = Interpreter::new();
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

    let mut interpreter = Interpreter::new();
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

    let mut interpreter = Interpreter::new();
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

    let mut interpreter = Interpreter::new();
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

    let mut interpreter = Interpreter::new();
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

    let mut interpreter = Interpreter::new();
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

    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expression);

    match result {
        Ok(Value::Literal(Literal::Boolean(b))) => assert!(b),
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

    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expression);

    match result {
        Ok(Value::Literal(Literal::Boolean(b))) => assert!(b),
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

    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expression);

    match result {
        Ok(Value::Literal(Literal::Boolean(b))) => assert!(b),
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

    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expression);

    assert_runtime_error(result, "Operands must be numbers.");
}

#[test]
fn local_variable_is_accessible_inside_block() {
    let result = interpret(
        r#"
        {
            var a = "inside";
            print a;
        }
        "#,
    );

    assert!(matches!(result, ExecutionResult::Success));
}

#[test]
fn local_variable_is_not_accessible_outside_block() {
    let result = interpret(
        r#"
        {
            var a = "inside";
            print a;
        }

        print a;
        "#,
    );

    assert_interpret_runtime_error(result, "Undefined variable 'a'.");
}

#[test]
fn local_variable_shadows_global_variable() {
    let result = interpret(
        r#"
        var a = "global";

        {
            var a = "local";
            print a;
        }

        print a;
        "#,
    );

    assert!(matches!(result, ExecutionResult::Success));
}

#[test]
fn nested_block_can_access_outer_variable() {
    let result = interpret(
        r#"
        var a = "global";

        {
            var b = "outer";

            {
                var c = "inner";

                print a;
                print b;
                print c;
            }
        }
        "#,
    );

    assert!(matches!(result, ExecutionResult::Success));
}

#[test]
fn nested_blocks_shadow_variables_correctly() {
    let result = interpret(
        r#"
        var a = "global a";
        var b = "global b";
        var c = "global c";

        {
            var a = "outer a";
            var b = "outer b";

            {
                var a = "inner a";

                print a;
                print b;
                print c;
            }

            print a;
            print b;
            print c;
        }

        print a;
        print b;
        print c;
        "#,
    );

    assert!(matches!(result, ExecutionResult::Success));
}

#[test]
fn assignment_updates_variable_in_enclosing_scope() {
    let result = interpret(
        r#"
        var a = "global";

        {
            a = "modified";
        }

        print a;
        "#,
    );

    assert!(matches!(result, ExecutionResult::Success));
}

#[test]
fn assignment_updates_shadowing_local_variable() {
    let result = interpret(
        r#"
        var a = "global";

        {
            var a = "local";
            a = "modified";
            print a;
        }

        print a;
        "#,
    );

    assert!(matches!(result, ExecutionResult::Success));
}

#[test]
fn assignment_to_undefined_variable_is_runtime_error() {
    let result = interpret(
        r#"
        {
            a = "value";
        }
        "#,
    );

    assert_interpret_runtime_error(result, "Undefined variable 'a'.");
}

#[test]
fn if_executes_then_branch_when_condition_is_truthy() {
    let statement = crate::stmt::Stmt::If {
        condition: Expr::Literal {
            value: Literal::Boolean(true),
        },

        then_branch: Box::new(crate::stmt::Stmt::Expression {
            expression: Expr::Assign {
                id: 0,
                name: token(TokenType::Identifier, "a"),
                value: Box::new(Expr::Literal {
                    value: Literal::String("then".to_string()),
                }),
            },
        }),

        else_branch: Some(Box::new(crate::stmt::Stmt::Expression {
            expression: Expr::Assign {
                id: 1,
                name: token(TokenType::Identifier, "a"),
                value: Box::new(Expr::Literal {
                    value: Literal::String("else".to_string()),
                }),
            },
        })),
    };

    let mut interpreter = Interpreter::new();

    // Define a first.
    assert!(matches!(
        interpreter.interpret(&[crate::stmt::Stmt::Var {
            name: token(TokenType::Identifier, "a"),
            initializer: Some(Expr::Literal {
                value: Literal::String("before".to_string()),
            }),
        }]),
        ExecutionResult::Success
    ));

    // Execute if.
    assert!(matches!(
        interpreter.interpret(&[statement]),
        ExecutionResult::Success
    ));

    // Verify that the then branch executed.
    let result = interpreter.evaluate(&Expr::Variable {
        id: 2,
        name: token(TokenType::Identifier, "a"),
    });

    assert_string(result, "then");
}

#[test]
fn if_executes_else_branch_when_condition_is_falsey() {
    let statement = crate::stmt::Stmt::If {
        condition: Expr::Literal {
            value: Literal::Boolean(false),
        },

        then_branch: Box::new(crate::stmt::Stmt::Expression {
            expression: Expr::Assign {
                id: 0,
                name: token(TokenType::Identifier, "a"),
                value: Box::new(Expr::Literal {
                    value: Literal::String("then".to_string()),
                }),
            },
        }),

        else_branch: Some(Box::new(crate::stmt::Stmt::Expression {
            expression: Expr::Assign {
                id: 1,
                name: token(TokenType::Identifier, "a"),
                value: Box::new(Expr::Literal {
                    value: Literal::String("else".to_string()),
                }),
            },
        })),
    };

    let mut interpreter = Interpreter::new();

    assert!(matches!(
        interpreter.interpret(&[crate::stmt::Stmt::Var {
            name: token(TokenType::Identifier, "a"),
            initializer: Some(Expr::Literal {
                value: Literal::String("before".to_string()),
            }),
        }]),
        ExecutionResult::Success
    ));

    assert!(matches!(
        interpreter.interpret(&[statement]),
        ExecutionResult::Success
    ));

    let result = interpreter.evaluate(&Expr::Variable {
        id: 2,
        name: token(TokenType::Identifier, "a"),
    });

    assert_string(result, "else");
}

#[test]
fn if_without_else_does_nothing_when_false() {
    let statement = crate::stmt::Stmt::If {
        condition: Expr::Literal {
            value: Literal::Boolean(false),
        },

        then_branch: Box::new(crate::stmt::Stmt::Expression {
            expression: Expr::Assign {
                id: 0,
                name: token(TokenType::Identifier, "a"),
                value: Box::new(Expr::Literal {
                    value: Literal::String("changed".to_string()),
                }),
            },
        }),

        else_branch: None,
    };

    let mut interpreter = Interpreter::new();

    assert!(matches!(
        interpreter.interpret(&[crate::stmt::Stmt::Var {
            name: token(TokenType::Identifier, "a"),
            initializer: Some(Expr::Literal {
                value: Literal::String("before".to_string()),
            }),
        }]),
        ExecutionResult::Success
    ));

    assert!(matches!(
        interpreter.interpret(&[statement]),
        ExecutionResult::Success
    ));

    let result = interpreter.evaluate(&Expr::Variable {
        id: 1,
        name: token(TokenType::Identifier, "a"),
    });

    assert_string(result, "before");
}

#[test]
fn logical_or_returns_left_when_truthy() {
    let expression = Expr::Logical {
        left: Box::new(Expr::Literal {
            value: Literal::String("hi".to_string()),
        }),
        operator: token(TokenType::Or, "or"),
        right: Box::new(Expr::Literal {
            value: Literal::Number(2.0),
        }),
    };

    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expression);

    assert_string(result, "hi");
}

#[test]
fn logical_or_returns_right_when_left_is_falsey() {
    let expression = Expr::Logical {
        left: Box::new(Expr::Literal {
            value: Literal::Nil,
        }),
        operator: token(TokenType::Or, "or"),
        right: Box::new(Expr::Literal {
            value: Literal::String("yes".to_string()),
        }),
    };

    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expression);

    assert_string(result, "yes");
}

#[test]
fn logical_and_returns_left_when_falsey() {
    let expression = Expr::Logical {
        left: Box::new(Expr::Literal {
            value: Literal::Boolean(false),
        }),
        operator: token(TokenType::And, "and"),
        right: Box::new(Expr::Literal {
            value: Literal::String("never".to_string()),
        }),
    };

    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expression);

    assert_boolean(result, false);
}

#[test]
fn logical_and_returns_right_when_left_is_truthy() {
    let expression = Expr::Logical {
        left: Box::new(Expr::Literal {
            value: Literal::Boolean(true),
        }),
        operator: token(TokenType::And, "and"),
        right: Box::new(Expr::Literal {
            value: Literal::String("yes".to_string()),
        }),
    };

    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expression);

    assert_string(result, "yes");
}

#[test]
fn logical_and_short_circuits() {
    let expression = Expr::Logical {
        left: Box::new(Expr::Literal {
            value: Literal::Boolean(false),
        }),
        operator: token(TokenType::And, "and"),
        right: Box::new(Expr::Variable {
            id: 0,
            name: token(TokenType::Identifier, "undefined"),
        }),
    };

    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expression);

    assert_boolean(result, false);
}

#[test]
fn logical_or_short_circuits() {
    let expression = Expr::Logical {
        left: Box::new(Expr::Literal {
            value: Literal::Boolean(true),
        }),
        operator: token(TokenType::Or, "or"),
        right: Box::new(Expr::Variable {
            id: 0,
            name: token(TokenType::Identifier, "undefined"),
        }),
    };

    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expression);

    assert_boolean(result, true);
}

#[test]
fn break_returns_break_execution_result() {
    let statement = crate::stmt::Stmt::Break;

    let mut interpreter = Interpreter::new();

    let result = interpreter.interpret(&[statement]);

    assert!(matches!(result, ExecutionResult::Break));
}

#[test]
fn break_exits_while_loop() {
    let result = interpret(
        r#"
        while (true) {
            break;
        }
        "#,
    );

    assert!(matches!(result, ExecutionResult::Success));
}

#[test]
fn break_inside_if_exits_loop() {
    let result = interpret(
        r#"
        while (true) {
            if (true) {
                break;
            }
        }
        "#,
    );

    assert!(matches!(result, ExecutionResult::Success));
}

#[test]
fn native_clock_can_be_called() {
    let result = interpret("clock();");

    assert!(matches!(result, ExecutionResult::Success));
}

#[test]
fn this_in_method_accesses_instance_fields() {
    let result = interpret(
        r#"
        class Cake {
          taste() {
            var adjective = "delicious";
            print "The " + this.flavor + " cake is " + adjective + "!";
          }
        }

        var cake = Cake();
        cake.flavor = "German chocolate";
        cake.taste();
        "#,
    );

    assert!(matches!(result, ExecutionResult::Success));
}

#[test]
fn bound_method_retains_this() {
    let result = interpret(
        r#"
        class Egotist {
          speak() {
            print this;
          }
        }

        var method = Egotist().speak;
        method();
        "#,
    );

    assert!(matches!(result, ExecutionResult::Success));
}

#[test]
fn this_outside_class_is_resolution_error() {
    let mut lox = Lox::new();
    let mut scanner = Scanner::new("print this;");
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens, &mut lox);
    let statements = parser.parse();
    let mut interpreter = Interpreter::new();

    let errors = {
        let mut resolver = Resolver::new(&mut interpreter);
        resolver.resolve(&statements);
        resolver.into_errors()
    };

    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].message, "Can't use 'this' outside of a class.");
}

#[test]
fn class_call_creates_instance() {
    let result = interpret(
        r#"
        class Bagel {}
        var bagel = Bagel();
        print bagel;
        "#,
    );

    assert!(matches!(result, ExecutionResult::Success));
}

#[test]
fn instance_properties_can_be_read_and_written() {
    let result = interpret(
        r#"
        class Breakfast {}
        var breakfast = Breakfast();
        breakfast.food = "eggs";
        print breakfast.food;
        "#,
    );

    assert!(matches!(result, ExecutionResult::Success));
}

#[test]
fn get_on_non_instance_is_runtime_error() {
    let result = interpret(
        r#"
        var value = 123;
        print value.bad;
        "#,
    );

    assert_interpret_runtime_error(result, "Only instances have properties.");
}

#[test]
fn set_on_non_instance_is_runtime_error() {
    let result = interpret(
        r#"
        var value = 123;
        value.bad = "nope";
        "#,
    );

    assert_interpret_runtime_error(result, "Only instances have fields.");
}

#[test]
fn undefined_property_is_runtime_error() {
    let result = interpret(
        r#"
        class Thing {}
        var thing = Thing();
        print thing.missing;
        "#,
    );

    assert_interpret_runtime_error(result, "Undefined property 'missing'.");
}

#[test]
fn class_declaration_executes_successfully() {
    let result = interpret(
        r#"
        class DevonshireCream {
          serveOn() {
            return "Scones";
          }
        }

        print DevonshireCream;
        "#,
    );

    assert!(matches!(result, ExecutionResult::Success));
}

#[test]
fn native_clock_returns_number() {
    let result = interpret("print clock();");

    assert!(matches!(result, ExecutionResult::Success));
}
