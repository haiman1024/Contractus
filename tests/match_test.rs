// Contractus Match表达式测试
// 测试match表达式的各种用法和边界情况

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
fn test_simple_match() {
    // 测试简单match表达式
    let input = r#"
        fn test(x: i32) {
            match x {
                0 => print("zero"),
                1 => print("one"),
                _ => print("other"),
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
fn test_match_with_enum() {
    // 测试枚举match表达式
    let input = r#"
        enum Option<T> {
            Some(T),
            None,
        }
        
        fn test(opt: Option<i32>) {
            match opt {
                Some(value) => print(value),
                None => print("none"),
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
fn test_match_with_guard() {
    // 测试带条件的match表达式
    let input = r#"
        fn test(x: i32) {
            match x {
                n if n > 0 => print("positive"),
                n if n < 0 => print("negative"),
                _ => print("zero"),
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
fn test_nested_match() {
    // 测试嵌套match表达式
    let input = r#"
        fn test(x: i32, y: i32) {
            match x {
                0 => match y {
                    0 => print("both zero"),
                    _ => print("x zero"),
                },
                _ => print("x not zero"),
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
fn test_match_with_struct() {
    // 测试结构体match表达式
    let input = r#"
        struct Point {
            x: i32,
            y: i32,
        }
        
        fn test(p: Point) {
            match p {
                Point { x: 0, y: 0 } => print("origin"),
                Point { x: 0, y: y } => print("on y axis"),
                Point { x: x, y: 0 } => print("on x axis"),
                Point { x: x, y: y } => print("somewhere else"),
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
fn test_match_with_tuple() {
    // 测试元组match表达式
    let input = r#"
        fn test(pair: (i32, i32)) {
            match pair {
                (0, 0) => print("origin"),
                (0, y) => print("on y axis"),
                (x, 0) => print("on x axis"),
                (x, y) => print("somewhere else"),
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