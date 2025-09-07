// Contractus 词法分析器测试
// 测试词法分析器的所有功能

use contractus::{Lexer, TokenKind};

#[test]
fn test_basic_tokens() {
    let input = "fn main() { let x = 42; }";
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Fn);
    assert_eq!(tokens[1].kind, TokenKind::Ident("main".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::LeftParen);
    assert_eq!(tokens[3].kind, TokenKind::RightParen);
    assert_eq!(tokens[4].kind, TokenKind::LeftBrace);
    assert_eq!(tokens[5].kind, TokenKind::Let);
    assert_eq!(tokens[6].kind, TokenKind::Ident("x".to_string()));
    assert_eq!(tokens[7].kind, TokenKind::Assign);
    assert_eq!(tokens[8].kind, TokenKind::IntLiteral(42));
    assert_eq!(tokens[9].kind, TokenKind::Semicolon);
    assert_eq!(tokens[10].kind, TokenKind::RightBrace);
    assert_eq!(tokens[11].kind, TokenKind::Eof);
}

#[test]
fn test_keywords() {
    let input = "fn let if else for while struct return in";
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Fn);
    assert_eq!(tokens[1].kind, TokenKind::Let);
    assert_eq!(tokens[2].kind, TokenKind::If);
    assert_eq!(tokens[3].kind, TokenKind::Else);
    assert_eq!(tokens[4].kind, TokenKind::For);
    assert_eq!(tokens[5].kind, TokenKind::While);
    assert_eq!(tokens[6].kind, TokenKind::Struct);
    assert_eq!(tokens[7].kind, TokenKind::Return);
    assert_eq!(tokens[8].kind, TokenKind::In);
}

#[test]
fn test_numbers() {
    let input = "0 42 123 999";
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::IntLiteral(0));
    assert_eq!(tokens[1].kind, TokenKind::IntLiteral(42));
    assert_eq!(tokens[2].kind, TokenKind::IntLiteral(123));
    assert_eq!(tokens[3].kind, TokenKind::IntLiteral(999));
}

#[test]
fn test_operators() {
    let input = "+ - * / = == != < > <= >=";
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Plus);
    assert_eq!(tokens[1].kind, TokenKind::Minus);
    assert_eq!(tokens[2].kind, TokenKind::Star);
    assert_eq!(tokens[3].kind, TokenKind::Slash);
    assert_eq!(tokens[4].kind, TokenKind::Assign);
    assert_eq!(tokens[5].kind, TokenKind::Equal);
    assert_eq!(tokens[6].kind, TokenKind::NotEqual);
    assert_eq!(tokens[7].kind, TokenKind::Less);
    assert_eq!(tokens[8].kind, TokenKind::Greater);
    assert_eq!(tokens[9].kind, TokenKind::LessEqual);
    assert_eq!(tokens[10].kind, TokenKind::GreaterEqual);
}

#[test]
fn test_struct_definition() {
    let input = "struct Point { x: i32, y: i32 }";
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Struct);
    assert_eq!(tokens[1].kind, TokenKind::Ident("Point".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::LeftBrace);
    assert_eq!(tokens[3].kind, TokenKind::Ident("x".to_string()));
    assert_eq!(tokens[4].kind, TokenKind::Colon);
    assert_eq!(tokens[5].kind, TokenKind::I32);
    assert_eq!(tokens[6].kind, TokenKind::Comma);
    assert_eq!(tokens[7].kind, TokenKind::Ident("y".to_string()));
    assert_eq!(tokens[8].kind, TokenKind::Colon);
    assert_eq!(tokens[9].kind, TokenKind::I32);
}

#[test]
fn test_for_loop_syntax() {
    let input = "for i in 0..10 { print(i); }";
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::For);
    assert_eq!(tokens[1].kind, TokenKind::Ident("i".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::In);
    assert_eq!(tokens[3].kind, TokenKind::IntLiteral(0));
    assert_eq!(tokens[4].kind, TokenKind::DotDot);
    assert_eq!(tokens[5].kind, TokenKind::IntLiteral(10));
    assert_eq!(tokens[6].kind, TokenKind::LeftBrace);
    assert_eq!(tokens[7].kind, TokenKind::Ident("print".to_string()));
    assert_eq!(tokens[8].kind, TokenKind::LeftParen);
    assert_eq!(tokens[9].kind, TokenKind::Ident("i".to_string()));
    assert_eq!(tokens[10].kind, TokenKind::RightParen);
    assert_eq!(tokens[11].kind, TokenKind::Semicolon);
    assert_eq!(tokens[12].kind, TokenKind::RightBrace);
    assert_eq!(tokens[13].kind, TokenKind::Eof);
}