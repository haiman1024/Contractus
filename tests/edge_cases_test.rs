// Contractus 边界情况测试
// 测试各种边界情况和复杂场景

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
fn test_empty_function_body() {
    // 测试空函数体
    let input = r#"
        fn test() {
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
fn test_deeply_nested_expressions() {
    // 测试深度嵌套表达式
    let input = r#"
        fn test() {
            let x = (((((1 + 2) * 3) - 4) / 5) % 6);
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
fn test_complex_struct_initialization() {
    // 测试复杂结构体初始化
    let input = r#"
        struct Point {
            x: i32,
            y: i32,
        }
        
        struct Line {
            start: Point,
            end: Point,
        }
        
        fn test() {
            let line = Line {
                start: Point { x: 0, y: 0 },
                end: Point { x: 10, y: 10 },
            };
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
fn test_multiple_operators_same_precedence() {
    // 测试相同优先级的多个操作符
    let input = r#"
        fn test() {
            let x = 1 + 2 - 3 + 4 - 5;
            let y = 10 * 2 / 5 % 3;
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
fn test_complex_control_flow() {
    // 测试复杂控制流
    let input = r#"
        fn test(x: i32, y: i32) {
            if x > 0 {
                if y > 0 {
                    while x > y {
                        x = x - 1;
                    }
                } else {
                    for i in 0..x {
                        print(i);
                    }
                }
            } else {
                match y {
                    0 => print("zero"),
                    _ => print("negative"),
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
fn test_function_with_many_parameters() {
    // 测试多参数函数
    let input = r#"
        fn test(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32) {
            let sum = a + b + c + d + e + f;
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
fn test_large_array_literal() {
    // 测试大数组字面量
    let input = r#"
        fn test() {
            let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
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