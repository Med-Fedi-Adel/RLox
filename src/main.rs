mod expr;
mod interpreter;
mod lox;
mod parser;
mod scanner;
mod stmt;
mod token;

use lox::Lox;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut lox = Lox::new();

    if args.len() > 2 {
        println!("Usage: jlox [script]");
    } else if args.len() == 2 {
        lox.run_file(&args[1]);
    } else {
        lox.run_prompt();
    }
}
