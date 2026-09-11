use std::{
    fs,
    io::{self, Write},
};

use crate::{
    expr_id::{ExprId, ExprIdGenerator},
    interpreter::{self, Interpreter},
    parser::Parser,
    resolver::Resolver,
    scanner::Scanner,
    token::{Token, TokenType},
};

pub struct Lox {
    had_error: bool,
    had_runtime_error: bool,
    interpreter: Interpreter,
    expr_id_generator: ExprIdGenerator,
}

impl Lox {
    pub fn new() -> Self {
        Self {
            had_error: false,
            had_runtime_error: false,
            interpreter: Interpreter::new(),
            expr_id_generator: ExprIdGenerator::new(),
        }
    }

    pub fn run_file(&mut self, file_path: &str) {
        let contents =
            fs::read_to_string(file_path).expect("Should have been able to read that file");

        self.run(&contents, false);

        if self.had_error {
            std::process::exit(65);
        }

        if self.had_runtime_error {
            std::process::exit(70);
        }
    }

    pub fn run_prompt(&mut self) {
        loop {
            print!("> ");
            io::stdout().flush().unwrap();

            let mut line = String::new();

            match io::stdin().read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => self.run(&line, true),
                Err(_) => break,
            }

            self.had_error = false;
            self.had_runtime_error = false;
        }
    }

    fn run(&mut self, source: &str, print_expression: bool) {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();

        let statements = {
            let mut parser = Parser::new(tokens, self);
            parser.parse()
        };

        if self.had_error {
            return;
        }

        let resolution_erros = {
            let mut resolver = Resolver::new(&mut self.interpreter);
            resolver.resolve(&statements);
            resolver.into_errors()
        };

        for error in resolution_erros {
            self.error_token(&error.token, &error.message);
        }

        if self.had_error {
            return;
        }

        for statement in &statements {
            match statement {
                crate::stmt::Stmt::Expression { expression } if print_expression => {
                    match self.interpreter.evaluate_expression(expression) {
                        Ok(value) => println!("{}", self.interpreter.stringify(&value)),
                        Err(error) => {
                            self.runtime_error(&error);
                            return;
                        }
                    }
                }

                _ => {
                    match self.interpreter.interpret(std::slice::from_ref(statement)) {
                        interpreter::ExecutionResult::Success => {}

                        interpreter::ExecutionResult::RuntimeError(error) => {
                            self.runtime_error(&error);
                            return;
                        }

                        interpreter::ExecutionResult::Break => {
                            // This should normally be impossible because
                            // the parser only allows `break` inside a loop.
                            unreachable!("break escaped a loop");
                        }

                        interpreter::ExecutionResult::Return(_) => {
                            // No resolver yet to statically reject top-level
                            // `return`, so a stray one here is just a no-op.
                            // Once the resolver chapter is implemented, this
                            // becomes unreachable like Break above.
                            todo!()
                        }
                    }
                }
            }
        }
    }

    pub fn runtime_error(&mut self, error: &crate::interpreter::RuntimeError) {
        eprintln!("{}\n[line {}]", error.message, error.token.line);

        self.had_runtime_error = true;
    }

    pub fn error(&mut self, line: usize, message: &str) {
        self.report(line, "", message);
    }

    pub fn error_token(&mut self, token: &Token, message: &str) {
        if token.token_type == TokenType::Eof {
            self.report(token.line, " at end", message);
        } else {
            self.report(token.line, &format!(" at '{}'", token.lexeme), message);
        }
    }

    fn report(&mut self, line: usize, location: &str, message: &str) {
        eprintln!("[line {}] Error{}: {}", line, location, message);

        self.had_error = true;
    }

    pub fn next_expr_id(&mut self) -> ExprId {
        self.expr_id_generator.next()
    }
}
