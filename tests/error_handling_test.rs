// Contractus 错误处理测试
// 测试词法分析器和解析器的错误处理能力

use contractus::{Lexer, Parser};

fn parse_program(input: &str) -> Result<contractus::ast::Program, Vec<contractus::parser::ParseError>> {
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().map_err(|e| {
        vec![contractus::parser::ParseError::new(
            format!("Lexer error: {:?}", e),
            contractus::span::Span::new(0, 0, 1, 1),
        )]
    })?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[test]
fn test_lexer_error_handling() {
    // 测试词法分析器对无效输入的处理
    let invalid_input = r#"let x = 123abc;"#;
    let lexer = Lexer::new(invalid_input);
    // 词法分析器应该能够处理无效输入并产生错误标记
    let tokens = lexer.tokenize();
    assert!(tokens.is_ok()); // 词法分析器不应该panic
}

#[test]
fn test_parser_error_recovery() {
    // 测试解析器在遇到错误时的恢复能力
    let invalid_program = r#"
        fn test() {
            let x = ;  // 错误：缺少表达式
            let y = 42;  // 应该仍然能够解析这一行
        }
    "#;
    
    let result = parse_program(invalid_program);
    // 解析应该失败，但不应该panic
    assert!(result.is_err());
}

#[test]
fn test_unexpected_token() {
    let invalid_program = r#"
        fn test( {
            // 缺少参数和右括号
        }
    "#;
    
    let result = parse_program(invalid_program);
    assert!(result.is_err());
}

#[test]
fn test_missing_semicolon() {
    let invalid_program = r#"
        fn test() {
            let x = 42  // 缺少分号
            let y = 43;
        }
    "#;
    
    let result = parse_program(invalid_program);
    assert!(result.is_err());
}

#[test]
fn test_mismatched_braces() {
    let invalid_program = r#"
        fn test() {
            if true {
                print("hello")
            // 缺少右大括号
        }
    "#;
    
    let result = parse_program(invalid_program);
    assert!(result.is_err());
}

#[test]
fn test_invalid_struct_definition() {
    let invalid_program = r#"
        struct Point {
            x i32,  // 缺少冒号
            y: i32,
        }
    "#;
    
    let result = parse_program(invalid_program);
    assert!(result.is_err());
}

#[test]
fn test_invalid_function_definition() {
    let invalid_program = r#"
        fn add(x i32, y: i32) -> i32 {  // 参数缺少冒号
            return x + y;
        }
    "#;
    
    let result = parse_program(invalid_program);
    assert!(result.is_err());
}

#[test]
fn test_invalid_for_loop() {
    let invalid_program = r#"
        fn test() {
            for i 0..10 {  // 缺少 in 关键字
                print(i);
            }
        }
    "#;
    
    let result = parse_program(invalid_program);
    assert!(result.is_err());
}