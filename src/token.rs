use crate::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // 字面量
    IntLiteral(i32),
    BoolLiteral(bool),
    StringLiteral(String),
    CharLiteral(char),

    // 标识符
    Ident(String),

    // 关键字
    Fn,
    Let,
    Mut,
    Return,
    If,
    Else,
    While,
    For,
    In,
    Break,
    Continue,
    Struct,
    Enum,
    Match,
    Import,
    Export,
    Pub,
    Const,
    Static,
    As,

    // 类型关键字
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    Usize,
    Isize,
    F32,
    F64,
    Bool,
    Char,
    String,

    // 运算符
    Plus,    // +
    Minus,   // -
    Star,    // *
    Slash,   // /
    Percent, // %

    Assign,      // =
    PlusAssign,  // +=
    MinusAssign, // -=
    StarAssign,  // *=
    SlashAssign, // /=

    Equal,        // ==
    NotEqual,     // !=
    Less,         //
    Greater,      // >
    LessEqual,    // <=
    GreaterEqual, // >=

    LogicalAnd, // &&
    LogicalOr,  // ||
    LogicalNot, // !

    BitwiseAnd, // &
    BitwiseOr,  // |
    BitwiseXor, // ^
    BitwiseNot, // ~
    LeftShift,  // <<
    RightShift, // >>

    // 分隔符
    LeftParen,    // (
    RightParen,   // )
    LeftBrace,    // {
    RightBrace,   // }
    LeftBracket,  // [
    RightBracket, // ]

    Semicolon,   // ;
    Colon,       // :
    DoubleColon, // ::
    Comma,       // ,
    Dot,         // .
    DotDot,      // ..
    DotDotEqual, // ..=
    Arrow,       // ->
    FatArrow,    // =>
    Question,    // ?
    At,          // @
    Underscore,  // _

    // 特殊
    Newline, // 显式换行（某些情况下需要）
    Eof,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub raw: String, // 用于调试
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, raw: String) -> Self {
        Self { kind, span, raw }
    }
}
