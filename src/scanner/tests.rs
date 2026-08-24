use super::*;
use crate::token::{Literal, TokenType};

fn token_types(tokens: &[Token]) -> Vec<TokenType> {
    tokens
        .iter()
        .map(|token| token.token_type.clone())
        .collect()
}

#[test]
fn scans_single_character_tokens() {
    let mut scanner = Scanner::new("(){} ,.-+;*");

    let tokens = scanner.scan_tokens();

    assert_eq!(
        token_types(&tokens),
        vec![
            TokenType::LeftParen,
            TokenType::RightParen,
            TokenType::LeftBrace,
            TokenType::RightBrace,
            TokenType::Comma,
            TokenType::Dot,
            TokenType::Minus,
            TokenType::Plus,
            TokenType::Semicolon,
            TokenType::Star,
            TokenType::Eof,
        ]
    );
}

#[test]
fn scans_operators() {
    let mut scanner = Scanner::new("! != = == < <= > >=");

    let tokens = scanner.scan_tokens();

    assert_eq!(
        token_types(&tokens),
        vec![
            TokenType::Bang,
            TokenType::BangEqual,
            TokenType::Equal,
            TokenType::EqualEqual,
            TokenType::Less,
            TokenType::LessEqual,
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Eof,
        ]
    );
}

#[test]
fn ignores_comments() {
    let mut scanner = Scanner::new(
        r#"
        // this should disappear
        print 123;

        /* this should also disappear */

        print 456;
        "#,
    );

    let tokens = scanner.scan_tokens();

    assert_eq!(
        token_types(&tokens),
        vec![
            TokenType::Print,
            TokenType::Number,
            TokenType::Semicolon,
            TokenType::Print,
            TokenType::Number,
            TokenType::Semicolon,
            TokenType::Eof,
        ]
    );
}

#[test]
fn supports_nested_block_comments() {
    let mut scanner = Scanner::new(
        r#"
        print 1;

        /*
            outer comment

            /*
                nested comment
            */

            back in outer comment
        */

        print 2;
        "#,
    );

    let tokens = scanner.scan_tokens();

    assert_eq!(
        token_types(&tokens),
        vec![
            TokenType::Print,
            TokenType::Number,
            TokenType::Semicolon,
            TokenType::Print,
            TokenType::Number,
            TokenType::Semicolon,
            TokenType::Eof,
        ]
    );
}

#[test]
fn scans_strings() {
    let mut scanner = Scanner::new(r#""hello" "hello world""#);

    let tokens = scanner.scan_tokens();

    assert_eq!(tokens[0].token_type, TokenType::String);
    assert_eq!(
        tokens[0].literal,
        Some(Literal::String("hello".to_string()))
    );

    assert_eq!(tokens[1].token_type, TokenType::String);
    assert_eq!(
        tokens[1].literal,
        Some(Literal::String("hello world".to_string()))
    );

    assert_eq!(tokens[2].token_type, TokenType::Eof);
}

#[test]
fn scans_multiline_string() {
    let mut scanner = Scanner::new("\"hello\nworld\"");

    let tokens = scanner.scan_tokens();

    assert_eq!(tokens[0].token_type, TokenType::String);

    assert_eq!(
        tokens[0].literal,
        Some(Literal::String("hello\nworld".to_string()))
    );

    assert_eq!(tokens[1].token_type, TokenType::Eof);
}

#[test]
fn scans_numbers() {
    let mut scanner = Scanner::new("123 456.789");

    let tokens = scanner.scan_tokens();

    assert_eq!(tokens[0].literal, Some(Literal::Number(123.0)));

    assert_eq!(tokens[1].literal, Some(Literal::Number(456.789)));

    assert_eq!(tokens[2].token_type, TokenType::Eof);
}

#[test]
fn scans_number_edge_cases() {
    let mut scanner = Scanner::new("123. 123.45 .123");

    let tokens = scanner.scan_tokens();

    assert_eq!(tokens[0].token_type, TokenType::Number);
    assert_eq!(tokens[0].literal, Some(Literal::Number(123.0)));

    assert_eq!(tokens[1].token_type, TokenType::Dot);

    assert_eq!(tokens[2].token_type, TokenType::Number);
    assert_eq!(tokens[2].literal, Some(Literal::Number(123.45)));

    assert_eq!(tokens[3].token_type, TokenType::Dot);

    assert_eq!(tokens[4].token_type, TokenType::Number);
    assert_eq!(tokens[4].literal, Some(Literal::Number(123.0)));

    assert_eq!(tokens[5].token_type, TokenType::Eof);
}

#[test]
fn scans_identifiers() {
    let mut scanner = Scanner::new("hello world _private variable123 test_123");

    let tokens = scanner.scan_tokens();

    assert_eq!(
        token_types(&tokens),
        vec![
            TokenType::Identifier,
            TokenType::Identifier,
            TokenType::Identifier,
            TokenType::Identifier,
            TokenType::Identifier,
            TokenType::Eof,
        ]
    );
}

#[test]
fn scans_keywords() {
    let mut scanner = Scanner::new(
        "and class else false for fun if nil or \
         print return super this true var while",
    );

    let tokens = scanner.scan_tokens();

    assert_eq!(
        token_types(&tokens),
        vec![
            TokenType::And,
            TokenType::Class,
            TokenType::Else,
            TokenType::False,
            TokenType::For,
            TokenType::Fun,
            TokenType::If,
            TokenType::Nil,
            TokenType::Or,
            TokenType::Print,
            TokenType::Return,
            TokenType::Super,
            TokenType::This,
            TokenType::True,
            TokenType::Var,
            TokenType::While,
            TokenType::Eof,
        ]
    );
}

#[test]
fn does_not_confuse_keywords_with_identifiers() {
    let mut scanner = Scanner::new("or orchid class classifier true truthful");

    let tokens = scanner.scan_tokens();

    assert_eq!(
        token_types(&tokens),
        vec![
            TokenType::Or,
            TokenType::Identifier,
            TokenType::Class,
            TokenType::Identifier,
            TokenType::True,
            TokenType::Identifier,
            TokenType::Eof,
        ]
    );
}

#[test]
fn preserves_literal_values() {
    let mut scanner = Scanner::new(r#"123 45.67 "hello" "world""#);

    let tokens = scanner.scan_tokens();

    assert_eq!(tokens[0].literal, Some(Literal::Number(123.0)));

    assert_eq!(tokens[1].literal, Some(Literal::Number(45.67)));

    assert_eq!(
        tokens[2].literal,
        Some(Literal::String("hello".to_string()))
    );

    assert_eq!(
        tokens[3].literal,
        Some(Literal::String("world".to_string()))
    );
}

#[test]
fn tracks_lines() {
    let mut scanner = Scanner::new("print 1;\nprint 2;\nprint 3;");

    let tokens = scanner.scan_tokens();

    assert_eq!(tokens[0].line, 1);
    assert_eq!(tokens[4].line, 2);
    assert_eq!(tokens[8].line, 3);
}

#[test]
fn tracks_lines_inside_block_comments() {
    let mut scanner = Scanner::new("/*\n\n\n*/\nprint 123;");

    let tokens = scanner.scan_tokens();

    assert_eq!(tokens[0].token_type, TokenType::Print);
    assert_eq!(tokens[0].line, 5);

    assert_eq!(tokens[1].token_type, TokenType::Number);
    assert_eq!(tokens[1].line, 5);
}

#[test]
fn continues_after_invalid_character() {
    let mut scanner = Scanner::new("@ print 123; #");

    let tokens = scanner.scan_tokens();

    assert_eq!(
        token_types(&tokens),
        vec![
            TokenType::Print,
            TokenType::Number,
            TokenType::Semicolon,
            TokenType::Eof,
        ]
    );

    assert!(scanner.had_error());
}
