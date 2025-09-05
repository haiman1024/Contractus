// Contractus Lexer 边界情况测试
// 测试各种边界情况和错误处理

use contractus::lexer::{Lexer, TokenKind};

#[test]
fn test_empty_input() {
    let input = "";
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Eof);
}

#[test]
fn test_whitespace_only() {
    let input = "   \t\n\r  ";
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Eof);
}

#[test]
fn test_single_line_comment_only() {
    let input = "// This is just a comment";
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Eof);
}

#[test]
fn test_multiple_comments() {
    let input = r#"
        // First comment
        fn main() {
            // Second comment
            let x = 42; // Third comment
        }
        // Fourth comment
    "#;

    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    // 验证注释被正确跳过，只有实际代码被tokenize
    let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    assert!(token_kinds.contains(&&TokenKind::Fn));
    assert!(token_kinds.contains(&&TokenKind::Let));
    assert!(token_kinds.contains(&&TokenKind::IntLiteral(42)));

    // 最后应该是 EOF
    assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof);
}

#[test]
fn test_large_numbers() {
    let input = "0 1 123 999999 2147483647";
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::IntLiteral(0));
    assert_eq!(tokens[1].kind, TokenKind::IntLiteral(1));
    assert_eq!(tokens[2].kind, TokenKind::IntLiteral(123));
    assert_eq!(tokens[3].kind, TokenKind::IntLiteral(999999));
    assert_eq!(tokens[4].kind, TokenKind::IntLiteral(2147483647));
}

#[test]
fn test_number_overflow() {
    let input = "9999999999999999999999"; // 超过 i32 范围
    let lexer = Lexer::new(input);
    let result = lexer.tokenize();

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.contains("Invalid number"));
}

#[test]
fn test_string_with_escapes() {
    let input = r#""Hello\nWorld\t\"Test\"""#;
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(
        tokens[0].kind,
        TokenKind::StringLiteral("Hello\\nWorld\\t\\\"Test\\\"".to_string())
    );
}

#[test]
fn test_empty_string() {
    let input = r#""""#;
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::StringLiteral("".to_string()));
}

#[test]
fn test_adjacent_operators() {
    let input = "==!=<=>=->..";
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
fn test_mixed_case_identifiers() {
    let input = "myVar MyStruct CONSTANT snake_case CamelCase";
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::Ident("myVar".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Ident("MyStruct".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::Ident("CONSTANT".to_string()));
    assert_eq!(tokens[3].kind, TokenKind::Ident("snake_case".to_string()));
    assert_eq!(tokens[4].kind, TokenKind::Ident("CamelCase".to_string()));
}

#[test]
fn test_keywords_as_part_of_identifiers() {
    let input = "function letme ifelse forloop structure";
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    // 这些应该被识别为标识符，不是关键字
    assert_eq!(tokens[0].kind, TokenKind::Ident("function".to_string()));
    assert_eq!(tokens[1].kind, TokenKind::Ident("letme".to_string()));
    assert_eq!(tokens[2].kind, TokenKind::Ident("ifelse".to_string()));
    assert_eq!(tokens[3].kind, TokenKind::Ident("forloop".to_string()));
    assert_eq!(tokens[4].kind, TokenKind::Ident("structure".to_string()));
}

#[test]
fn test_position_tracking_multiline() {
    let input = "fn\n  main\n    (\n      )\n        {";
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].span.line, 1); // fn
    assert_eq!(tokens[1].span.line, 2); // main
    assert_eq!(tokens[2].span.line, 3); // (
    assert_eq!(tokens[3].span.line, 4); // )
    assert_eq!(tokens[4].span.line, 5); // {
}

#[test]
fn test_all_delimiters() {
    let input = "(){}[];:,.";
    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].kind, TokenKind::LeftParen);
    assert_eq!(tokens[1].kind, TokenKind::RightParen);
    assert_eq!(tokens[2].kind, TokenKind::LeftBrace);
    assert_eq!(tokens[3].kind, TokenKind::RightBrace);
    assert_eq!(tokens[4].kind, TokenKind::LeftBracket);
    assert_eq!(tokens[5].kind, TokenKind::RightBracket);
    assert_eq!(tokens[6].kind, TokenKind::Semicolon);
    assert_eq!(tokens[7].kind, TokenKind::Colon);
    assert_eq!(tokens[8].kind, TokenKind::Comma);
    assert_eq!(tokens[9].kind, TokenKind::Dot);
}

#[test]
fn test_invalid_characters() {
    let invalid_chars = ["@", "#", "$", "%", "^", "&", "~", "`"];

    for &invalid_char in &invalid_chars {
        let input = format!("fn main() {{ let x = {}; }}", invalid_char);
        let lexer = Lexer::new(&input);
        let result = lexer.tokenize();

        assert!(
            result.is_err(),
            "Should fail for character: {}",
            invalid_char
        );
        let error = result.unwrap_err();
        assert!(
            error.contains("Unexpected character"),
            "Error should mention unexpected character for: {}",
            invalid_char
        );
    }
}

#[test]
fn test_mvp_program_from_requirements() {
    // 这是需求文档中的 MVP 程序
    let input = r#"
        struct Point {
            x: i32,
            y: i32,
        }

        fn add_points(a: Point, b: Point) -> Point {
            let result: Point = Point {
                x: a.x + b.x,
                y: a.y + b.y,
            };
            return result;
        }

        fn sum_array(arr: [i32; 5]) -> i32 {
            let total = 0;
            for item in arr {
                total = total + item;
            }
            return total;
        }

        fn main() -> i32 {
            let p1 = Point { x: 10, y: 20 };
            let p2 = Point { x: 30, y: 40 };
            let sum = add_points(p1, p2);

            let numbers: [i32; 5] = [1, 2, 3, 4, 5];
            let array_sum = sum_array(numbers);

            print(sum.x);      // 输出 40
            print(array_sum);  // 输出 15
            return 0;
        }
    "#;

    let lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();

    // 验证关键结构存在
    let token_kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();

    // 结构体相关
    assert!(token_kinds.contains(&&TokenKind::Struct));
    assert!(token_kinds.contains(&&TokenKind::Ident("Point".to_string())));

    // 函数相关
    assert!(token_kinds.contains(&&TokenKind::Fn));
    assert!(token_kinds.contains(&&TokenKind::Ident("add_points".to_string())));
    assert!(token_kinds.contains(&&TokenKind::Ident("sum_array".to_string())));
    assert!(token_kinds.contains(&&TokenKind::Ident("main".to_string())));

    // for 循环相关
    assert!(token_kinds.contains(&&TokenKind::For));
    assert!(token_kinds.contains(&&TokenKind::In));

    // 数组相关
    assert!(token_kinds.contains(&&TokenKind::LeftBracket));
    assert!(token_kinds.contains(&&TokenKind::RightBracket));

    // 类型相关
    assert!(token_kinds.contains(&&TokenKind::I32));

    // 运算符
    assert!(token_kinds.contains(&&TokenKind::Arrow));
    assert!(token_kinds.contains(&&TokenKind::Dot));
    assert!(token_kinds.contains(&&TokenKind::Plus));

    // 字面量
    assert!(token_kinds.contains(&&TokenKind::IntLiteral(10)));
    assert!(token_kinds.contains(&&TokenKind::IntLiteral(20)));
    assert!(token_kinds.contains(&&TokenKind::IntLiteral(30)));
    assert!(token_kinds.contains(&&TokenKind::IntLiteral(40)));

    // 最后是 EOF
    assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof);

    println!("MVP 程序成功 tokenize，共 {} 个 tokens", tokens.len());
}

#[test]
fn test_performance_large_input() {
    // 生成一个较大的输入来测试性能
    let mut large_input = String::new();
    for i in 0..1000 {
        large_input.push_str(&format!("let var{} = {}; ", i, i));
    }

    let start = std::time::Instant::now();
    let lexer = Lexer::new(&large_input);
    let tokens = lexer.tokenize().unwrap();
    let duration = start.elapsed();

    // 验证结果正确性
    assert_eq!(tokens.len(), 5001); // 1000 * (let + var + = + number + ;) + EOF

    // 性能检查：应该在合理时间内完成
    assert!(
        duration.as_millis() < 100,
        "Tokenization took too long: {:?}",
        duration
    );

    println!("Large input tokenization took: {:?}", duration);
}
