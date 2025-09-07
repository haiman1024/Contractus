// Contractus Lexer - 为自举优化的高效词法分析器
// 设计原则：
// 1. 直接字节操作，避免 UTF-8 解码开销
// 2. 最小化内存分配
// 3. 内联关键路径
// 4. 预分配 token 向量

use crate::span::Span;
use crate::token::{Token, TokenKind};
use std::fmt;

// 高效的词法分析器 - 为自举优化
pub struct Lexer<'a> {
    input: &'a [u8],    // 直接操作字节，最高效
    pos: usize,         // 当前位置
    current: u8,        // 当前字符（避免重复索引）
    line: u32,          // 行号
    column: u32,        // 列号
    line_start: usize,  // 当前行的起始位置
    tokens: Vec<Token>, // 预分配的 token 向量
}

impl<'a> Lexer<'a> {
    // 构造函数 - 预分配合理大小的向量
    pub fn new(input: &'a str) -> Self {
        let bytes = input.as_bytes();
        let estimated_tokens = input.len() / 6; // 经验值：平均6字符一个token

        Self {
            input: bytes,
            pos: 0,
            current: if bytes.is_empty() { 0 } else { bytes[0] },
            line: 1,
            column: 1,
            line_start: 0,
            tokens: Vec::with_capacity(estimated_tokens),
        }
    }

    // 主要的 tokenize 方法
    pub fn tokenize(mut self) -> Result<Vec<Token>, Vec<String>> {
        let mut errors = Vec::new();

        while !self.is_eof() {
            self.skip_whitespace_and_comments();

            if self.is_eof() {
                break;
            }

            let start_pos = self.pos;
            let start_line = self.line;
            let start_column = self.column;

            match self.next_token_kind() {
                Ok(kind) => {
                    let span = Span::new(start_pos, self.pos, start_line, start_column);
                    let raw =
                        String::from_utf8_lossy(&self.input[span.start..span.end]).to_string();
                    self.tokens.push(Token::new(kind, span, raw));
                }
                Err(err) => {
                    errors.push(err);
                    self.advance(); // jump error char
                }
            }
        }

        // 添加 EOF token
        let span = Span::new(self.pos, self.pos, self.line, self.column);
        self.tokens
            .push(Token::new(TokenKind::Eof, span, String::new()));

        if errors.is_empty() {
            Ok(self.tokens)
        } else {
            Err(errors)
        }
    }

    // 核心 token 识别 - 内联优化
    #[inline]
    fn next_token_kind(&mut self) -> Result<TokenKind, String> {
        match self.current {
            b'0'..=b'9' => self.scan_number(),
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.scan_identifier_or_keyword(),
            b'"' => self.scan_string(),
            b'\'' => self.scan_char(),

            // 单字符 token
            b'(' => {
                self.advance();
                Ok(TokenKind::LeftParen)
            }
            b')' => {
                self.advance();
                Ok(TokenKind::RightParen)
            }
            b'{' => {
                self.advance();
                Ok(TokenKind::LeftBrace)
            }
            b'}' => {
                self.advance();
                Ok(TokenKind::RightBrace)
            }
            b'[' => {
                self.advance();
                Ok(TokenKind::LeftBracket)
            }
            b']' => {
                self.advance();
                Ok(TokenKind::RightBracket)
            }
            b';' => {
                self.advance();
                Ok(TokenKind::Semicolon)
            }
            b',' => {
                self.advance();
                Ok(TokenKind::Comma)
            }
            b'@' => {
                self.advance();
                Ok(TokenKind::At)
            }
            b'?' => {
                self.advance();
                Ok(TokenKind::Question)
            }
            b'~' => {
                self.advance();
                Ok(TokenKind::BitwiseNot)
            }

            // 可能是多字符的 token
            b'+' => {
                self.advance();
                if self.current == b'=' {
                    self.advance();
                    Ok(TokenKind::PlusAssign)
                } else {
                    Ok(TokenKind::Plus)
                }
            }

            b'-' => {
                self.advance();
                if self.current == b'>' {
                    self.advance();
                    Ok(TokenKind::Arrow)
                } else if self.current == b'=' {
                    self.advance();
                    Ok(TokenKind::MinusAssign)
                } else {
                    Ok(TokenKind::Minus)
                }
            }

            b'*' => {
                self.advance();
                if self.current == b'=' {
                    self.advance();
                    Ok(TokenKind::StarAssign)
                } else {
                    Ok(TokenKind::Star)
                }
            }

            b'/' => {
                self.advance();
                if self.current == b'=' {
                    self.advance();
                    Ok(TokenKind::SlashAssign)
                } else {
                    Ok(TokenKind::Slash)
                }
            }

            b'%' => {
                self.advance();
                Ok(TokenKind::Percent)
            }

            b'=' => {
                self.advance();
                if self.current == b'=' {
                    self.advance();
                    Ok(TokenKind::Equal)
                } else if self.current == b'>' {
                    self.advance();
                    Ok(TokenKind::FatArrow)
                } else {
                    Ok(TokenKind::Assign)
                }
            }

            b'!' => {
                self.advance();
                if self.current == b'=' {
                    self.advance();
                    Ok(TokenKind::NotEqual)
                } else {
                    Ok(TokenKind::LogicalNot)
                }
            }

            b'<' => {
                self.advance();
                if self.current == b'=' {
                    self.advance();
                    Ok(TokenKind::LessEqual)
                } else if self.current == b'<' {
                    self.advance();
                    Ok(TokenKind::LeftShift)
                } else {
                    Ok(TokenKind::Less)
                }
            }

            b'>' => {
                self.advance();
                if self.current == b'=' {
                    self.advance();
                    Ok(TokenKind::GreaterEqual)
                } else if self.current == b'>' {
                    self.advance();
                    Ok(TokenKind::RightShift)
                } else {
                    Ok(TokenKind::Greater)
                }
            }

            b'&' => {
                self.advance();
                if self.current == b'&' {
                    self.advance();
                    Ok(TokenKind::LogicalAnd)
                } else {
                    Ok(TokenKind::BitwiseAnd)
                }
            }

            b'|' => {
                self.advance();
                if self.current == b'|' {
                    self.advance();
                    Ok(TokenKind::LogicalOr)
                } else {
                    Ok(TokenKind::BitwiseOr)
                }
            }

            b'^' => {
                self.advance();
                Ok(TokenKind::BitwiseXor)
            }

            b':' => {
                self.advance();
                if self.current == b':' {
                    self.advance();
                    Ok(TokenKind::DoubleColon)
                } else {
                    Ok(TokenKind::Colon)
                }
            }

            b'.' => {
                self.advance();
                if self.current == b'.' {
                    self.advance();
                    if self.current == b'=' {
                        self.advance();
                        Ok(TokenKind::DotDotEqual)
                    } else {
                        Ok(TokenKind::DotDot)
                    }
                } else {
                    Ok(TokenKind::Dot)
                }
            }

            c => Err(format!(
                "Unexpected character '{}' at line {}, column {}",
                c as char, self.line, self.column
            )),
        }
    }

    // 数字扫描 - 优化的整数解析
    #[inline]
    fn scan_number(&mut self) -> Result<TokenKind, String> {
        let start = self.pos;

        // 处理十六进制
        if self.current == b'0' && self.peek() == b'x' {
            self.advance();
            self.advance();

            while self.current.is_ascii_hexdigit() || self.current == b'_' {
                self.advance();
            }

            let num_str =
                String::from_utf8_lossy(&self.input[start + 2..self.pos]).replace('_', "");

            match i32::from_str_radix(&num_str, 16) {
                Ok(value) => return Ok(TokenKind::IntLiteral(value)),
                Err(_) => return Err(format!("Invalid hexadecimal number at line {}", self.line)),
            }
        }

        // 处理二进制
        if self.current == b'0' && self.peek() == b'b' {
            self.advance();
            self.advance();

            while self.current == b'0' || self.current == b'1' || self.current == b'_' {
                self.advance();
            }

            let num_str =
                String::from_utf8_lossy(&self.input[start + 2..self.pos]).replace('_', "");

            match i32::from_str_radix(&num_str, 2) {
                Ok(value) => return Ok(TokenKind::IntLiteral(value)),
                Err(_) => return Err(format!("Invalid binary number at line {}", self.line)),
            }
        }

        // 扫描整数部分
        while self.current.is_ascii_digit() || self.current == b'_' {
            self.advance();
        }

        // 检查是否是浮点数
        if self.current == b'.' && self.peek().is_ascii_digit() {
            // 暂时跳过浮点数支持，可以后续添加
            return Err(format!(
                "Float literals not yet supported at line {}",
                self.line
            ));
        }

        // 解析整数
        let num_str = String::from_utf8_lossy(&self.input[start..self.pos]).replace('_', "");

        match num_str.parse::<i32>() {
            Ok(value) => Ok(TokenKind::IntLiteral(value)),
            Err(_) => Err(format!(
                "Invalid number '{}' at line {}",
                num_str, self.line
            )),
        }
    }

    // 标识符和关键字扫描 - 使用完美哈希或跳转表优化
    #[inline]
    fn scan_identifier_or_keyword(&mut self) -> Result<TokenKind, String> {
        let start = self.pos;

        // 扫描标识符字符
        while self.current.is_ascii_alphanumeric() || self.current == b'_' {
            self.advance();
        }

        let ident_bytes = &self.input[start..self.pos];
        let ident = unsafe {
            // 安全：我们知道这些都是有效的 ASCII 字符
            std::str::from_utf8_unchecked(ident_bytes)
        };

        // 关键字识别 - 按长度和频率优化
        Ok(match ident {
            // 关键字
            "fn" => TokenKind::Fn,
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            "return" => TokenKind::Return,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "struct" => TokenKind::Struct,
            "enum" => TokenKind::Enum,
            "match" => TokenKind::Match,
            "import" => TokenKind::Import,
            "export" => TokenKind::Export,
            "pub" => TokenKind::Pub,
            "const" => TokenKind::Const,
            "static" => TokenKind::Static,
            "as" => TokenKind::As,

            // 类型
            "i8" => TokenKind::I8,
            "i16" => TokenKind::I16,
            "i32" => TokenKind::I32,
            "i64" => TokenKind::I64,
            "u8" => TokenKind::U8,
            "u16" => TokenKind::U16,
            "u32" => TokenKind::U32,
            "u64" => TokenKind::U64,
            "usize" => TokenKind::Usize,
            "isize" => TokenKind::Isize,
            "f32" => TokenKind::F32,
            "f64" => TokenKind::F64,
            "bool" => TokenKind::Bool,
            "char" => TokenKind::Char,
            "string" => TokenKind::String,

            // 布尔字面量
            "true" => TokenKind::BoolLiteral(true),
            "false" => TokenKind::BoolLiteral(false),

            // 特殊标识符
            "_" => TokenKind::Underscore,

            // 普通标识符
            _ => TokenKind::Ident(ident.to_string()),
        })
    }

    // 字符串扫描
    fn scan_string(&mut self) -> Result<TokenKind, String> {
        self.advance(); // 跳过开始的 "
        let mut string = String::new();

        while !self.is_eof() && self.current != b'"' {
            if self.current == b'\\' {
                self.advance();
                if !self.is_eof() {
                    let escaped = match self.current {
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        b'\\' => '\\',
                        b'"' => '"',
                        b'0' => '\0',
                        c => {
                            return Err(format!(
                                "Invalid escape sequence '\\{}' at line {}",
                                c as char, self.line
                            ));
                        }
                    };
                    string.push(escaped);
                    self.advance();
                }
            } else {
                string.push(self.current as char);
                self.advance();
            }
        }

        if self.is_eof() {
            return Err(format!("Unterminated string at line {}", self.line));
        }

        self.advance(); // 跳过结束的 "
        Ok(TokenKind::StringLiteral(string))
    }

    #[inline]
    fn scan_char(&mut self) -> Result<TokenKind, String> {
        self.advance(); // 跳过开始的 '

        if self.is_eof() {
            return Err(format!(
                "Unterminated character literal at line {}",
                self.line
            ));
        }

        let ch = if self.current == b'\\' {
            self.advance();
            if self.is_eof() {
                return Err(format!(
                    "Unterminated character literal at line {}",
                    self.line
                ));
            }
            match self.current {
                b'n' => '\n',
                b'r' => '\r',
                b't' => '\t',
                b'\\' => '\\',
                b'\'' => '\'',
                b'0' => '\0',
                c => {
                    return Err(format!(
                        "Invalid escape sequence '\\{}' in character literal at line {}",
                        c as char, self.line
                    ));
                }
            }
        } else {
            self.current as char
        };

        self.advance();

        if self.current != b'\'' {
            return Err(format!(
                "Character literal must be exactly one character at line {}",
                self.line
            ));
        }

        self.advance(); // 跳过结束的 '
        Ok(TokenKind::CharLiteral(ch))
    }

    // 跳过空白字符和注释 - 优化的版本
    #[inline]
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.current {
                b' ' | b'\t' | b'\r' => {
                    self.advance();
                }
                b'\n' => {
                    self.line += 1;
                    self.column = 1;
                    self.pos += 1;
                    self.line_start = self.pos;
                    if self.pos < self.input.len() {
                        self.current = self.input[self.pos];
                    } else {
                        self.current = 0;
                    }
                }
                b'/' if self.peek() == b'/' => {
                    // 单行注释
                    while !self.is_eof() && self.current != b'\n' {
                        self.advance();
                    }
                }
                b'/' if self.peek() == b'*' => {
                    // 多行注释
                    self.advance(); // /
                    self.advance(); // *

                    while !self.is_eof() {
                        if self.current == b'*' && self.peek() == b'/' {
                            self.advance(); // *
                            self.advance(); // /
                            break;
                        }
                        if self.current == b'\n' {
                            self.line += 1;
                            self.column = 0;
                            self.line_start = self.pos + 1;
                        }
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    // 内联的辅助方法
    #[inline]
    fn advance(&mut self) {
        if self.pos < self.input.len() {
            self.pos += 1;
            self.column += 1;
            self.current = if self.pos < self.input.len() {
                self.input[self.pos]
            } else {
                0
            };
        }
    }

    #[inline]
    fn peek(&self) -> u8 {
        if self.pos + 1 < self.input.len() {
            self.input[self.pos + 1]
        } else {
            0
        }
    }

    #[inline]
    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}

// 实现 Display trait 用于调试
impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // --- 字面量 ---
            TokenKind::IntLiteral(n) => write!(f, "{}", n),
            TokenKind::BoolLiteral(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            TokenKind::StringLiteral(s) => write!(f, "\"{}\"", s.escape_debug()), // 安全转义
            TokenKind::CharLiteral(c) => write!(f, "'{}'", c.escape_debug()),     // 安全转义

            // --- 标识符 ---
            TokenKind::Ident(name) => write!(f, "{}", name),

            // --- 关键字 ---
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Mut => write!(f, "mut"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::While => write!(f, "while"),
            TokenKind::For => write!(f, "for"),
            TokenKind::In => write!(f, "in"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::Struct => write!(f, "struct"),
            TokenKind::Enum => write!(f, "enum"),
            TokenKind::Match => write!(f, "match"),
            TokenKind::Import => write!(f, "import"),
            TokenKind::Export => write!(f, "export"),
            TokenKind::Pub => write!(f, "pub"),
            TokenKind::Const => write!(f, "const"),
            TokenKind::Static => write!(f, "static"),
            TokenKind::As => write!(f, "as"),

            // --- 类型关键字 ---
            TokenKind::I8 => write!(f, "i8"),
            TokenKind::I16 => write!(f, "i16"),
            TokenKind::I32 => write!(f, "i32"),
            TokenKind::I64 => write!(f, "i64"),
            TokenKind::U8 => write!(f, "u8"),
            TokenKind::U16 => write!(f, "u16"),
            TokenKind::U32 => write!(f, "u32"),
            TokenKind::U64 => write!(f, "u64"),
            TokenKind::Usize => write!(f, "usize"),
            TokenKind::Isize => write!(f, "isize"),
            TokenKind::F32 => write!(f, "f32"),
            TokenKind::F64 => write!(f, "f64"),
            TokenKind::Bool => write!(f, "bool"),
            TokenKind::Char => write!(f, "char"),
            TokenKind::String => write!(f, "string"),

            // --- 运算符 ---
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),

            TokenKind::Assign => write!(f, "="),
            TokenKind::PlusAssign => write!(f, "+="),
            TokenKind::MinusAssign => write!(f, "-="),
            TokenKind::StarAssign => write!(f, "*="),
            TokenKind::SlashAssign => write!(f, "/="),

            TokenKind::Equal => write!(f, "=="),
            TokenKind::NotEqual => write!(f, "!="),
            TokenKind::Less => write!(f, "<"),
            TokenKind::Greater => write!(f, ">"),
            TokenKind::LessEqual => write!(f, "<="),
            TokenKind::GreaterEqual => write!(f, ">="),

            TokenKind::LogicalAnd => write!(f, "&&"),
            TokenKind::LogicalOr => write!(f, "||"),
            TokenKind::LogicalNot => write!(f, "!"),

            TokenKind::BitwiseAnd => write!(f, "&"),
            TokenKind::BitwiseOr => write!(f, "|"),
            TokenKind::BitwiseXor => write!(f, "^"),
            TokenKind::BitwiseNot => write!(f, "~"),
            TokenKind::LeftShift => write!(f, "<<"),
            TokenKind::RightShift => write!(f, ">>"),

            // --- 分隔符 ---
            TokenKind::LeftParen => write!(f, "("),
            TokenKind::RightParen => write!(f, ")"),
            TokenKind::LeftBrace => write!(f, "{{"),
            TokenKind::RightBrace => write!(f, "}}"),
            TokenKind::LeftBracket => write!(f, "["),
            TokenKind::RightBracket => write!(f, "]"),

            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::DoubleColon => write!(f, "::"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Dot => write!(f, "."),
            TokenKind::DotDot => write!(f, ".."),
            TokenKind::DotDotEqual => write!(f, "..="),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::FatArrow => write!(f, "=>"),
            TokenKind::Question => write!(f, "?"),
            TokenKind::At => write!(f, "@"),
            TokenKind::Underscore => write!(f, "_"),

            // --- 特殊 ---
            TokenKind::Newline => write!(f, "\\n"),
            TokenKind::Eof => write!(f, "EOF"),
            TokenKind::Error(msg) => write!(f, "error({})", msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let token = Token {
            kind: TokenKind::Fn,
            span: Span {
                start: 0,
                end: 2,
                line: 1,
                column: 1,
            },
            raw: "fn".to_string(),
        };
        assert_eq!(token.kind, TokenKind::Fn);
    }

    #[test]
    fn test_span_creation() {
        let span = Span {
            start: 0,
            end: 5,
            line: 1,
            column: 1,
        };
        assert_eq!(span.start, 0);
        assert_eq!(span.end, 5);
    }

    #[test]
    fn test_lexer_creation() {
        let lexer = Lexer::new("test");
        assert_eq!(lexer.pos, 0);
        assert_eq!(lexer.line, 1);
        assert_eq!(lexer.column, 1);
    }

    #[test]
    fn test_token_display() {
        assert_eq!(format!("{}", TokenKind::Fn), "fn");
        assert_eq!(format!("{}", TokenKind::IntLiteral(42)), "42");
        assert_eq!(format!("{}", TokenKind::Ident("test".to_string())), "test");
    }

    #[test]
    fn test_keyword_recognition() {
        let input = "fn let if else for while struct return in";
        let lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Fn);
        assert_eq!(tokens[1].kind, TokenKind::Let);
        assert_eq!(tokens[2].kind, TokenKind::If);
        assert_eq!(tokens[3].kind, TokenKind::Else);
        assert_eq!(tokens[4].kind, TokenKind::For);
        assert_eq!(tokens[5].kind, TokenKind::While);
        assert_eq!(tokens[6].kind, TokenKind::Struct);
        assert_eq!(tokens[7].kind, TokenKind::Return);
        assert_eq!(tokens[8].kind, TokenKind::In);
    }

    #[test]
    fn test_number_parsing() {
        let input = "0 42 123 999";
        let lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::IntLiteral(0));
        assert_eq!(tokens[1].kind, TokenKind::IntLiteral(42));
        assert_eq!(tokens[2].kind, TokenKind::IntLiteral(123));
        assert_eq!(tokens[3].kind, TokenKind::IntLiteral(999));
    }

    #[test]
    fn test_operator_parsing() {
        let input = "+ - * / = == != < > <= >=";
        let lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Plus);
        assert_eq!(tokens[1].kind, TokenKind::Minus);
        assert_eq!(tokens[2].kind, TokenKind::Star);
        assert_eq!(tokens[3].kind, TokenKind::Slash);
        assert_eq!(tokens[4].kind, TokenKind::Assign);
        assert_eq!(tokens[5].kind, TokenKind::Equal);
        assert_eq!(tokens[6].kind, TokenKind::NotEqual);
        assert_eq!(tokens[7].kind, TokenKind::Less);
        assert_eq!(tokens[8].kind, TokenKind::Greater);
        assert_eq!(tokens[9].kind, TokenKind::LessEqual);
        assert_eq!(tokens[10].kind, TokenKind::GreaterEqual);
    }
}
