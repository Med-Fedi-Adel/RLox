use std::{ptr::null, rc::Rc};

use crate::{
    expr::Expr,
    expr_id::{ExprId, ExprIdGenerator},
    lox::Lox,
    stmt::{ClassMember, Stmt},
    token::{Literal, Token, TokenType},
};

#[derive(Debug)]
pub struct ParseError;

pub struct Parser<'a> {
    tokens: Vec<Token>,
    current: usize,
    lox: &'a mut Lox,
    loop_depth: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, lox: &'a mut Lox) -> Self {
        Self {
            tokens,
            current: 0,
            lox,
            loop_depth: 0,
        }
    }

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            match self.declaration() {
                Ok(statement) => statements.push(statement),
                Err(_) => {
                    self.synchronize();
                }
            }
        }

        statements
    }

    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        if self.matches(&[TokenType::Class]) {
            return self.class_decl();
        }

        if self.check(TokenType::Fun) && self.check_next(TokenType::Identifier) {
            self.advance();
            return self.function_decl("function");
        }

        if self.matches(&[TokenType::Var]) {
            return self.var_declaration();
        }

        self.statement()
    }

    fn class_decl(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenType::Identifier, "Expect class name.")?;

        let superclass = if self.matches(&[TokenType::Less]) {
            let super_name = self.consume(TokenType::Identifier, "Expect superclass name.")?;

            Some(Expr::Variable {
                id: self.lox.next_expr_id(),
                name: super_name,
            })
        } else {
            None
        };

        self.consume(TokenType::LeftBrace, "Expect '{' before class body.")?;

        let mut members = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            members.push(self.class_member()?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after class body.")?;

        Ok(Stmt::Class {
            name,
            superclass,
            members,
        })
    }

    fn class_member(&mut self) -> Result<ClassMember, ParseError> {
        let is_static = self.matches(&[TokenType::Class]);
        let name = self.consume(TokenType::Identifier, "Expect method name.")?;

        if self.check(TokenType::LeftBrace) {
            if is_static {
                return Err(self.error(
                    &name,
                    "Static getters are not allowed.",
                ));
            }

            let body = self.getter_body()?;

            return Ok(ClassMember::Getter { name, body });
        }

        let (parameters, body) = self.method_params_and_body("method")?;

        if is_static {
            Ok(ClassMember::StaticMethod {
                name,
                parameters,
                body,
            })
        } else {
            Ok(ClassMember::Method {
                name,
                parameters,
                body,
            })
        }
    }

    fn method_params_and_body(
        &mut self,
        kind: &str,
    ) -> Result<(Rc<Vec<Token>>, Rc<Vec<Stmt>>), ParseError> {
        self.consume(
            TokenType::LeftParen,
            &format!("Expect '(' after {} name.", kind),
        )?;

        let mut parameters: Vec<Token> = Vec::new();

        if !self.check(TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    let token = self.peek().clone();
                    self.error(&token, "Can't have more than 255 parameters.");
                }

                parameters.push(self.consume(TokenType::Identifier, "Expect parameter name.")?);

                if !self.matches(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;

        self.consume(
            TokenType::LeftBrace,
            &format!("Expect '{{' before {} body.", kind),
        )?;

        let body = self.block()?;

        Ok((Rc::new(parameters), Rc::new(body)))
    }

    fn getter_body(&mut self) -> Result<Rc<Vec<Stmt>>, ParseError> {
        self.consume(TokenType::LeftBrace, "Expect '{' before getter body.")?;

        Ok(Rc::new(self.block()?))
    }

    fn check_next(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        match self.tokens.get(self.current + 1) {
            Some(token) => token.token_type == token_type,
            None => false,
        }
    }

    // Used by declaration(): consumes name, then delegates.
    fn function_decl(&mut self, kind: &str) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenType::Identifier, &format!("Expect {} name.", kind))?;
        let (parameters, body) = self.function_params_and_body(kind)?;
        Ok(Stmt::Function {
            name,
            parameters,
            body,
        })
    }

    // Used by primary(): no name, produces an Expr.
    fn function_body(&mut self, kind: &str) -> Result<Expr, ParseError> {
        let (params, body) = self.function_params_and_body(kind)?;
        Ok(Expr::Function { params, body })
    }

    // Shared: `(params) { body }`
    fn function_params_and_body(
        &mut self,
        kind: &str,
    ) -> Result<(Rc<Vec<Token>>, Rc<Vec<Stmt>>), ParseError> {
        self.consume(
            TokenType::LeftParen,
            &format!("Expect '(' after {} name.", kind),
        )?;

        let mut parameters: Vec<Token> = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    let token = self.peek().clone();
                    self.error(&token, "Can't have more than 255 parameters.");
                }
                parameters.push(self.consume(TokenType::Identifier, "Expect parameter name.")?);
                if !self.matches(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;

        self.consume(
            TokenType::LeftBrace,
            &format!("Expect '{{' before {} body.", kind),
        )?;
        let body = self.block()?;

        Ok((Rc::new(parameters), Rc::new(body)))
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;

        let initializer = if self.matches(&[TokenType::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;

        Ok(Stmt::Var { name, initializer })
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.matches(&[TokenType::For]) {
            return self.for_statement();
        }

        if self.matches(&[TokenType::If]) {
            return self.if_statement();
        }

        if self.matches(&[TokenType::Print]) {
            return self.print_statement();
        }

        if self.matches(&[TokenType::Return]) {
            return self.return_statement();
        }

        if self.matches(&[TokenType::While]) {
            return self.while_statement();
        }

        if self.matches(&[TokenType::Break]) {
            return self.break_statement();
        }

        if self.matches(&[TokenType::LeftBrace]) {
            return Ok(Stmt::Block {
                statements: self.block()?,
            });
        }

        return self.expression_statement();
    }

    fn return_statement(&mut self) -> Result<Stmt, ParseError> {
        let keyword: Token = self.previous().clone();

        let value = if !self.check(TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expect ';' after return value.")?;

        Ok(Stmt::Return { keyword, value })
    }

    fn break_statement(&mut self) -> Result<Stmt, ParseError> {
        let keyword = self.previous();

        if self.loop_depth == 0 {
            return Err(self.error(&keyword, "Cannot use 'break' outside of a loop."));
        }

        self.consume(TokenType::Semicolon, "Expect ';' after 'break'.")?;

        Ok(Stmt::Break)
    }

    fn for_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.)")?;

        let initializer = if self.matches(&[TokenType::Semicolon]) {
            None
        } else if self.matches(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if !self.check(TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expect ';' after loop condition.")?;

        let increment = if !self.check(TokenType::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::RightParen, "Expect ')' after clauses.")?;

        self.loop_depth += 1;
        let mut body = self.statement()?;
        self.loop_depth -= 1;

        if let Some(increment) = increment {
            body = Stmt::Block {
                statements: vec![
                    body,
                    Stmt::Expression {
                        expression: increment,
                    },
                ],
            }
        };

        let condition = condition.unwrap_or(Expr::Literal {
            value: Literal::Boolean(true),
        });

        body = Stmt::While {
            condition,
            body: Box::new(body),
        };

        if let Some(initializer) = initializer {
            body = Stmt::Block {
                statements: vec![initializer, body],
            };
        }

        Ok(body)
    }

    fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;

        let condition = self.expression()?;

        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;

        self.loop_depth += 1;
        let body = self.statement()?;
        self.loop_depth -= 1;

        Ok(Stmt::While {
            condition,
            body: Box::new(body),
        })
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.)")?;

        let condition = self.expression()?;

        self.consume(TokenType::RightParen, "Expect ')' after if condition.)")?;
        let then_branch = self.statement()?;

        let else_branch = if self.matches(&[TokenType::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            match self.declaration() {
                Ok(statement) => statements.push(statement),
                Err(_) => {
                    self.synchronize();
                }
            }
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;

        Ok(statements)
    }

    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression()?;

        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;

        Ok(Stmt::Print { expression: value })
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;

        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;

        Ok(Stmt::Expression { expression: expr })
    }

    // expression → assignment ;
    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    // assignment -> INDENTIFIER "=" assignment | equality;
    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.logic_or()?;

        if self.matches(&[TokenType::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;
            if let Expr::Variable { name, .. } = expr {
                return Ok(Expr::Assign {
                    id: self.lox.next_expr_id(),
                    name,
                    value: Box::new(value),
                });
            }

            if let Expr::Get { object, name } = expr {
                return Ok(Expr::Set {
                    object,
                    name,
                    value: Box::new(value),
                });
            }

            self.error(&equals, "Invalid assignment target.");
        }

        Ok(expr)
    }

    fn logic_or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.logic_and()?;

        while self.matches(&[TokenType::Or]) {
            let operator = self.previous().clone();
            let right = self.logic_and()?;

            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn logic_and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;

        while self.matches(&[TokenType::And]) {
            let operator = self.previous().clone();
            let right = self.equality()?;

            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
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

    // unary → ( "!" | "-" ) unary | primary | call;
    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.matches(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;

            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }

        self.call()
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;

        loop {
            if self.matches(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.matches(&[TokenType::Dot]) {
                let name = self.consume(
                    TokenType::Identifier,
                    "Expect property name after '.'.",
                )?;

                expr = Expr::Get {
                    object: Box::new(expr),
                    name,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        let mut arguments = Vec::new();

        if !self.check(TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    let token = self.peek().clone();
                    self.error(&token, "Can't have more than 255 arguments.");
                }

                arguments.push(self.expression()?);

                if !self.matches(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;

        Ok(Expr::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        })
    }

    // primary → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" | IDENTIFIER ;
    // | "(" expression ")" ;
    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.matches(&[TokenType::Fun]) {
            let keyword = self.previous();

            if self.check(TokenType::Identifier) && self.check_next(TokenType::LeftParen) {
                return Err(self.error(&keyword, "Expect expression."));
            }

            return self.function_body("function");
        }

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

        if self.matches(&[TokenType::Super]) {
            let keyword = self.previous();

            self.consume(TokenType::Dot, "Expect '.' after 'super'.")?;

            let method = self.consume(TokenType::Identifier, "Expect superclass method name.")?;

            return Ok(Expr::Super {
                id: self.lox.next_expr_id(),
                keyword,
                method,
            });
        }

        if self.matches(&[TokenType::This]) {
            return Ok(Expr::This {
                id: self.lox.next_expr_id(),
                keyword: self.previous(),
            });
        }

        if self.matches(&[TokenType::Identifier]) {
            return Ok(Expr::Variable {
                id: self.lox.next_expr_id(),
                name: self.previous(),
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

    fn next_expr_id(&mut self) -> ExprId {
        self.lox.next_expr_id()
    }
}

#[cfg(test)]
mod tests;
