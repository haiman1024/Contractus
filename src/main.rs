use std::env;
use std::fs;
use std::process;

use contractus::{Lexer, Parser};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: contractus <file.ctx>");
        process::exit(1);
    }

    let filename: &String = &args[1];
    let source: String = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", filename, err);
            process::exit(1);
        }
    };

    println!("Contractus Compiler v0.1.0");
    println!("Compiling: {}", filename);

    // 词法分析
    let lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => {
            println!("=== Lexical Analysis ===");
            println!("Total tokens: {}", tokens.len());
            tokens
        }
        Err(errors) => {
            for err in errors {
                eprintln!("Lexical analysis failed: {}", err);
            }
            process::exit(1);
        }
    };

    // 语法分析
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(program) => {
            println!("=== Syntax Analysis ===");

            // 分别统计函数和结构体
            let mut functions = Vec::new();
            let mut structs = Vec::new();

            for item in program.items {
                match item {
                    contractus::Item::Function(func) => functions.push(func),
                    contractus::Item::Struct(struct_) => structs.push(struct_),
                    _ => (),
                }
            }

            println!("Structs: {}", structs.len());
            println!("Functions: {}", functions.len());

            for struct_ in &structs {
                println!("  struct {}: {} fields", struct_.name, struct_.fields.len());
            }

            for func in &functions {
                println!("  fn {}: {} params", func.name, func.params.len());
            }

            println!("✅ Parsing successful!");
        }
        Err(errors) => {
            eprintln!("=== Parse Errors ===");
            for error in errors {
                eprintln!("{}", error);
            }
            process::exit(1);
        }
    }
}
