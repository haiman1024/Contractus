use crate::span::Span;

#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Item {
    Function(Function),
    Struct(StructDef),
    Enum(EnumDef),
    Const(ConstDef),
    Static(StaticDef),
    Import(ImportStmt),
    Export(ExportStmt),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub visibility: Visibility,
    pub name: String,
    pub generics: Option<Generics>,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub visibility: Visibility,
    pub name: String,
    pub generics: Option<Generics>,
    pub fields: Vec<Field>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumDef {
    pub visibility: Visibility,
    pub name: String,
    pub generics: Option<Generics>,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Option<Vec<Type>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ConstDef {
    pub visibility: Visibility,
    pub name: String,
    pub ty: Type,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StaticDef {
    pub visibility: Visibility,
    pub mutable: bool,
    pub name: String,
    pub ty: Type,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ImportStmt {
    pub path: Vec<String>,
    pub alias: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ExportStmt {
    pub items: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug, Clone)]
pub struct Generics {
    pub params: Vec<GenericParam>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub visibility: Visibility,
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub pattern: Pattern,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let(LetStmt),
    Expr(ExprStmt),
    Return(ReturnStmt),
    If(IfStmt),
    While(WhileStmt),
    For(ForStmt),
    Match(MatchStmt),
    Break(BreakStmt),
    Continue(ContinueStmt),
    Block(Block),
}

#[derive(Debug, Clone)]
pub struct LetStmt {
    pub pattern: Pattern,
    pub ty: Option<Type>,
    pub init: Option<Expr>,
    pub mutable: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ExprStmt {
    pub expr: Expr,
    pub semicolon: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ReturnStmt {
    pub expr: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub cond: Expr,
    pub then_block: Block,
    pub else_block: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub cond: Expr,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub pattern: Pattern,
    pub iterable: Expr,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MatchStmt {
    pub expr: Expr,
    pub arms: Vec<MatchArm>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct BreakStmt {
    pub label: Option<String>,
    pub expr: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ContinueStmt {
    pub label: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Ident(String),
    Literal(Literal),
    Struct(String, Vec<(String, Pattern)>),
    Tuple(Vec<Pattern>),
    Or(Vec<Pattern>),
    Wildcard,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal, Span),
    Ident(String, Span),
    Binary(BinOp, Box<Expr>, Box<Expr>, Span),
    Unary(UnOp, Box<Expr>, Span),
    Call(Box<Expr>, Vec<Expr>, Span),
    MethodCall(Box<Expr>, String, Vec<Expr>, Span),
    FieldAccess(Box<Expr>, String, Span),
    IndexAccess(Box<Expr>, Box<Expr>, Span),
    StructLit(String, Vec<(String, Expr)>, Span),
    ArrayLit(Vec<Expr>, Span),
    TupleLit(Vec<Expr>, Span),
    Range(Box<Expr>, Box<Expr>, bool, Span), // inclusive flag
    Assign(Box<Expr>, Box<Expr>, Span),
    CompoundAssign(BinOp, Box<Expr>, Box<Expr>, Span),
    Block(Block, Span),
    If(Box<Expr>, Block, Option<Block>, Span),
    Match(Box<Expr>, Vec<MatchArm>, Span),
    While(Box<Expr>, Block, Span),
    For(Pattern, Box<Expr>, Block, Span),
    Break(Option<String>, Option<Box<Expr>>, Span),
    Continue(Option<String>, Span),
    Return(Option<Box<Expr>>, Span),
    Closure(Vec<Parameter>, Option<Type>, Box<Expr>, Span),
    Cast(Box<Expr>, Type, Span),
    Ref(Box<Expr>, bool, Span), // mutable flag
    Deref(Box<Expr>, Span),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // 基础类型
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
    Unit,

    // 复合类型
    Array(Box<Type>, usize),
    Slice(Box<Type>),
    Tuple(Vec<Type>),
    Pointer(Box<Type>, bool),   // mutable flag
    Reference(Box<Type>, bool), // mutable flag

    // 用户定义类型
    Named(String),
    Generic(String, Vec<Type>),

    // 函数类型
    Function(Vec<Type>, Box<Type>),

    // 特殊类型
    Never,
    Infer,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    // 算术
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // 比较
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,

    // 逻辑
    LogicalAnd,
    LogicalOr,

    // 位运算
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LeftShift,
    RightShift,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnOp {
    Neg,        // -
    LogicalNot, // !
    BitwiseNot, // ~
    Deref,      // *
    Ref,        // &
    RefMut,     // &mut
}
