use std::{
    fs,
    io::{self, Write},
};

use crate::{
    interpreter::{self, Interpreter},
    parser::Parser,
    scanner::Scanner,
    token::{Token, TokenType},
};

pub struct Lox {
    had_error: bool,
    had_runtime_error: bool,
}

impl Lox {
    pub fn new() -> Self {
        Self {
            had_error: false,
            had_runtime_error: false,
        }
    }

    pub fn run_file(&mut self, file_path: &str) {
        let contents =
            fs::read_to_string(file_path).expect("Should have been able to read that file");

        self.run(&contents);

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
                Ok(_) => self.run(&line),
                Err(_) => break,
            }

            self.had_error = false;
            self.had_runtime_error = false;
        }
    }

    fn run(&mut self, source: &str) {
        let mut scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();

        let expression = {
            let mut parser = Parser::new(tokens, self);
            parser.parse()
        };

        if self.had_error {
            return;
        }

        let Some(expression) = expression else {
            return;
        };

        let interpreter = Interpreter::new();

        if let Err(error) = interpreter.interpret(&expression) {
            self.runtime_error(&error);
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
}
