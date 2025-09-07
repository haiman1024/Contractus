// Contractus 闭包测试
// 测试闭包的各种用法和边界情况

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
fn test_simple_closure() {
    // 测试简单闭包
    let input = r#"
        fn test() {
            let add = |x, y| x + y;
            let result = add(1, 2);
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
fn test_closure_with_types() {
    // 测试带类型的闭包
    let input = r#"
        fn test() {
            let add = |x: i32, y: i32| -> i32 { x + y };
            let result = add(1, 2);
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
fn test_closure_with_single_param() {
    // 测试单参数闭包
    let input = r#"
        fn test() {
            let double = |x| x * 2;
            let result = double(5);
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
fn test_closure_with_no_params() {
    // 测试无参数闭包
    let input = r#"
        fn test() {
            let hello = || print("Hello");
            hello();
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
fn test_closure_in_function_call() {
    // 测试作为函数参数的闭包
    let input = r#"
        fn map(arr: [i32; 3], f: fn(i32) -> i32) -> [i32; 3] {
            // ...
        }
        
        fn test() {
            let arr = [1, 2, 3];
            let doubled = map(arr, |x| x * 2);
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
fn test_closure_capture() {
    // 测试闭包捕获外部变量
    let input = r#"
        fn test() {
            let x = 10;
            let add_x = |y| x + y;
            let result = add_x(5);
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