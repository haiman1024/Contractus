// Contractus 语法分析器测试
// 测试语法分析器的所有功能

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
fn test_parse_function() {
    let input = r#"
        fn add(x: i32, y: i32) -> i32 {
            return x + y;
        }
    "#;
    assert!(parse_program(input).is_ok());
}

#[test]
fn test_parse_struct() {
    let input = r#"
        struct Point {
            x: i32,
            y: i32,
        }
    "#;
    assert!(parse_program(input).is_ok());
}

#[test]
fn test_parse_enum() {
    let input = r#"
        enum Option<T> {
            Some(T),
            None,
        }
    "#;
    assert!(parse_program(input).is_ok());
}

#[test]
fn test_parse_if_else() {
    let input = r#"
        fn test() {
            if x > 0 {
                print("positive");
            } else if x < 0 {
                print("negative");
            } else {
                print("zero");
            }
        }
    "#;
    assert!(parse_program(input).is_ok());
}

#[test]
fn test_parse_while_loop() {
    let input = r#"
        fn test() {
            let mut i = 0;
            while i < 10 {
                print(i);
                i = i + 1;
            }
        }
    "#;
    assert!(parse_program(input).is_ok());
}

#[test]
fn test_parse_literals() {
    // 测试文字解析已集成到程序解析中
    let input1 = r#"fn test() { let x = 42; }"#;
    let input2 = r#"fn test() { let x = true; }"#;
    let input3 = r#"fn test() { let x = false; }"#;
    let input4 = r#"fn test() { let x = 'a'; }"#;
    let input5 = r#"fn test() { let x = "hello"; }"#;
    
    assert!(parse_program(input1).is_ok());
    assert!(parse_program(input2).is_ok());
    assert!(parse_program(input3).is_ok());
    assert!(parse_program(input4).is_ok());
    assert!(parse_program(input5).is_ok());
}

#[test]
fn test_parse_binary_ops() {
    // 测试二元运算符解析
    let input1 = r#"fn test() { let x = 1 + 2; }"#;
    let input2 = r#"fn test() { let x = 3 * 4 + 5; }"#;
    let input3 = r#"fn test() { let x = 6 + 7 * 8; }"#;
    let input4 = r#"fn test() { let x = a && b || c; }"#;
    let input5 = r#"fn test() { let x = x < y && y <= z; }"#;
    
    assert!(parse_program(input1).is_ok());
    assert!(parse_program(input2).is_ok());
    assert!(parse_program(input3).is_ok());
    assert!(parse_program(input4).is_ok());
    assert!(parse_program(input5).is_ok());
}

#[test]
fn test_parse_struct_literal() {
    let input = r#"
        fn test() {
            let p = Point { 
                x: 10, 
                y: 20 
            };
        }
    "#;
    assert!(parse_program(input).is_ok());
}

#[test]
fn test_parse_array_and_index() {
    let input = r#"
        fn test() {
            let arr = [1, 2, 3, 4, 5];
            let x = arr[0];
        }
    "#;
    assert!(parse_program(input).is_ok());
}

#[test]
fn test_parse_method_call() {
    let input = r#"
        fn test() {
            let s = "hello";
            let len = s.len();
            let result = vec.push(42).pop().unwrap();
        }
    "#;
    assert!(parse_program(input).is_ok());
}

#[test]
fn test_parse_references() {
    let input = r#"
        fn test() {
            let x = 42;
            let r = &x;
            let mr = &mut x;
        }
    "#;
    assert!(parse_program(input).is_ok());
}

#[test]
fn test_parse_type_cast() {
    let input = r#"
        fn test() {
            let x = 42;
            let y = x as i64;
        }
    "#;
    assert!(parse_program(input).is_ok());
}

#[test]
fn test_parse_generics() {
    let input = r#"
        fn map<T, U>(arr: [T; 5], f: fn(T) -> U) -> [U; 5] {
            // ...
        }
        
        struct Vec<T> {
            data: *mut T,
            len: usize,
            cap: usize,
        }
    "#;
    assert!(parse_program(input).is_ok());
}