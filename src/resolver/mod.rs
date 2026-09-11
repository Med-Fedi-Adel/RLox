use std::{collections::HashMap, thread::scope};

use crate::{
    expr::Expr,
    expr_id::ExprId,
    interpreter::{self, Interpreter},
    stmt::Stmt,
    token::Token,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum FunctionType {
    None,
    Function,
}

#[derive(Debug)]
pub struct ResolutionError {
    pub token: Token,
    pub message: String,
}

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    errors: Vec<ResolutionError>,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
            errors: Vec::new(),
        }
    }

    pub fn resolve(&mut self, statements: &[Stmt]) {
        for statement in statements {
            self.resolve_stmt(statement);
        }
    }

    pub fn resolve_stmt(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Expression { expression } => {
                self.resolve_expr(expression);
            }

            Stmt::Print { expression } => {
                self.resolve_expr(expression);
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(condition);
                self.resolve_stmt(then_branch);

                if let Some(else_branch) = else_branch {
                    self.resolve_stmt(else_branch);
                }
            }

            Stmt::While { condition, body } => {
                self.resolve_expr(condition);
                self.resolve_stmt(body);
            }

            Stmt::Block { statements } => {
                self.begin_scope();
                self.resolve(statements);
                self.end_scope();
            }

            Stmt::Var { name, initializer } => {
                self.declare(name);

                if let Some(initializer) = initializer {
                    self.resolve_expr(initializer);
                }

                self.define(name);
            }

            Stmt::Function {
                name,
                parameters,
                body,
            } => {
                self.declare(name);
                self.define(name);

                self.resolve_function(parameters, body, FunctionType::Function);
            }

            Stmt::Return { keyword, value } => {
                if self.current_function == FunctionType::None {
                    self.error(keyword, "Can't return from top level code.");
                }

                if let Some(value) = value {
                    self.resolve_expr(value);
                }
            }

            _ => {}
        }
    }

    fn resolve_expr(&mut self, expression: &Expr) {
        match expression {
            Expr::Binary {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }

            Expr::Grouping { expression } => {
                self.resolve_expr(expression);
            }

            Expr::Literal { value: _ } => {}

            Expr::Unary { operator: _, right } => {
                self.resolve_expr(right);
            }

            Expr::Variable { id, name } => {
                let read_in_own_initializer =
                    self.scopes.last().and_then(|scope| scope.get(&name.lexeme)) == Some(&false);

                if read_in_own_initializer {
                    self.error(name, "Can't read local variable in its own initializer.");
                }

                self.resolve_local(*id, name);
            }

            Expr::Assign { id, name, value } => {
                self.resolve_expr(value);
                self.resolve_local(*id, name);
            }

            Expr::Logical {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }

            Expr::Call {
                callee,
                paren: _,
                arguments,
            } => {
                self.resolve_expr(callee);

                for argument in arguments {
                    self.resolve_expr(argument);
                }
            }

            Expr::Function { params, body } => {
                self.resolve_function(params, body, FunctionType::Function);
            }
            _ => {}
        }
    }

    fn resolve_function(&mut self, params: &[Token], body: &[Stmt], function_type: FunctionType) {
        let enclosing_function = self.current_function;
        self.current_function = function_type;

        self.begin_scope();

        for param in params {
            self.declare(param);
            self.define(param);
        }

        self.resolve(body);
        self.end_scope();

        self.current_function = enclosing_function;
    }

    fn resolve_local(&mut self, id: ExprId, name: &Token) {
        for (distance, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(id, distance);
                return;
            }
        }
    }

    fn declare(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        let already_declared = self
            .scopes
            .last_mut()
            .expect("Resolver must have an active scope")
            .contains_key(&name.lexeme);

        if already_declared {
            self.error(name, "Already a variable with this name in this scope.");
        }

        self.scopes
            .last_mut()
            .expect("Resolver must have an active scope")
            .insert(name.lexeme.clone(), false);
    }

    fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        let scope = self.scopes.last_mut().unwrap();

        scope.insert(name.lexeme.clone(), true);
    }

    fn error(&mut self, token: &Token, message: impl Into<String>) {
        self.errors.push(ResolutionError {
            token: token.clone(),
            message: message.into(),
        });
    }

    pub fn into_errors(self) -> Vec<ResolutionError> {
        self.errors
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }
}
