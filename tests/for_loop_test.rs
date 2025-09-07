// Contractus For循环功能测试
// 测试for循环的各种用法和边界情况

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
fn test_range_for_loop() {
    // 测试范围for循环
    let input = r#"
        fn test() {
            for i in 0..10 {
                print(i);
            }
        }
    "#;
    
    let result = parse_program(input);
    if let Err(errors) = &result {
        for error in errors {
            eprintln!("Parse error: {}", error);
        }
    }
    assert!(result.is_ok());
}

#[test]
fn test_range_inclusive_for_loop() {
    // 测试包含终点的范围for循环
    let input = r#"
        fn test() {
            for i in 0..=10 {
                print(i);
            }
        }
    "#;
    
    let result = parse_program(input);
    if let Err(errors) = &result {
        for error in errors {
            eprintln!("Parse error: {}", error);
        }
    }
    assert!(result.is_ok());
}

#[test]
fn test_array_for_loop() {
    // 测试数组遍历for循环
    let input = r#"
        fn test() {
            let arr = [1, 2, 3, 4, 5];
            for item in arr {
                print(item);
            }
        }
    "#;
    
    let result = parse_program(input);
    if let Err(errors) = &result {
        for error in errors {
            eprintln!("Parse error: {}", error);
        }
    }
    assert!(result.is_ok());
}

#[test]
fn test_nested_for_loop() {
    // 测试嵌套for循环
    let input = r#"
        fn test() {
            for i in 0..3 {
                for j in 0..3 {
                    print(i * 3 + j);
                }
            }
        }
    "#;
    
    let result = parse_program(input);
    if let Err(errors) = &result {
        for error in errors {
            eprintln!("Parse error: {}", error);
        }
    }
    assert!(result.is_ok());
}

#[test]
fn test_for_loop_with_break() {
    // 测试for循环中的break语句
    let input = r#"
        fn test() {
            for i in 0..10 {
                if i == 5 {
                    break;
                }
                print(i);
            }
        }
    "#;
    
    let result = parse_program(input);
    if let Err(errors) = &result {
        for error in errors {
            eprintln!("Parse error: {}", error);
        }
    }
    assert!(result.is_ok());
}

#[test]
fn test_for_loop_with_continue() {
    // 测试for循环中的continue语句
    let input = r#"
        fn test() {
            for i in 0..10 {
                if i % 2 == 0 {
                    continue;
                }
                print(i);
            }
        }
    "#;
    
    let result = parse_program(input);
    if let Err(errors) = &result {
        for error in errors {
            eprintln!("Parse error: {}", error);
        }
    }
    assert!(result.is_ok());
}

#[test]
fn test_complex_for_loop_expression() {
    // 测试复杂的for循环表达式
    let input = r#"
        fn test() {
            let start = 0;
            let end = 10;
            for i in start..end {
                print(i);
            }
        }
    "#;
    
    let result = parse_program(input);
    if let Err(errors) = &result {
        for error in errors {
            eprintln!("Parse error: {}", error);
        }
    }
    assert!(result.is_ok());
}