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

            Expr::Variable { name } => name.lexeme.clone(),
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
