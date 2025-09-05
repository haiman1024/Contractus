use std::env;
use std::fs;
use std::process;

pub mod lexer;

use lexer::Lexer;

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
    let lexer: Lexer<'_> = Lexer::new(&source);
    match lexer.tokenize() {
        Ok(tokens) => {
            println!("=== Tokens ===");
            for (i, token) in tokens.iter().enumerate() {
                println!("{:3}: {:?}", i, token);
            }
            println!("Total tokens: {}", tokens.len());
        }
        Err(err) => {
            eprintln!("Lexical analysis failed: {}", err);
            process::exit(1);
        }
    }
}
