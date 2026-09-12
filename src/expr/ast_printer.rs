use std::fmt::format;

use super::Expr;

pub struct AstPrinter;

impl AstPrinter {
    pub fn new() -> Self {
        Self
    }

    pub fn print(&self, expr: &Expr) -> String {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => self.parenthesize(&operator.lexeme, &[left.as_ref(), right.as_ref()]),

            Expr::Grouping { expression } => self.parenthesize("group", &[expression.as_ref()]),

            Expr::Literal { value } => value.to_string(),

            Expr::Unary { operator, right } => {
                self.parenthesize(&operator.lexeme, &[right.as_ref()])
            }

            Expr::Variable { name, .. } => name.lexeme.clone(),

            Expr::Assign { name, value, .. } => {
                format!("(= {} {})", name.lexeme, self.print(value))
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => self.parenthesize(&operator.lexeme, &[left.as_ref(), right.as_ref()]),

            Expr::Call {
                callee, arguments, ..
            } => {
                let mut expressions = Vec::with_capacity(arguments.len() + 1);
                expressions.push(callee.as_ref());

                for argument in arguments {
                    expressions.push(argument);
                }

                self.parenthesize("call", &expressions)
            }

            Expr::Get { object, name } => {
                format!("(. {} {})", self.print(object), name.lexeme)
            }

            Expr::Set { object, name, value } => {
                format!("(= (. {} {}) {})", self.print(object), name.lexeme, self.print(value))
            }

            Expr::This { .. } => "this".to_string(),

            Expr::Function { params, .. } => {
                let param_names: Vec<&str> =
                    params.iter().map(|token| token.lexeme.as_str()).collect();
                format!("(fun ({}) <body>)", param_names.join(" "))
            }
        }
    }

    fn parenthesize(&self, name: &str, expressions: &[&Expr]) -> String {
        let mut builder = String::new();

        builder.push('(');
        builder.push_str(name);

        for expression in expressions {
            builder.push(' ');
            builder.push_str(&self.print(expression));
        }

        builder.push(')');

        builder
    }
}
