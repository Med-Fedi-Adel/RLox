use super::*;
use crate::token::{Literal, Token, TokenType};

#[test]
fn prints_expression_tree() {
    let expression = Expr::Binary {
        left: Box::new(Expr::Unary {
            operator: Token::new(TokenType::Minus, "-".to_string(), None, 1),
            right: Box::new(Expr::Literal {
                value: Literal::Number(123.0),
            }),
        }),

        operator: Token::new(TokenType::Star, "*".to_string(), None, 1),

        right: Box::new(Expr::Grouping {
            expression: Box::new(Expr::Literal {
                value: Literal::Number(45.67),
            }),
        }),
    };

    let printer = AstPrinter::new();

    assert_eq!(printer.print(&expression), "(* (- 123) (group 45.67))");
}
