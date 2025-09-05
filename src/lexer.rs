// Contractus Lexer - 为自举优化的高效词法分析器
// 设计原则：
// 1. 直接字节操作，避免 UTF-8 解码开销
// 2. 最小化内存分配
// 3. 内联关键路径
// 4. 预分配 token 向量

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // 字面量
    IntLiteral(i32),
    BoolLiteral(bool),
    StringLiteral(String),

    // 标识符
    Ident(String),

    // 关键字
    Fn, Let, Return, If, Else, While, For, In, Struct,

    // 类型关键字
    I32, Bool, U8,

    // 运算符
    Assign,        // =
    Plus, Minus,   // + -
    Star, Slash,   // * /
    Equal, NotEqual, // == !=
    Less, Greater, LessEqual, GreaterEqual, // < > <= >=
    Arrow,         // ->

    // 分隔符
    LeftParen, RightParen,      // ( )
    LeftBrace, RightBrace,      // { }
    LeftBracket, RightBracket,  // [ ]
    Semicolon, Colon, Comma, Dot, DotDot, // ; : , . ..

    // 特殊
    Eof,
    Error(String),
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

// 高效的词法分析器 - 为自举优化
pub struct Lexer<'a> {
    input: &'a [u8],        // 直接操作字节，最高效
    pos: usize,             // 当前位置
    current: u8,            // 当前字符（避免重复索引）
    line: u32,              // 行号
    column: u32,            // 列号
    tokens: Vec<Token>,     // 预分配的 token 向量
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
            tokens: Vec::with_capacity(estimated_tokens),
        }
    }

    // 主要的 tokenize 方法
    pub fn tokenize(mut self) -> Result<Vec<Token>, String> {
        while !self.is_eof() {
            self.skip_whitespace_and_comments();

            if self.is_eof() {
                break;
            }

            let start_pos = self.pos;
            let start_line = self.line;
            let start_column = self.column;

            let kind = self.next_token_kind()?;

            let span = Span {
                start: start_pos,
                end: self.pos,
                line: start_line,
                column: start_column,
            };

            self.tokens.push(Token { kind, span });
        }

        // 添加 EOF token
        self.tokens.push(Token {
            kind: TokenKind::Eof,
            span: Span {
                start: self.pos,
                end: self.pos,
                line: self.line,
                column: self.column,
            },
        });

        Ok(self.tokens)
    }

    // 核心 token 识别 - 内联优化
    #[inline]
    fn next_token_kind(&mut self) -> Result<TokenKind, String> {
        match self.current {
            // 数字 - 最常见，放在前面
            b'0'..=b'9' => self.scan_number(),

            // 标识符和关键字 - 第二常见
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.scan_identifier_or_keyword(),

            // 单字符运算符 - 按频率排序
            b'=' => {
                self.advance();
                if self.current == b'=' {
                    self.advance();
                    Ok(TokenKind::Equal)
                } else {
                    Ok(TokenKind::Assign)
                }
            }

            b'+' => { self.advance(); Ok(TokenKind::Plus) }
            b'-' => {
                self.advance();
                if self.current == b'>' {
                    self.advance();
                    Ok(TokenKind::Arrow)
                } else {
                    Ok(TokenKind::Minus)
                }
            }

            b'*' => { self.advance(); Ok(TokenKind::Star) }
            b'/' => { self.advance(); Ok(TokenKind::Slash) }

            b'!' => {
                self.advance();
                if self.current == b'=' {
                    self.advance();
                    Ok(TokenKind::NotEqual)
                } else {
                    Err(format!("Unexpected character '!' at line {}, column {}", self.line, self.column))
                }
            }

            b'<' => {
                self.advance();
                if self.current == b'=' {
                    self.advance();
                    Ok(TokenKind::LessEqual)
                } else {
                    Ok(TokenKind::Less)
                }
            }

            b'>' => {
                self.advance();
                if self.current == b'=' {
                    self.advance();
                    Ok(TokenKind::GreaterEqual)
                } else {
                    Ok(TokenKind::Greater)
                }
            }

            // 分隔符
            b'(' => { self.advance(); Ok(TokenKind::LeftParen) }
            b')' => { self.advance(); Ok(TokenKind::RightParen) }
            b'{' => { self.advance(); Ok(TokenKind::LeftBrace) }
            b'}' => { self.advance(); Ok(TokenKind::RightBrace) }
            b'[' => { self.advance(); Ok(TokenKind::LeftBracket) }
            b']' => { self.advance(); Ok(TokenKind::RightBracket) }
            b';' => { self.advance(); Ok(TokenKind::Semicolon) }
            b':' => { self.advance(); Ok(TokenKind::Colon) }
            b',' => { self.advance(); Ok(TokenKind::Comma) }

            b'.' => {
                self.advance();
                if self.current == b'.' {
                    self.advance();
                    Ok(TokenKind::DotDot)
                } else {
                    Ok(TokenKind::Dot)
                }
            }

            // 字符串字面量
            b'"' => self.scan_string(),

            // 错误情况
            c => Err(format!("Unexpected character '{}' at line {}, column {}",
                           c as char, self.line, self.column)),
        }
    }

    // 数字扫描 - 优化的整数解析
    #[inline]
    fn scan_number(&mut self) -> Result<TokenKind, String> {
        let start = self.pos;

        // 快速扫描数字
        while self.current.is_ascii_digit() {
            self.advance();
        }

        // 直接从字节切片解析，避免 UTF-8 转换
        let num_bytes = &self.input[start..self.pos];
        let num_str = unsafe {
            // 安全：我们知道这些都是 ASCII 数字
            std::str::from_utf8_unchecked(num_bytes)
        };

        match num_str.parse::<i32>() {
            Ok(value) => Ok(TokenKind::IntLiteral(value)),
            Err(_) => Err(format!("Invalid number '{}' at line {}, column {}",
                                num_str, self.line, self.column)),
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
            // 长度2 - 最常用
            "fn" => TokenKind::Fn,
            "if" => TokenKind::If,
            "in" => TokenKind::In,

            // 长度3
            "let" => TokenKind::Let,
            "for" => TokenKind::For,
            "i32" => TokenKind::I32,
            "u8" => TokenKind::U8,

            // 长度4
            "else" => TokenKind::Else,
            "bool" => TokenKind::Bool,
            "true" => TokenKind::BoolLiteral(true),

            // 长度5
            "while" => TokenKind::While,
            "false" => TokenKind::BoolLiteral(false),

            // 长度6
            "return" => TokenKind::Return,
            "struct" => TokenKind::Struct,

            // 默认为标识符
            _ => TokenKind::Ident(ident.to_string()),
        })
    }

    // 字符串扫描
    fn scan_string(&mut self) -> Result<TokenKind, String> {
        self.advance(); // 跳过开始的 "
        let start = self.pos;

        while !self.is_eof() && self.current != b'"' {
            if self.current == b'\\' {
                self.advance(); // 跳过转义字符
                if !self.is_eof() {
                    self.advance();
                }
            } else {
                self.advance();
            }
        }

        if self.is_eof() {
            return Err(format!("Unterminated string at line {}, column {}",
                             self.line, self.column));
        }

        let string_bytes = &self.input[start..self.pos];
        let string_content = String::from_utf8_lossy(string_bytes).into_owned();

        self.advance(); // 跳过结束的 "
        Ok(TokenKind::StringLiteral(string_content))
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
                    self.advance();
                }
                b'/' if self.peek() == b'/' => {
                    // 单行注释
                    while !self.is_eof() && self.current != b'\n' {
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
    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    #[inline]
    fn peek(&self) -> u8 {
        if self.pos + 1 < self.input.len() {
            self.input[self.pos + 1]
        } else {
            0
        }
    }
}

// 实现 Display trait 用于调试
impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::IntLiteral(n) => write!(f, "{}", n),
            TokenKind::BoolLiteral(b) => write!(f, "{}", b),
            TokenKind::StringLiteral(s) => write!(f, "\"{}\"", s),
            TokenKind::Ident(s) => write!(f, "{}", s),
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::While => write!(f, "while"),
            TokenKind::For => write!(f, "for"),
            TokenKind::In => write!(f, "in"),
            TokenKind::Struct => write!(f, "struct"),
            TokenKind::I32 => write!(f, "i32"),
            TokenKind::Bool => write!(f, "bool"),
            TokenKind::U8 => write!(f, "u8"),
            TokenKind::Assign => write!(f, "="),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Equal => write!(f, "=="),
            TokenKind::NotEqual => write!(f, "!="),
            TokenKind::Less => write!(f, "<"),
            TokenKind::Greater => write!(f, ">"),
            TokenKind::LessEqual => write!(f, "<="),
            TokenKind::GreaterEqual => write!(f, ">="),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::LeftParen => write!(f, "("),
            TokenKind::RightParen => write!(f, ")"),
            TokenKind::LeftBrace => write!(f, "{{"),
            TokenKind::RightBrace => write!(f, "}}"),
            TokenKind::LeftBracket => write!(f, "["),
            TokenKind::RightBracket => write!(f, "]"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Dot => write!(f, "."),
            TokenKind::DotDot => write!(f,".."),
            TokenKind::Eof => write!(f, "EOF"),
            TokenKind::Error(msg) => write!(f, "ERROR: {}", msg),
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
            span: Span { start: 0, end: 2, line: 1, column: 1 },
        };
        assert_eq!(token.kind, TokenKind::Fn);
    }

    #[test]
    fn test_span_creation() {
        let span = Span { start: 0, end: 5, line: 1, column: 1 };
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
