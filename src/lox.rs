use std::{
    fs,
    io::{self, Write},
};

use crate::scanner::Scanner;

pub struct Lox {
    had_error: bool,
}

impl Lox {
    pub fn new() -> Self {
        Self { had_error: false }
    }

    pub fn run_file(&mut self, file_path: &str) {
        let contents =
            fs::read_to_string(file_path).expect("Should have been able to read that file");

        self.run(&contents);

        if (self.had_error) {
            std::process::exit(65);
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
        }
    }

    fn run(&mut self, source: &str) {
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();

        tokens.iter().for_each(|token| println!("{}", token));
    }

    fn error(&mut self, line: i32, message: &str) {
        self.report(line, "", message);
    }

    fn report(&mut self, line: i32, location: &str, message: &str) {
        println!("[line {}] Error {}: {}", line, location, message);
        self.had_error = true;
    }
}
