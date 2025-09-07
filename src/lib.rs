// Contractus 编程语言编译器库
//
// 这个库包含了 Contractus 编程语言的所有核心组件：
// - 词法分析器 (Lexer)
// - 语法分析器 (Parser) - 待实现
// - 语义分析器 (Semantic Analyzer) - 待实现
// - 中间表示 (MIR) - 待实现
// - 代码生成器 (Code Generator) - 待实现

// 声明模块
pub mod ast;
pub mod lexer;
pub mod parser;
pub mod span;
pub mod token;

// 重新导出主要的公共接口
pub use ast::*;
pub use lexer::Lexer;
pub use parser::{ParseError, Parser};
pub use span::Span;
pub use token::{Token, TokenKind};
