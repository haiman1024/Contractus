// Contractus 复杂类型测试
// 测试复杂类型系统功能

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
fn test_function_types() {
    // 测试函数类型
    let input = r#"
        fn test() {
            let f: fn(i32) -> i32 = |x| x * 2;
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
fn test_nested_generic_types() {
    // 测试嵌套泛型类型
    let input = r#"
        struct Wrapper<T> {
            value: T,
        }
        
        struct Container<T, U> {
            first: Wrapper<T>,
            second: Wrapper<U>,
        }
        
        fn test() {
            let c: Container<i32, bool> = Container {
                first: Wrapper { value: 42 },
                second: Wrapper { value: true },
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
fn test_pointer_types() {
    // 测试指针类型
    let input = r#"
        fn test() {
            let x = 42;
            let ptr: *i32 = &x;
            let mut y = 10;
            let mut_ptr: *mut i32 = &mut y;
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
fn test_reference_types() {
    // 测试引用类型
    let input = r#"
        fn test() {
            let x = 42;
            let r: &i32 = &x;
            let mut y = 10;
            let mut_r: &mut i32 = &mut y;
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
fn test_array_types() {
    // 测试数组类型
    let input = r#"
        fn test() {
            let fixed_array: [i32; 5] = [1, 2, 3, 4, 5];
            let slice: [i32] = [1, 2, 3];
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
fn test_tuple_types() {
    // 测试元组类型
    let input = r#"
        fn test() {
            let pair: (i32, bool) = (42, true);
            let triple: (i32, bool, &str) = (42, true, "hello");
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
fn test_complex_generic_function() {
    // 测试复杂泛型函数
    let input = r#"
        fn map<T, U>(arr: [T; 3], f: fn(T) -> U) -> [U; 3] {
            // ...
        }
        
        fn test() {
            let numbers = [1, 2, 3];
            let strings = map(numbers, |x| "number");
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