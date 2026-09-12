use std::{fmt, rc::Rc};

use crate::interpreter::{
    LoxCallable,
    lox_class::LoxClass,
    lox_instance::LoxInstanceRef,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Break,

    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

#[derive(Clone)]
pub enum Value {
    Literal(Literal),
    Callable(Rc<dyn LoxCallable>),
    Class(Rc<LoxClass>),
    Instance(LoxInstanceRef),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Literal(value) => f.debug_tuple("Literal").field(value).finish(),
            Value::Callable(_) => write!(f, "Callable"),
            Value::Class(class) => f.debug_tuple("Class").field(&class.name()).finish(),
            Value::Instance(instance) => f
                .debug_tuple("Instance")
                .field(&instance.borrow().klass_name())
                .finish(),
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Number(value) => write!(f, "{}", value),
            Literal::String(value) => write!(f, "{}", value),
            Literal::Boolean(value) => write!(f, "{}", value),
            Literal::Nil => write!(f, "nil"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub line: usize,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        lexeme: String,
        literal: Option<Literal>,
        line: usize,
    ) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.literal {
            Some(literal) => {
                write!(f, "{} {} {}", self.token_type, self.lexeme, literal)
            }
            None => {
                write!(f, "{} {}", self.token_type, self.lexeme)
            }
        }
    }
}
