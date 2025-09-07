// Contractus 性能测试
// 测试编译器处理大型输入的性能

use contractus::{Lexer, Parser};
use std::time::Instant;

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
fn test_large_function() {
    // 测试大型函数的解析性能
    let mut input = String::from("fn large_function() {\n");
    
    // 生成大量语句
    for i in 0..1000 {
        input.push_str(&format!("    let x{} = {};\n", i, i));
    }
    
    input.push_str("}\n");
    
    let start = Instant::now();
    let result = parse_program(&input);
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    
    // 性能要求：1000个语句应该在100ms内完成解析
    assert!(duration.as_millis() < 100, "Parsing took too long: {:?}", duration);
    
    println!("Parsed 1000 statements in {:?}", duration);
}

#[test]
fn test_large_struct() {
    // 测试大型结构体的解析性能
    let mut input = String::from("struct LargeStruct {\n");
    
    // 生成大量字段
    for i in 0..100 {
        input.push_str(&format!("    field{}: i32,\n", i));
    }
    
    input.push_str("}\n");
    
    let start = Instant::now();
    let result = parse_program(&input);
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    
    // 性能要求：100个字段应该在50ms内完成解析
    assert!(duration.as_millis() < 50, "Parsing took too long: {:?}", duration);
    
    println!("Parsed struct with 100 fields in {:?}", duration);
}

#[test]
fn test_deeply_nested_blocks() {
    // 测试深度嵌套代码块的解析性能
    let mut input = String::from("fn nested_function() {\n");
    
    // 创建10层嵌套的if语句
    for i in 0..10 {
        for _ in 0..i {
            input.push_str("    ");
        }
        input.push_str(&format!("if true {{ // level {}\n", i));
    }
    
    // 添加最内层代码
    for _ in 0..10 {
        input.push_str("    ");
    }
    input.push_str("let x = 42;\n");
    
    // 关闭嵌套块
    for i in (0..10).rev() {
        for _ in 0..i {
            input.push_str("    ");
        }
        input.push_str("}\n");
    }
    
    input.push_str("}\n");
    
    let start = Instant::now();
    let result = parse_program(&input);
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    
    // 性能要求：10层嵌套应该在50ms内完成解析
    assert!(duration.as_millis() < 50, "Parsing took too long: {:?}", duration);
    
    println!("Parsed 10-level nested blocks in {:?}", duration);
}

#[test]
fn test_complex_expressions() {
    // 测试复杂表达式的解析性能
    let mut expr = String::new();
    
    // 创建一个非常复杂的表达式
    expr.push_str("let x = ");
    for i in 0..100 {
        if i > 0 {
            expr.push_str(" + ");
        }
        expr.push_str(&format!("(a{} * b{} + c{} / d{})", i, i, i, i));
    }
    expr.push_str(";");
    
    let input = format!("fn complex_expr() {{\n    {}\n}}\n", expr);
    
    let start = Instant::now();
    let result = parse_program(&input);
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    
    // 性能要求：复杂表达式应该在50ms内完成解析
    assert!(duration.as_millis() < 50, "Parsing took too long: {:?}", duration);
    
    println!("Parsed complex expression in {:?}", duration);
}