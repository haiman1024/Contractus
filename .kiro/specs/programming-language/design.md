# Contractus 编程语言技术设计文档

## 设计概述

基于需求分析，本设计文档详细描述了 Contractus —— 一个实用主义导向的系统编程语言的技术架构。设计遵循"C转译优先、增量自举、功能优先于完美"的核心原则。

## 架构设计

### 整体架构

```
源代码 (.ctx)
    ↓
┌─────────────────────────────────────────────────────────────┐
│                  Contractus 编译器                           │
├─────────────────────────────────────────────────────────────┤
│ Lexer → Parser → AST → Sema → MIR → C_CodeGen              │
├─────────────────────────────────────────────────────────────┤
│                   错误诊断系统                                │
└─────────────────────────────────────────────────────────────┘
    ↓
C 源代码 (.c)
    ↓
C 编译器 (gcc/clang)
    ↓
可执行文件
```

### 编译管道详细设计

#### 1. 词法分析器 (Lexer)

**设计原则：** 手写实现，优先错误恢复和性能

```rust
// src/lexer/mod.rs
pub struct Lexer<'a> {
    input: &'a [u8],           // 直接操作字节，性能最优
    pos: usize,                // 当前位置
    line: usize,               // 行号（错误报告用）
    column: usize,             // 列号（错误报告用）
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // 字面量
    IntLiteral(i32),
    BoolLiteral(bool),
    StringLiteral(String),

    // 标识符和关键字
    Identifier(String),
    Fn, Let, If, Else, For, While, Return, Struct, In,

    // 类型关键字
    I32, Bool, U8,

    // 运算符
    Plus, Minus, Star, Slash,
    Equal, NotEqual, Less, Greater, LessEqual, GreaterEqual,
    Assign, Arrow,

    // 分隔符
    LeftParen, RightParen,      // ( )
    LeftBrace, RightBrace,      // { }
    LeftBracket, RightBracket,  // [ ]
    Semicolon, Colon, Comma, Dot, DotDot,  // .. (范围操作符)

    // 特殊
    Eof, Error(String),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}
```

**关键特性：**

- 直接字节操作，避免UTF-8解码开销
- 内置错误恢复机制
- 精确的位置跟踪（用于错误报告）
- 关键字识别优化
- 支持结构体、数组、切片、for循环语法

#### 2. 语法分析器 (Parser)

**设计原则：** 递归下降 + Pratt parsing，手写实现

```rust
// src/parser/mod.rs
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    errors: Vec<ParseError>,
}

// AST 节点设计
#[derive(Debug, Clone)]
pub struct Program {
    pub structs: Vec<StructDef>,    // 结构体定义
    pub functions: Vec<Function>,   // 函数定义
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<Field>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let { name: String, type_: Option<Type>, init: Option<Expr>, span: Span },
    Expr { expr: Expr, span: Span },
    Return { expr: Option<Expr>, span: Span },
    If { cond: Expr, then: Block, else_: Option<Block>, span: Span },
    While { cond: Expr, body: Block, span: Span },
    For { var: String, iterable: Expr, body: Block, span: Span },  // 新增 for 循环
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal { value: Literal, span: Span },
    Ident { name: String, span: Span },
    Binary { op: BinOp, left: Box<Expr>, right: Box<Expr>, span: Span },
    Call { name: String, args: Vec<Expr>, span: Span },
    Assign { name: String, value: Box<Expr>, span: Span },

    // 结构体和数组相关
    FieldAccess { obj: Box<Expr>, field: String, span: Span },
    IndexAccess { array: Box<Expr>, index: Box<Expr>, span: Span },
    StructLit { name: String, fields: Vec<(String, Expr)>, span: Span },
    ArrayLit { elements: Vec<Expr>, span: Span },

    // 范围表达式（用于 for 循环）
    Range { start: Box<Expr>, end: Box<Expr>, span: Span },  // start..end
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // 标量类型
    I32,
    Bool,
    U8,

    // 复合类型（基础设施）
    Struct(String),             // 结构体名
    Array(Box<Type>, usize),    // 固定数组 [T; N]
    Slice(Box<Type>),           // 动态数组/字符串 [T]
    Pointer(Box<Type>),         // 指针 *T
    Function(Vec<Type>, Box<Type>), // 函数类型
    Unit,                       // 空类型
}
```

**解析策略：**

- 递归下降处理语句和声明
- Pratt parsing处理表达式优先级
- 错误恢复：遇到错误时跳到下一个同步点
- 所有节点都包含span信息
- 支持结构体定义和字面量语法
- 支持 for 循环语法：`for i in 0..10 { ... }`

#### 3. 语义分析器 (Sema)

**设计原则：** 简单的类型检查和名称解析

```rust
// src/sema/mod.rs
pub struct SemanticAnalyzer {
    scopes: Vec<Scope>,
    structs: HashMap<String, StructDef>,    // 结构体定义表
    functions: HashMap<String, FunctionSignature>,
    errors: Vec<SemanticError>,
}

#[derive(Debug, Clone)]
pub struct Scope {
    variables: HashMap<String, Variable>,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub type_: Type,
    pub mutable: bool,
    pub initialized: bool,
}

#[derive(Debug, Clone)]
pub struct StructInfo {
    pub name: String,
    pub fields: HashMap<String, (Type, usize)>, // 字段类型和偏移量
    pub size: usize,
    pub alignment: usize,
}
```

**分析阶段：**

1. **结构体收集：** 构建结构体定义表
2. **名称解析：** 构建符号表，检查未定义变量/类型
3. **类型检查：** 验证类型兼容性，结构体字段访问
4. **for循环检查：** 验证可迭代表达式类型（数组、切片、范围）
5. **内存布局：** 计算结构体大小和字段偏移
6. **控制流分析：** 检查return语句，未初始化变量使用

#### 4. 中间表示 (MIR)

**设计原则：** 简化的中级IR，便于优化和代码生成

```rust
// src/mir/mod.rs
#[derive(Debug, Clone)]
pub struct MirProgram {
    pub structs: Vec<MirStruct>,
    pub functions: Vec<MirFunction>,
}

#[derive(Debug, Clone)]
pub struct MirStruct {
    pub name: String,
    pub fields: Vec<MirField>,
    pub size: usize,
    pub alignment: usize,
}

#[derive(Debug, Clone)]
pub struct MirField {
    pub name: String,
    pub type_: Type,
    pub offset: usize,
}

#[derive(Debug, Clone)]
pub struct MirFunction {
    pub name: String,
    pub params: Vec<MirVariable>,
    pub return_type: Type,
    pub locals: Vec<MirVariable>,
    pub body: Vec<MirInstruction>,
}

#[derive(Debug, Clone)]
pub struct MirVariable {
    pub id: VarId,
    pub name: String,
    pub type_: Type,
}

#[derive(Debug, Clone)]
pub enum MirInstruction {
    // 值操作
    Const { dest: VarId, value: i32 },
    Load { dest: VarId, src: VarId },
    Store { dest: VarId, src: VarId },

    // 算术操作
    Binary { dest: VarId, op: BinOp, left: VarId, right: VarId },

    // 内存操作
    Alloc { dest: VarId, size: usize },
    Free { ptr: VarId },
    GetFieldPtr { dest: VarId, obj: VarId, field_offset: usize },
    GetElementPtr { dest: VarId, array: VarId, index: VarId, elem_size: usize },

    // 控制流
    Jump { target: BlockId },
    JumpIf { cond: VarId, target: BlockId },
    Call { dest: Option<VarId>, func: String, args: Vec<VarId> },
    Return { value: Option<VarId> },

    // for 循环相关（降级为基础循环）
    // for i in 0..10 降级为：
    // let i = 0; while i < 10 { body; i = i + 1; }
}

pub type VarId = usize;
pub type BlockId = usize;
```

**MIR特性：**

- SSA形式（静态单赋值）
- 显式的内存管理指令
- 结构体字段访问转换为指针运算
- 数组索引转换为指针运算
- for循环降级为while循环
- 简化的控制流表示

#### 5. C代码生成器 (C_CodeGen)

**设计原则：** 生成可读的C代码，利用C编译器优化

```rust
// src/codegen/c_gen.rs
pub struct CCodeGenerator {
    output: String,
    indent_level: usize,
    temp_counter: usize,
}

impl CCodeGenerator {
    pub fn generate(&mut self, program: &MirProgram) -> String {
        self.emit_header();

        // 生成结构体定义
        for struct_def in &program.structs {
            self.generate_struct(struct_def);
        }

        // 生成函数
        for function in &program.functions {
            self.generate_function(function);
        }

        self.emit_main_wrapper();
        self.output.clone()
    }

    fn emit_header(&mut self) {
        self.emit_line("#include <stdio.h>");
        self.emit_line("#include <stdlib.h>");
        self.emit_line("#include <stdint.h>");
        self.emit_line("#include <string.h>");
        self.emit_line("");
        self.emit_line("// Runtime functions");
        self.emit_line("void contractus_print_i32(int32_t value) {");
        self.emit_line("    printf(\"%d\\n\", value);");
        self.emit_line("}");
        self.emit_line("");
        self.emit_line("void* contractus_alloc(size_t size) {");
        self.emit_line("    return malloc(size);");
        self.emit_line("}");
        self.emit_line("");
        self.emit_line("void contractus_free(void* ptr) {");
        self.emit_line("    free(ptr);");
        self.emit_line("}");
        self.emit_line("");
    }

    fn generate_struct(&mut self, struct_def: &MirStruct) {
        self.emit_line(&format!("typedef struct {} {{", struct_def.name));
        self.indent_level += 1;

        for field in &struct_def.fields {
            let c_type = self.type_to_c(&field.type_);
            self.emit_line(&format!("{} {};", c_type, field.name));
        }

        self.indent_level -= 1;
        self.emit_line(&format!("}} {};", struct_def.name));
        self.emit_line("");
    }

    fn type_to_c(&self, ty: &Type) -> String {
        match ty {
            Type::I32 => "int32_t".to_string(),
            Type::Bool => "int".to_string(),
            Type::U8 => "uint8_t".to_string(),
            Type::Struct(name) => name.clone(),
            Type::Pointer(inner) => format!("{}*", self.type_to_c(inner)),
            Type::Array(inner, size) => format!("{}[{}]", self.type_to_c(inner), size),
            Type::Slice(_) => "void*".to_string(), // 简化处理
            Type::Unit => "void".to_string(),
            _ => "void".to_string(),
        }
    }
}
```

**生成策略：**

- 每个MIR结构体生成对应的C结构体
- 每个MIR函数生成对应的C函数
- 变量映射：MIR变量 → C局部变量
- 类型映射：i32 → int32_t, bool → int, struct → C struct
- 字段访问：obj.field → obj.field
- 数组访问：arr[i] → arr[i]
- for循环：已在MIR层降级为while循环
- 内存管理：contractus_alloc/contractus_free包装

## 语法定义（EBNF）

```ebnf
// Contractus 语法定义（包含 for 循环）
program     = (struct_def | function)*

struct_def  = "struct" IDENT "{" field_list "}"
field_list  = (IDENT ":" type)*

function    = "fn" IDENT "(" params? ")" ("->" type)? block
params      = param ("," param)*
param       = IDENT ":" type

type        = "i32" | "bool" | "u8"
            | "[" type "]"              // 切片
            | "[" type ";" INT "]"      // 固定数组
            | "*" type                  // 指针
            | IDENT                     // 结构体名

block       = "{" statement* "}"
statement   = let_stmt | expr_stmt | return_stmt | if_stmt | while_stmt | for_stmt

let_stmt    = "let" IDENT (":" type)? ("=" expr)? ";"
expr_stmt   = expr ";"
return_stmt = "return" expr? ";"
if_stmt     = "if" expr block ("else" block)?
while_stmt  = "while" expr block
for_stmt    = "for" IDENT "in" expr block    // 新增 for 循环

expr        = assignment
assignment  = logical_or ("=" assignment)?
logical_or  = logical_and ("||" logical_and)*
logical_and = equality ("&&" equality)*
equality    = comparison (("==" | "!=") comparison)*
comparison  = range ((">" | ">=" | "<" | "<=") range)*
range       = term (".." term)?              // 新增范围表达式
term        = factor (("+" | "-") factor)*
factor      = unary (("*" | "/") unary)*
unary       = ("!" | "-" | "*") unary | postfix
postfix     = primary ("." IDENT | "[" expr "]")*
primary     = INT | BOOL | IDENT | "(" expr ")"
            | struct_lit | array_lit | call

struct_lit  = IDENT "{" field_init_list "}"
field_init_list = (IDENT ":" expr)*
array_lit   = "[" expr_list "]"
call        = IDENT "(" expr_list? ")"
expr_list   = expr ("," expr)*
```

## For 循环设计详解

### 支持的 for 循环类型

```contractus
// 1. 范围循环（最常用）
for i in 0..10 {
    print(i);  // 输出 0 到 9
}

// 2. 数组遍历
let arr: [i32; 5] = [1, 2, 3, 4, 5];
for item in arr {
    print(item);
}

// 3. 切片遍历
fn process_slice(data: [i32]) {
    for item in data {
        print(item);
    }
}
```

### MIR 降级策略

```rust
// for i in 0..10 { body } 降级为：
// {
//     let __start = 0;
//     let __end = 10;
//     let i = __start;
//     while i < __end {
//         body;
//         i = i + 1;
//     }
// }

impl MirBuilder {
    fn lower_for_stmt(&mut self, var: &str, iterable: &Expr, body: &Block) -> Vec<MirInstruction> {
        match iterable {
            Expr::Range { start, end, .. } => {
                // 范围循环降级
                let start_var = self.lower_expr(start);
                let end_var = self.lower_expr(end);
                let loop_var = self.new_var(var, Type::I32);

                // 生成 while 循环的 MIR
                self.generate_range_loop(loop_var, start_var, end_var, body)
            }
            _ => {
                // 数组/切片遍历降级
                self.generate_array_loop(var, iterable, body)
            }
        }
    }
}
```

### C 代码生成示例

```c
// for i in 0..10 { print(i); } 生成的 C 代码：
{
    int32_t __start_0 = 0;
    int32_t __end_1 = 10;
    int32_t i = __start_0;
    while (i < __end_1) {
        contractus_print_i32(i);
        i = i + 1;
    }
}
```

## 组件接口设计

### 错误处理系统（更新）

```rust
// src/error/mod.rs
#[derive(Debug)]
pub enum CompileError {
    LexError { pos: usize, msg: String },
    ParseError { span: Span, expected: String, found: String },
    SemanticError { span: Span, msg: String },
    TypeError { span: Span, expected: Type, found: Type },
    UndefinedStruct { span: Span, name: String },
    UndefinedField { span: Span, struct_name: String, field_name: String },
    InvalidIterable { span: Span, found_type: Type },  // for 循环错误
    CodeGenError { msg: String },
}
```

### 测试框架设计（更新）

```rust
// tests/integration_tests.rs
#[test]
fn test_for_range_loop() {
    let source = r#"
        fn main() -> i32 {
            let sum = 0;
            for i in 0..5 {
                sum = sum + i;
            }
            return sum;  // 应该返回 10 (0+1+2+3+4)
        }
    "#;

    let result = compile_and_run(source);
    assert_eq!(result, 10);
}

#[test]
fn test_for_array_loop() {
    let source = r#"
        fn main() -> i32 {
            let arr: [i32; 3] = [10, 20, 30];
            let sum = 0;
            for item in arr {
                sum = sum + item;
            }
            return sum;  // 应该返回 60
        }
    "#;

    let result = compile_and_run(source);
    assert_eq!(result, 60);
}
```

## 性能考虑

### For 循环优化

1. **范围循环优化：** 编译时计算范围边界
2. **数组循环优化：** 直接指针遍历，避免边界检查
3. **循环展开：** 小范围循环可以展开
4. **强度削减：** 乘法转换为加法

### 生成代码示例

```c
// 优化前：for i in 0..1000
for (int32_t i = 0; i < 1000; i++) {
    // body
}

// 优化后：循环展开（小范围）
// for i in 0..4
{
    int32_t i = 0; /* body */
    i = 1; /* body */
    i = 2; /* body */
    i = 3; /* body */
}
```

---

这个更新的设计文档现在包含了完整的 for 循环支持，包括语法定义、语义分析、MIR 降级和 C 代码生成策略。for 循环是数组遍历的基础，对于系统编程语言来说确实是必需的特性。
