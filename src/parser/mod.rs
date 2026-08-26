use crate::{
    expr::Expr,
    lox::Lox,
    token::{Literal, Token, TokenType},
};

#[derive(Debug)]
pub struct ParseError;

pub struct Parser<'a> {
    tokens: Vec<Token>,
    current: usize,
    lox: &'a mut Lox,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, lox: &'a mut Lox) -> Self {
        Self {
            tokens,
            current: 0,
            lox,
        }
    }

    pub fn parse(&mut self) -> Option<Expr> {
        match self.expression() {
            Ok(expr) => Some(expr),
            Err(_) => None,
        }
    }

    // expression → equality ;
    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
    }

    // equality → comparison ( ( "!=" | "==" ) comparison )* ;
    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while self.matches(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    // comparison → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;

        while self.matches(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.term()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    // term → factor ( ( "-" | "+" ) factor )* ;
    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;

        while self.matches(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.factor()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    // factor → unary ( ( "/" | "*" ) unary )* ;
    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        while self.matches(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    // unary → ( "!" | "-" ) unary | primary ;
    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.matches(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;

            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }

        self.primary()
    }

    // primary → NUMBER | STRING | "true" | "false" | "nil"
    // | "(" expression ")" ;
    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.matches(&[TokenType::False]) {
            return Ok(Expr::Literal {
                value: Literal::Boolean(false),
            });
        }

        if self.matches(&[TokenType::True]) {
            return Ok(Expr::Literal {
                value: Literal::Boolean(true),
            });
        }

        if self.matches(&[TokenType::Nil]) {
            return Ok(Expr::Literal {
                value: Literal::Nil,
            });
        }

        if self.matches(&[TokenType::Number, TokenType::String]) {
            let token = self.previous();

            return Ok(Expr::Literal {
                value: token.literal.clone().unwrap(),
            });
        }

        if self.matches(&[TokenType::LeftParen]) {
            let expr = self.expression()?;

            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;

            return Ok(Expr::Grouping {
                expression: Box::new(expr),
            });
        }

        let token = self.peek().clone();
        Err(self.error(&token, "Expect expression."))
    }

    fn matches(&mut self, types: &[TokenType]) -> bool {
        for token_type in types {
            if self.check(*token_type) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, ParseError> {
        if self.check(token_type) {
            return Ok(self.advance());
        }

        let token = self.peek().clone();

        Err(self.error(&token, message))
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().token_type == token_type
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn error(&mut self, token: &Token, message: &str) -> ParseError {
        self.lox.error_token(token, message);

        ParseError
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,

                _ => self.advance(),
            };
        }
    }
}
