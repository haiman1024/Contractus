
// Contractus Lexer 集成测试
use contractus::lexer::{Lexer, TokenKind};

#[test]
fn test_param() {
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
}

#[test]
fn test_array_syntax() {
    let input = "let arr: [i32; 5] = [1, 2, 3, 4, 5];";
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Let);
    assert_eq!(tokens[1].kind, TokenKind::Ident("arr".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::Colon);
    assert_eq!(tokens[3].kind, TokenKind::LeftBracket);
    assert_eq!(tokens[4].kind, TokenKind::I32);
    assert_eq!(tokens[5].kind, TokenKind::Semicolon);
    assert_eq!(tokens[6].kind, TokenKind::IntLiteral(5));
    assert_eq!(tokens[7].kind, TokenKind::RightBracket);
    assert_eq!(tokens[8].kind, TokenKind::Assign);
    assert_eq!(tokens[9].kind, TokenKind::LeftBracket);
    assert_eq!(tokens[10].kind, TokenKind::IntLiteral(1));
}

#[test]
fn test_operators() {
    let input = "== != <= >= -> ..";
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Equal);
    assert_eq!(tokens[1].kind, TokenKind::NotEqual);
    assert_eq!(tokens[2].kind, TokenKind::LessEqual);
    assert_eq!(tokens[3].kind, TokenKind::GreaterEqual);
    assert_eq!(tokens[4].kind, TokenKind::Arrow);
    assert_eq!(tokens[5].kind, TokenKind::DotDot);
}

#[test]
fn test_string_literals() {
    let input = r#""Hello, World!" "test""#;
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::StringLiteral("Hello, World!".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::StringLiteral("test".to_string()));
}

#[test]
fn test_comments_ignored() {
    let input = "fn main() { // This is a comment\n let x = 42; }";
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    // 注释应该被跳过
    assert_eq!(tokens[0].kind, TokenKind::Fn);
    assert_eq!(tokens[1].kind, TokenKind::Ident("main".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::LeftParen);
    assert_eq!(tokens[3].kind, TokenKind::RightParen);
    assert_eq!(tokens[4].kind, TokenKind::LeftBrace);
    // 注释被跳过，直接到 let
    assert_eq!(tokens[5].kind, TokenKind::Let);
    assert_eq!(tokens[6].kind, TokenKind::Ident("x".to_string()));
}

#[test]
fn test_complex_program() {
    let input = r#"
        struct Point {
            x: i32,
            y: i32,
        }

        fn add_points(a: Point, b: Point) -> Point {
            return Point { x: a.x + b.x, y: a.y + b.y };
        }

        fn main() -> i32 {
            let p1 = Point { x: 10, y: 20 };
            for i in 0..5 {
                print(i);
            }
            return 0;
        }
    "#;

    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    // 验证关键 tokens 存在
    let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();

    assert!(token_kinds.contains(&&TokenKind::Struct));
    assert!(token_kinds.contains(&&TokenKind::Fn));
    assert!(token_kinds.contains(&&TokenKind::For));
    assert!(token_kinds.contains(&&TokenKind::In));
    assert!(token_kinds.contains(&&TokenKind::DotDot));
    assert!(token_kinds.contains(&&TokenKind::Arrow));
    assert!(token_kinds.contains(&&TokenKind::Return));

    // 验证最后是 EOF
    assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof);
}

#[test]
fn test_error_handling() {
    let input = "fn main() { let x = @; }"; // @ 是无效字符
    let lexer = Lexer::new(input);
    let result = lexer.tokenize();

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("Unexpected character"));
}

#[test]
fn test_unterminated_string() {
    let input = r#"fn main() { let s = "unterminated; }"#;
    let lexer = Lexer::new(input);
    let result = lexer.tokenize();

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("Unterminated string"));
}
