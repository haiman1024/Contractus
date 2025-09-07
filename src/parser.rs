// Contractus Parser - 递归下降语法分析器
// 设计原则：
// 1. 递归下降处理语句和声明
// 2. Pratt parsing 处理表达式优先级
// 3. 错误恢复：遇到错误时跳到下一个同步点
// 4. 所有节点都包含 span 信息

use crate::ast::*;
use crate::span::Span;
use crate::token::{Token, TokenKind};

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
    pub help: Option<String>,
}

impl ParseError {
    pub fn new(message: String, span: Span) -> Self {
        Self {
            message,
            span,
            help: None,
        }
    }

    pub fn with_help(mut self, help: String) -> Self {
        self.help = Some(help);
        self
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Parse error at line {}, column {}: {}",
            self.span.line, self.span.column, self.message
        )?;
        if let Some(help) = &self.help {
            write!(f, "\nhelp: {}", help)?;
        }
        Ok(())
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    errors: Vec<ParseError>,
    panic_mode: bool,
    loop_depth: usize, // 跟踪循环嵌套深度，用于break/continue验证
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            errors: Vec::new(),
            panic_mode: false,
            loop_depth: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Program, Vec<ParseError>> {
        let start_span = self.current_span();
        let mut items = Vec::new();

        while !self.is_at_end() {
            match self.parse_item() {
                Ok(item) => {
                    items.push(item);
                    self.panic_mode = false;
                }
                Err(err) => {
                    self.errors.push(err);
                    if !self.panic_mode {
                        self.panic_mode = true;
                        self.synchronize();
                    }
                }
            }
        }

        if self.errors.is_empty() {
            let end_span = if self.current > 0 {
                self.tokens[self.current - 1].span
            } else {
                start_span
            };

            Ok(Program {
                items,
                span: start_span.merge(&end_span),
            })
        } else {
            Err(self.errors.clone())
        }
    }

    // 顶层项目解析
    fn parse_item(&mut self) -> Result<Item, ParseError> {
        // 检查可见性
        let visibility = if self.match_token(&TokenKind::Pub) {
            Visibility::Public
        } else {
            Visibility::Private
        };

        match self.current_token_kind() {
            TokenKind::Fn => self.parse_function(visibility).map(Item::Function),
            TokenKind::Struct => self.parse_struct(visibility).map(Item::Struct),
            TokenKind::Enum => self.parse_enum(visibility).map(Item::Enum),
            TokenKind::Const => self.parse_const(visibility).map(Item::Const),
            TokenKind::Static => self.parse_static(visibility).map(Item::Static),
            TokenKind::Import => self.parse_import().map(Item::Import),
            TokenKind::Export => self.parse_export().map(Item::Export),
            _ => Err(ParseError::new(
                format!(
                    "Expected item declaration, found {:?}",
                    self.current_token_kind()
                ),
                self.current_span(),
            )),
        }
    }

    // 函数解析
    fn parse_function(&mut self, visibility: Visibility) -> Result<Function, ParseError> {
        let start_span = self.current_span();
        self.consume(TokenKind::Fn, "Expected 'fn'")?;

        let name = self.expect_ident("Expected function name")?;

        // 泛型参数（可选）
        let generics = if self.match_token(&TokenKind::Less) {
            Some(self.parse_generics()?)
        } else {
            None
        };

        // 参数列表
        self.consume(TokenKind::LeftParen, "Expected '(' after function name")?;
        let mut params = Vec::new();

        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            params.push(self.parse_parameter()?);

            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        self.consume(TokenKind::RightParen, "Expected ')' after parameters")?;

        // 返回类型（可选）
        let return_type = if self.match_token(&TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // 函数体
        let body = self.parse_block()?;

        Ok(Function {
            visibility,
            name,
            generics,
            params,
            return_type,
            body,
            span: start_span.merge(&self.previous().span),
        })
    }

    // 结构体解析
    fn parse_struct(&mut self, visibility: Visibility) -> Result<StructDef, ParseError> {
        let start_span = self.current_span();
        self.consume(TokenKind::Struct, "Expected 'struct'")?;

        let name = self.expect_ident("Expected struct name")?;

        let generics = if self.match_token(&TokenKind::Less) {
            Some(self.parse_generics()?)
        } else {
            None
        };

        self.consume(TokenKind::LeftBrace, "Expected '{' after struct name")?;

        let mut fields = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let field_visibility = if self.match_token(&TokenKind::Pub) {
                Visibility::Public
            } else {
                Visibility::Private
            };

            let field_name = self.expect_ident("Expected field name")?;
            self.consume(TokenKind::Colon, "Expected ':' after field name")?;
            let field_type = self.parse_type()?;

            fields.push(Field {
                visibility: field_visibility,
                name: field_name,
                ty: field_type,
                span: self.previous().span,
            });

            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        self.consume(TokenKind::RightBrace, "Expected '}' after struct fields")?;

        Ok(StructDef {
            visibility,
            name,
            generics,
            fields,
            span: start_span.merge(&self.previous().span),
        })
    }

    // 枚举解析
    fn parse_enum(&mut self, visibility: Visibility) -> Result<EnumDef, ParseError> {
        let start_span = self.current_span();
        self.consume(TokenKind::Enum, "Expected 'enum'")?;

        let name = self.expect_ident("Expected enum name")?;

        let generics = if self.match_token(&TokenKind::Less) {
            Some(self.parse_generics()?)
        } else {
            None
        };

        self.consume(TokenKind::LeftBrace, "Expected '{' after enum name")?;

        let mut variants = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let variant_start = self.current_span();
            let variant_name = self.expect_ident("Expected variant name")?;

            let fields = if self.match_token(&TokenKind::LeftParen) {
                let mut field_types = Vec::new();
                while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
                    field_types.push(self.parse_type()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.consume(TokenKind::RightParen, "Expected ')' after variant fields")?;
                Some(field_types)
            } else {
                None
            };

            variants.push(EnumVariant {
                name: variant_name,
                fields,
                span: variant_start.merge(&self.previous().span),
            });

            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        self.consume(TokenKind::RightBrace, "Expected '}' after enum variants")?;

        Ok(EnumDef {
            visibility,
            name,
            generics,
            variants,
            span: start_span.merge(&self.previous().span),
        })
    }

    // const 解析
    fn parse_const(&mut self, visibility: Visibility) -> Result<ConstDef, ParseError> {
        let start_span = self.current_span();
        self.consume(TokenKind::Const, "Expected 'const'")?;

        let name = self.expect_ident("Expected const name")?;
        self.consume(TokenKind::Colon, "Expected ':' after const name")?;
        let ty = self.parse_type()?;
        self.consume(TokenKind::Assign, "Expected '=' after const type")?;
        let value = self.parse_expression()?;
        self.consume(TokenKind::Semicolon, "Expected ';' after const value")?;

        Ok(ConstDef {
            visibility,
            name,
            ty,
            value,
            span: start_span.merge(&self.previous().span),
        })
    }

    // static 解析
    fn parse_static(&mut self, visibility: Visibility) -> Result<StaticDef, ParseError> {
        let start_span = self.current_span();
        self.consume(TokenKind::Static, "Expected 'static'")?;

        let mutable = self.match_token(&TokenKind::Mut);
        let name = self.expect_ident("Expected static name")?;
        self.consume(TokenKind::Colon, "Expected ':' after static name")?;
        let ty = self.parse_type()?;
        self.consume(TokenKind::Assign, "Expected '=' after static type")?;
        let value = self.parse_expression()?;
        self.consume(TokenKind::Semicolon, "Expected ';' after static value")?;

        Ok(StaticDef {
            visibility,
            mutable,
            name,
            ty,
            value,
            span: start_span.merge(&self.previous().span),
        })
    }

    // import 解析
    fn parse_import(&mut self) -> Result<ImportStmt, ParseError> {
        let start_span = self.current_span();
        self.consume(TokenKind::Import, "Expected 'import'")?;

        let mut path = vec![self.expect_ident("Expected module path")?];

        while self.match_token(&TokenKind::DoubleColon) {
            path.push(self.expect_ident("Expected module name after '::'")?);
        }

        let alias = if self.match_token(&TokenKind::As) {
            Some(self.expect_ident("Expected alias name after 'as'")?)
        } else {
            None
        };

        self.consume(TokenKind::Semicolon, "Expected ';' after import")?;

        Ok(ImportStmt {
            path,
            alias,
            span: start_span.merge(&self.previous().span),
        })
    }

    // export 解析
    fn parse_export(&mut self) -> Result<ExportStmt, ParseError> {
        let start_span = self.current_span();
        self.consume(TokenKind::Export, "Expected 'export'")?;

        self.consume(TokenKind::LeftBrace, "Expected '{' after export")?;

        let mut items = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            items.push(self.expect_ident("Expected export item name")?);
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        self.consume(TokenKind::RightBrace, "Expected '}' after export items")?;
        self.consume(TokenKind::Semicolon, "Expected ';' after export")?;

        Ok(ExportStmt {
            items,
            span: start_span.merge(&self.previous().span),
        })
    }

    // 泛型参数解析
    fn parse_generics(&mut self) -> Result<Generics, ParseError> {
        let start_span = self.current_span();
        let mut params = Vec::new();

        while !self.check(&TokenKind::Greater) && !self.is_at_end() {
            let param_start = self.current_span();
            let name = self.expect_ident("Expected generic parameter name")?;

            let mut bounds = Vec::new();
            if self.match_token(&TokenKind::Colon) {
                bounds.push(self.expect_ident("Expected trait bound")?);
                while self.match_token(&TokenKind::Plus) {
                    bounds.push(self.expect_ident("Expected trait bound after '+'")?);
                }
            }

            params.push(GenericParam {
                name,
                bounds,
                span: param_start.merge(&self.previous().span),
            });

            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        self.consume(TokenKind::Greater, "Expected '>' after generic parameters")?;

        Ok(Generics {
            params,
            span: start_span.merge(&self.previous().span),
        })
    }

    // 参数解析
    fn parse_parameter(&mut self) -> Result<Parameter, ParseError> {
        let start_span = self.current_span();
        let pattern = self.parse_pattern()?;
        self.consume(TokenKind::Colon, "Expected ':' after parameter pattern")?;
        let ty = self.parse_type()?;

        Ok(Parameter {
            pattern,
            ty,
            span: start_span.merge(&self.previous().span),
        })
    }

    // 模式解析
    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        // Parse first pattern
        let first_pattern = self.parse_single_pattern()?;

        // Check for or pattern
        if self.check(&TokenKind::BitwiseOr) {
            let mut patterns = vec![first_pattern];
            while self.match_token(&TokenKind::BitwiseOr) {
                patterns.push(self.parse_single_pattern()?);
            }
            return Ok(Pattern::Or(patterns));
        }

        Ok(first_pattern)
    }

    // 单个模式解析
    fn parse_single_pattern(&mut self) -> Result<Pattern, ParseError> {
        match self.current_token_kind() {
            TokenKind::Underscore => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.advance();

                // 检查是否是结构体模式
                if self.check(&TokenKind::LeftBrace) {
                    self.advance();
                    let mut fields = Vec::new();

                    while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
                        let field_name = self.expect_ident("Expected field name in pattern")?;

                        let field_pattern = if self.match_token(&TokenKind::Colon) {
                            self.parse_pattern()?
                        } else {
                            Pattern::Ident(field_name.clone())
                        };

                        fields.push((field_name, field_pattern));

                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }

                    self.consume(
                        TokenKind::RightBrace,
                        "Expected '}' after struct pattern fields",
                    )?;
                    Ok(Pattern::Struct(name, fields))
                } else {
                    Ok(Pattern::Ident(name))
                }
            }
            TokenKind::LeftParen => {
                self.advance();
                let mut patterns = Vec::new();

                while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
                    patterns.push(self.parse_pattern()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }

                self.consume(TokenKind::RightParen, "Expected ')' after tuple pattern")?;
                Ok(Pattern::Tuple(patterns))
            }
            TokenKind::IntLiteral(n) => {
                let n = *n as i64;
                self.advance();
                Ok(Pattern::Literal(Literal::Int(n)))
            }
            TokenKind::BoolLiteral(b) => {
                let b = *b;
                self.advance();
                Ok(Pattern::Literal(Literal::Bool(b)))
            }
            _ => Err(ParseError::new(
                format!("Expected pattern, found {:?}", self.current_token_kind()),
                self.current_span(),
            )),
        }
    }

    // 类型解析
    fn parse_type(&mut self) -> Result<Type, ParseError> {
        let ty = match self.current_token_kind() {
            TokenKind::I8 => {
                self.advance();
                Type::I8
            }
            TokenKind::I16 => {
                self.advance();
                Type::I16
            }
            TokenKind::I32 => {
                self.advance();
                Type::I32
            }
            TokenKind::I64 => {
                self.advance();
                Type::I64
            }
            TokenKind::U8 => {
                self.advance();
                Type::U8
            }
            TokenKind::U16 => {
                self.advance();
                Type::U16
            }
            TokenKind::U32 => {
                self.advance();
                Type::U32
            }
            TokenKind::U64 => {
                self.advance();
                Type::U64
            }
            TokenKind::Usize => {
                self.advance();
                Type::Usize
            }
            TokenKind::Isize => {
                self.advance();
                Type::Isize
            }
            TokenKind::F32 => {
                self.advance();
                Type::F32
            }
            TokenKind::F64 => {
                self.advance();
                Type::F64
            }
            TokenKind::Bool => {
                self.advance();
                Type::Bool
            }
            TokenKind::Char => {
                self.advance();
                Type::Char
            }
            TokenKind::String => {
                self.advance();
                Type::String
            }
            TokenKind::Fn => {
                self.advance();
                self.consume(TokenKind::LeftParen, "Expected '(' after 'fn'")?;

                let mut param_types = Vec::new();
                while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
                    param_types.push(self.parse_type()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }

                self.consume(
                    TokenKind::RightParen,
                    "Expected ')' after function parameters",
                )?;
                self.consume(TokenKind::Arrow, "Expected '->' after function parameters")?;
                let return_type = Box::new(self.parse_type()?);

                Type::Function(param_types, return_type)
            }
            TokenKind::LeftBracket => {
                self.advance();
                let element_type = Box::new(self.parse_type()?);

                if self.match_token(&TokenKind::Semicolon) {
                    // 固定大小数组 [T; N]
                    if let TokenKind::IntLiteral(size) = self.current_token_kind() {
                        let size = *size as usize;
                        self.advance();
                        self.consume(TokenKind::RightBracket, "Expected ']' after array size")?;
                        Type::Array(element_type, size)
                    } else {
                        return Err(ParseError::new(
                            "Expected array size".to_string(),
                            self.current_span(),
                        ));
                    }
                } else {
                    // 切片 [T]
                    self.consume(TokenKind::RightBracket, "Expected ']' after slice type")?;
                    Type::Slice(element_type)
                }
            }

            TokenKind::Star => {
                self.advance();
                let mutable = self.match_token(&TokenKind::Mut);
                let inner = Box::new(self.parse_type()?);
                Type::Pointer(inner, mutable)
            }

            TokenKind::BitwiseAnd => {
                self.advance();
                let mutable = self.match_token(&TokenKind::Mut);
                let inner = Box::new(self.parse_type()?);
                Type::Reference(inner, mutable)
            }

            TokenKind::LeftParen => {
                self.advance();

                // 检查是否是unit类型 ()
                if self.check(&TokenKind::RightParen) {
                    self.advance();
                    Type::Unit
                } else {
                    // 元组类型或函数类型
                    let mut types = vec![self.parse_type()?];

                    while self.match_token(&TokenKind::Comma) {
                        if self.check(&TokenKind::RightParen) {
                            break;
                        }
                        types.push(self.parse_type()?);
                    }

                    self.consume(TokenKind::RightParen, "Expected ')' after tuple types")?;

                    // 检查是否是函数类型
                    if self.match_token(&TokenKind::Arrow) {
                        let return_type = Box::new(self.parse_type()?);
                        Type::Function(types, return_type)
                    } else {
                        Type::Tuple(types)
                    }
                }
            }

            TokenKind::Ident(name) => {
                let name = name.clone();
                self.advance();

                // 检查泛型参数
                if self.match_token(&TokenKind::Less) {
                    let mut args = vec![self.parse_type()?];

                    while self.match_token(&TokenKind::Comma) {
                        args.push(self.parse_type()?);
                    }

                    self.consume(TokenKind::Greater, "Expected '>' after generic arguments")?;
                    Type::Generic(name, args)
                } else {
                    Type::Named(name)
                }
            }

            TokenKind::LogicalNot => {
                self.advance();
                Type::Never
            }

            TokenKind::Underscore => {
                self.advance();
                Type::Infer
            }

            _ => {
                return Err(ParseError::new(
                    format!("Expected type, found {:?}", self.current_token_kind()),
                    self.current_span(),
                ));
            }
        };

        Ok(ty)
    }

    // 代码块解析
    fn parse_block(&mut self) -> Result<Block, ParseError> {
        let start_span = self.current_span();
        self.consume(TokenKind::LeftBrace, "Expected '{'")?;

        let mut statements = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        self.consume(TokenKind::RightBrace, "Expected '}'")?;

        Ok(Block {
            statements,
            span: start_span.merge(&self.previous().span),
        })
    }

    // 语句解析
    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.current_token_kind() {
            TokenKind::Let => Ok(Statement::Let(self.parse_let_statement()?)),
            TokenKind::Return => Ok(Statement::Return(self.parse_return_statement()?)),
            TokenKind::If => Ok(Statement::If(self.parse_if_statement()?)),
            TokenKind::While => Ok(Statement::While(self.parse_while_statement()?)),
            TokenKind::For => Ok(Statement::For(self.parse_for_statement()?)),
            TokenKind::Match => Ok(Statement::Match(self.parse_match_statement()?)),
            TokenKind::Break => Ok(Statement::Break(self.parse_break_statement()?)),
            TokenKind::Continue => Ok(Statement::Continue(self.parse_continue_statement()?)),
            TokenKind::LeftBrace => Ok(Statement::Block(self.parse_block()?)),
            _ => {
                // 表达式语句
                let expr = self.parse_expression()?;
                let semicolon = self.match_token(&TokenKind::Semicolon);

                Ok(Statement::Expr(ExprStmt {
                    span: expr.span(),
                    expr,
                    semicolon,
                }))
            }
        }
    }

    // let 语句解析
    fn parse_let_statement(&mut self) -> Result<LetStmt, ParseError> {
        let start_span = self.current_span();
        self.consume(TokenKind::Let, "Expected 'let'")?;

        // 检查是否有 mut 关键字
        let mutable = self.match_token(&TokenKind::Mut);

        let pattern = self.parse_pattern()?;

        let ty = if self.match_token(&TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let init = if self.match_token(&TokenKind::Assign) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.consume(TokenKind::Semicolon, "Expected ';' after let statement")?;

        Ok(LetStmt {
            pattern,
            ty,
            init,
            mutable,
            span: start_span.merge(&self.previous().span),
        })
    }

    // return 语句解析
    fn parse_return_statement(&mut self) -> Result<ReturnStmt, ParseError> {
        let start_span = self.current_span();
        self.consume(TokenKind::Return, "Expected 'return'")?;

        let expr = if self.check(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };

        self.consume(TokenKind::Semicolon, "Expected ';' after return statement")?;

        Ok(ReturnStmt {
            expr,
            span: start_span.merge(&self.previous().span),
        })
    }

    // if 语句解析
    fn parse_if_statement(&mut self) -> Result<IfStmt, ParseError> {
        let start_span = self.current_span();
        self.consume(TokenKind::If, "Expected 'if'")?;

        let cond = self.parse_expression()?;
        let then_block = self.parse_block()?;

        let else_block = if self.match_token(&TokenKind::Else) {
            if self.check(&TokenKind::If) {
                // else if - 递归解析为嵌套的if
                let nested_if = self.parse_if_statement()?;
                Some(Block {
                    statements: vec![Statement::If(nested_if)],
                    span: self.previous().span,
                })
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };

        Ok(IfStmt {
            cond,
            then_block,
            else_block,
            span: start_span.merge(&self.previous().span),
        })
    }

    // while 语句解析
    fn parse_while_statement(&mut self) -> Result<WhileStmt, ParseError> {
        let start_span = self.current_span();
        self.consume(TokenKind::While, "Expected 'while'")?;

        let cond = self.parse_expression()?;

        self.loop_depth += 1;
        let body = self.parse_block()?;
        self.loop_depth -= 1;

        Ok(WhileStmt {
            cond,
            body,
            span: start_span.merge(&self.previous().span),
        })
    }

    // for 语句解析
    fn parse_for_statement(&mut self) -> Result<ForStmt, ParseError> {
        let start_span = self.current_span();
        self.consume(TokenKind::For, "Expected 'for'")?;

        let pattern = self.parse_pattern()?;
        self.consume(TokenKind::In, "Expected 'in' after for loop variable")?;
        let iterable = self.parse_expression()?;

        self.loop_depth += 1;
        let body = self.parse_block()?;
        self.loop_depth -= 1;

        Ok(ForStmt {
            pattern,
            iterable,
            body,
            span: start_span.merge(&self.previous().span),
        })
    }

    // match 语句解析
    fn parse_match_statement(&mut self) -> Result<MatchStmt, ParseError> {
        let start_span = self.current_span();
        self.consume(TokenKind::Match, "Expected 'match'")?;

        let expr = self.parse_expression()?;
        self.consume(TokenKind::LeftBrace, "Expected '{' after match expression")?;

        let mut arms = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let arm_start = self.current_span();
            let pattern = self.parse_pattern()?;

            let guard = if self.match_token(&TokenKind::If) {
                Some(self.parse_expression()?)
            } else {
                None
            };

            self.consume(TokenKind::FatArrow, "Expected '=>' after match pattern")?;

            let body = if self.check(&TokenKind::LeftBrace) {
                let block = self.parse_block()?;
                Expr::Block(block.clone(), block.span)
            } else {
                self.parse_expression()?
            };

            arms.push(MatchArm {
                pattern,
                guard,
                body,
                span: arm_start.merge(&self.previous().span),
            });

            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        self.consume(TokenKind::RightBrace, "Expected '}' after match arms")?;

        Ok(MatchStmt {
            expr,
            arms,
            span: start_span.merge(&self.previous().span),
        })
    }

    // break 语句解析
    fn parse_break_statement(&mut self) -> Result<BreakStmt, ParseError> {
        let start_span = self.current_span();
        self.consume(TokenKind::Break, "Expected 'break'")?;

        if self.loop_depth == 0 {
            return Err(
                ParseError::new("break statement outside of loop".to_string(), start_span)
                    .with_help("break can only be used inside while or for loops".to_string()),
            );
        }

        let label = if let TokenKind::Ident(name) = self.current_token_kind() {
            let name = name.clone();
            self.advance();
            Some(name)
        } else {
            None
        };

        let expr = if !self.check(&TokenKind::Semicolon) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.consume(TokenKind::Semicolon, "Expected ';' after break statement")?;

        Ok(BreakStmt {
            label,
            expr,
            span: start_span.merge(&self.previous().span),
        })
    }

    // continue 语句解析
    fn parse_continue_statement(&mut self) -> Result<ContinueStmt, ParseError> {
        let start_span = self.current_span();
        self.consume(TokenKind::Continue, "Expected 'continue'")?;

        if self.loop_depth == 0 {
            return Err(ParseError::new(
                "continue statement outside of loop".to_string(),
                start_span,
            )
            .with_help("continue can only be used inside while or for loops".to_string()));
        }

        let label = if let TokenKind::Ident(name) = self.current_token_kind() {
            let name = name.clone();
            self.advance();
            Some(name)
        } else {
            None
        };

        self.consume(
            TokenKind::Semicolon,
            "Expected ';' after continue statement",
        )?;

        Ok(ContinueStmt {
            label,
            span: start_span.merge(&self.previous().span),
        })
    }

    // 表达式解析 - Pratt Parsing
    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_logical_or()?;

        if self.check(&TokenKind::Assign) {
            self.advance();
            let right = self.parse_assignment()?;

            return Ok(Expr::Assign(
                Box::new(expr.clone()),
                Box::new(right),
                expr.span().merge(&self.previous().span),
            ));
        }

        // 复合赋值运算符
        let compound_op = match self.current_token_kind() {
            TokenKind::PlusAssign => Some(BinOp::Add),
            TokenKind::MinusAssign => Some(BinOp::Sub),
            TokenKind::StarAssign => Some(BinOp::Mul),
            TokenKind::SlashAssign => Some(BinOp::Div),
            _ => None,
        };

        if let Some(op) = compound_op {
            self.advance();
            let right = self.parse_assignment()?;

            return Ok(Expr::CompoundAssign(
                op,
                Box::new(expr.clone()),
                Box::new(right),
                expr.span().merge(&self.previous().span),
            ));
        }

        Ok(expr)
    }

    fn parse_logical_or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_logical_and()?;

        while self.match_token(&TokenKind::LogicalOr) {
            let right = self.parse_logical_and()?;
            let span = expr.span().merge(&right.span());
            expr = Expr::Binary(BinOp::LogicalOr, Box::new(expr), Box::new(right), span);
        }

        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_bitwise_or()?;

        while self.match_token(&TokenKind::LogicalAnd) {
            let right = self.parse_bitwise_or()?;
            let span = expr.span().merge(&right.span());
            expr = Expr::Binary(BinOp::LogicalAnd, Box::new(expr), Box::new(right), span);
        }

        Ok(expr)
    }

    fn parse_bitwise_or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_bitwise_xor()?;

        while self.match_token(&TokenKind::BitwiseOr) {
            let right = self.parse_bitwise_xor()?;
            let span = expr.span().merge(&right.span());
            expr = Expr::Binary(BinOp::BitwiseOr, Box::new(expr), Box::new(right), span);
        }

        Ok(expr)
    }

    fn parse_bitwise_xor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_bitwise_and()?;

        while self.match_token(&TokenKind::BitwiseXor) {
            let right = self.parse_bitwise_and()?;
            let span = expr.span().merge(&right.span());
            expr = Expr::Binary(BinOp::BitwiseXor, Box::new(expr), Box::new(right), span);
        }

        Ok(expr)
    }

    fn parse_bitwise_and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_equality()?;

        while self.match_token(&TokenKind::BitwiseAnd) {
            let right = self.parse_equality()?;
            let span = expr.span().merge(&right.span());
            expr = Expr::Binary(BinOp::BitwiseAnd, Box::new(expr), Box::new(right), span);
        }

        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_comparison()?;

        while let Some(op) = self.match_binary_op(&[TokenKind::Equal, TokenKind::NotEqual]) {
            let right = self.parse_comparison()?;
            let span = expr.span().merge(&right.span());
            expr = Expr::Binary(op, Box::new(expr), Box::new(right), span);
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_shift()?;

        while let Some(op) = self.match_binary_op(&[
            TokenKind::Less,
            TokenKind::Greater,
            TokenKind::LessEqual,
            TokenKind::GreaterEqual,
        ]) {
            let right = self.parse_shift()?;
            let span = expr.span().merge(&right.span());
            expr = Expr::Binary(op, Box::new(expr), Box::new(right), span);
        }

        Ok(expr)
    }

    fn parse_shift(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_range()?;

        while let Some(op) = self.match_binary_op(&[TokenKind::LeftShift, TokenKind::RightShift]) {
            let right = self.parse_range()?;
            let span = expr.span().merge(&right.span());
            expr = Expr::Binary(op, Box::new(expr), Box::new(right), span);
        }

        Ok(expr)
    }

    fn parse_range(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_additive()?;

        if self.match_token(&TokenKind::DotDot) {
            let inclusive = self.match_token(&TokenKind::Assign); // ..=
            let end = self.parse_additive()?;
            let span = expr.span().merge(&end.span());
            expr = Expr::Range(Box::new(expr), Box::new(end), inclusive, span);
        }

        Ok(expr)
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_multiplicative()?;

        while let Some(op) = self.match_binary_op(&[TokenKind::Plus, TokenKind::Minus]) {
            let right = self.parse_multiplicative()?;
            let span = expr.span().merge(&right.span());
            expr = Expr::Binary(op, Box::new(expr), Box::new(right), span);
        }

        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_cast()?;

        while let Some(op) =
            self.match_binary_op(&[TokenKind::Star, TokenKind::Slash, TokenKind::Percent])
        {
            let right = self.parse_cast()?;
            let span = expr.span().merge(&right.span());
            expr = Expr::Binary(op, Box::new(expr), Box::new(right), span);
        }

        Ok(expr)
    }

    fn parse_cast(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_unary()?;

        if self.match_token(&TokenKind::As) {
            let ty = self.parse_type()?;
            let span = expr.span().merge(&self.previous().span);
            expr = Expr::Cast(Box::new(expr), ty, span);
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current_span();

        let op = match self.current_token_kind() {
            TokenKind::Minus => Some(UnOp::Neg),
            TokenKind::LogicalNot => Some(UnOp::LogicalNot),
            TokenKind::BitwiseNot => Some(UnOp::BitwiseNot),
            TokenKind::Star => Some(UnOp::Deref),
            TokenKind::BitwiseAnd => {
                self.advance();
                if self.match_token(&TokenKind::Mut) {
                    let expr = self.parse_unary()?;
                    return Ok(Expr::Unary(
                        UnOp::RefMut,
                        Box::new(expr),
                        start_span.merge(&self.previous().span),
                    ));
                } else {
                    let expr = self.parse_unary()?;
                    return Ok(Expr::Unary(
                        UnOp::Ref,
                        Box::new(expr),
                        start_span.merge(&self.previous().span),
                    ));
                }
            }
            _ => None,
        };

        if let Some(op) = op {
            self.advance();
            let expr = self.parse_unary()?;
            Ok(Expr::Unary(
                op,
                Box::new(expr),
                start_span.merge(&self.previous().span),
            ))
        } else {
            self.parse_postfix()
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;

        loop {
            match self.current_token_kind() {
                TokenKind::LeftParen => {
                    // 函数调用或方法调用
                    self.advance();
                    let args = self.parse_args()?;
                    self.consume(TokenKind::RightParen, "Expected ')' after arguments")?;
                    let span = expr.span().merge(&self.previous().span);
                    expr = Expr::Call(Box::new(expr), args, span);
                }

                TokenKind::Dot => {
                    self.advance();

                    if let TokenKind::Ident(name) = self.current_token_kind() {
                        let name = name.clone();
                        self.advance();

                        if self.check(&TokenKind::LeftParen) {
                            // 方法调用
                            self.advance();
                            let args = self.parse_args()?;
                            self.consume(
                                TokenKind::RightParen,
                                "Expected ')' after method arguments",
                            )?;
                            let span = expr.span().merge(&self.previous().span);
                            expr = Expr::MethodCall(Box::new(expr), name, args, span);
                        } else {
                            // 字段访问
                            let span = expr.span().merge(&self.previous().span);
                            expr = Expr::FieldAccess(Box::new(expr), name, span);
                        }
                    } else {
                        return Err(ParseError::new(
                            "Expected identifier after '.'".to_string(),
                            self.current_span(),
                        ));
                    }
                }

                TokenKind::LeftBracket => {
                    // 索引访问
                    self.advance();
                    let index = self.parse_expression()?;
                    self.consume(TokenKind::RightBracket, "Expected ']' after index")?;
                    let span = expr.span().merge(&self.previous().span);
                    expr = Expr::IndexAccess(Box::new(expr), Box::new(index), span);
                }

                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let start_span = self.current_span();

        match self.current_token_kind() {
            TokenKind::IntLiteral(n) => {
                let n = *n as i64;
                self.advance();
                Ok(Expr::Literal(Literal::Int(n), start_span))
            }

            TokenKind::BoolLiteral(b) => {
                let b = *b;
                self.advance();
                Ok(Expr::Literal(Literal::Bool(b), start_span))
            }

            TokenKind::CharLiteral(c) => {
                let c = *c;
                self.advance();
                Ok(Expr::Literal(Literal::Char(c), start_span))
            }

            TokenKind::StringLiteral(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::Literal(Literal::String(s), start_span))
            }

            TokenKind::Ident(name) => {
                let name = name.clone();
                self.advance();

                // 检查是否是结构体字面量
                if self.check(&TokenKind::LeftBrace) {
                    self.advance();
                    let fields = self.parse_struct_fields()?;
                    self.consume(TokenKind::RightBrace, "Expected '}' after struct fields")?;
                    Ok(Expr::StructLit(
                        name,
                        fields,
                        start_span.merge(&self.previous().span),
                    ))
                } else {
                    Ok(Expr::Ident(name, start_span))
                }
            }

            TokenKind::LeftParen => {
                self.advance();

                // 空元组
                if self.check(&TokenKind::RightParen) {
                    self.advance();
                    return Ok(Expr::TupleLit(
                        vec![],
                        start_span.merge(&self.previous().span),
                    ));
                }

                // 尝试解析为闭包
                if self.check(&TokenKind::BitwiseOr)
                    || (matches!(self.current_token_kind(), TokenKind::Ident(_))
                        && self.peek_ahead(1) == Some(&TokenKind::Colon))
                {
                    return self.parse_closure_from_paren(start_span);
                }

                let mut exprs = vec![self.parse_expression()?];

                // 元组或括号表达式
                if self.match_token(&TokenKind::Comma) {
                    while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
                        exprs.push(self.parse_expression()?);
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                    self.consume(TokenKind::RightParen, "Expected ')' after tuple")?;
                    Ok(Expr::TupleLit(
                        exprs,
                        start_span.merge(&self.previous().span),
                    ))
                } else {
                    self.consume(TokenKind::RightParen, "Expected ')' after expression")?;
                    Ok(exprs.into_iter().next().unwrap())
                }
            }

            TokenKind::LeftBracket => {
                self.advance();
                let elements = self.parse_array_elements()?;
                self.consume(TokenKind::RightBracket, "Expected ']' after array elements")?;
                Ok(Expr::ArrayLit(
                    elements,
                    start_span.merge(&self.previous().span),
                ))
            }

            TokenKind::LeftBrace => {
                let block = self.parse_block()?;
                Ok(Expr::Block(block.clone(), block.span))
            }

            TokenKind::If => {
                self.advance();
                let cond = Box::new(self.parse_expression()?);
                let then_block = self.parse_block()?;
                let else_block = if self.match_token(&TokenKind::Else) {
                    Some(self.parse_block()?)
                } else {
                    None
                };
                Ok(Expr::If(
                    cond,
                    then_block,
                    else_block,
                    start_span.merge(&self.previous().span),
                ))
            }

            TokenKind::While => {
                self.advance();
                let cond = Box::new(self.parse_expression()?);
                self.loop_depth += 1;
                let block = self.parse_block()?;
                self.loop_depth -= 1;
                Ok(Expr::While(
                    cond,
                    block,
                    start_span.merge(&self.previous().span),
                ))
            }

            TokenKind::For => {
                self.advance();
                let pattern = self.parse_pattern()?;
                self.consume(TokenKind::In, "Expected 'in' in for loop")?;
                let iter = Box::new(self.parse_expression()?);
                self.loop_depth += 1;
                let block = self.parse_block()?;
                self.loop_depth -= 1;
                Ok(Expr::For(
                    pattern,
                    iter,
                    block,
                    start_span.merge(&self.previous().span),
                ))
            }

            TokenKind::Match => {
                self.advance();
                let expr = Box::new(self.parse_expression()?);
                self.consume(TokenKind::LeftBrace, "Expected '{' after match expression")?;
                let arms = self.parse_match_arms()?;
                self.consume(TokenKind::RightBrace, "Expected '}' after match arms")?;
                Ok(Expr::Match(
                    expr,
                    arms,
                    start_span.merge(&self.previous().span),
                ))
            }

            TokenKind::Break => {
                self.advance();
                if self.loop_depth == 0 {
                    return Err(ParseError::new(
                        "break outside of loop".to_string(),
                        start_span,
                    ));
                }

                let label = if let TokenKind::Ident(name) = self.current_token_kind() {
                    let name = name.clone();
                    self.advance();
                    Some(name)
                } else {
                    None
                };

                let expr = if !self.check(&TokenKind::Semicolon)
                    && !self.check(&TokenKind::RightBrace)
                    && !self.check(&TokenKind::Comma)
                {
                    Some(Box::new(self.parse_expression()?))
                } else {
                    None
                };

                Ok(Expr::Break(
                    label,
                    expr,
                    start_span.merge(&self.previous().span),
                ))
            }

            TokenKind::Continue => {
                self.advance();
                if self.loop_depth == 0 {
                    return Err(ParseError::new(
                        "continue outside of loop".to_string(),
                        start_span,
                    ));
                }

                let label = if let TokenKind::Ident(name) = self.current_token_kind() {
                    let name = name.clone();
                    self.advance();
                    Some(name)
                } else {
                    None
                };

                Ok(Expr::Continue(
                    label,
                    start_span.merge(&self.previous().span),
                ))
            }

            TokenKind::Return => {
                self.advance();
                let expr =
                    if !self.check(&TokenKind::Semicolon) && !self.check(&TokenKind::RightBrace) {
                        Some(Box::new(self.parse_expression()?))
                    } else {
                        None
                    };
                Ok(Expr::Return(expr, start_span.merge(&self.previous().span)))
            }

            TokenKind::BitwiseOr => {
                // 闭包
                self.parse_closure(start_span)
            }

            _ => Err(ParseError::new(
                format!(
                    "Unexpected token in expression: {:?}",
                    self.current_token_kind()
                ),
                self.current_span(),
            )),
        }
    }

    // 辅助方法
    fn parse_args(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = Vec::new();

        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            args.push(self.parse_expression()?);
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        Ok(args)
    }

    fn parse_struct_fields(&mut self) -> Result<Vec<(String, Expr)>, ParseError> {
        let mut fields = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let name = self.expect_ident("Expected field name")?;

            let expr = if self.match_token(&TokenKind::Colon) {
                self.parse_expression()?
            } else {
                // 简写形式：`Point { x, y }` 等价于 `Point { x: x, y: y }`
                Expr::Ident(name.clone(), self.previous().span)
            };

            fields.push((name, expr));

            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        Ok(fields)
    }

    fn parse_array_elements(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut elements = Vec::new();

        while !self.check(&TokenKind::RightBracket) && !self.is_at_end() {
            elements.push(self.parse_expression()?);

            // 检查是否是重复语法 [expr; count]
            if self.match_token(&TokenKind::Semicolon) {
                if let TokenKind::IntLiteral(count) = self.current_token_kind() {
                    let count = *count as usize;
                    self.advance();

                    // 展开为重复的元素
                    let expr = elements[0].clone();
                    elements.clear();
                    for _ in 0..count {
                        elements.push(expr.clone());
                    }
                    break;
                } else {
                    return Err(ParseError::new(
                        "Expected integer after ';' in array literal".to_string(),
                        self.current_span(),
                    ));
                }
            }

            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        Ok(elements)
    }

    fn parse_match_arms(&mut self) -> Result<Vec<MatchArm>, ParseError> {
        let mut arms = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            let start = self.current_span();
            let pattern = self.parse_pattern()?;

            let guard = if self.match_token(&TokenKind::If) {
                Some(self.parse_expression()?)
            } else {
                None
            };

            self.consume(TokenKind::FatArrow, "Expected '=>' in match arm")?;

            let body = self.parse_expression()?;

            arms.push(MatchArm {
                pattern,
                guard,
                body,
                span: start.merge(&self.previous().span),
            });

            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        Ok(arms)
    }

    fn parse_closure(&mut self, start_span: Span) -> Result<Expr, ParseError> {
        self.consume(TokenKind::BitwiseOr, "Expected '|' to start closure")?;

        let mut params = Vec::new();
        while !self.check(&TokenKind::BitwiseOr) && !self.is_at_end() {
            params.push(self.parse_closure_param()?);
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        self.consume(
            TokenKind::BitwiseOr,
            "Expected '|' after closure parameters",
        )?;

        let return_type = if self.match_token(&TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = Box::new(self.parse_expression()?);

        Ok(Expr::Closure(
            params,
            return_type,
            body,
            start_span.merge(&self.previous().span),
        ))
    }

    // In parse_parameter method, make type optional for closures
    fn parse_closure_param(&mut self) -> Result<Parameter, ParseError> {
        let start_span = self.current_span();
        let pattern = self.parse_pattern()?;

        let ty = if self.match_token(&TokenKind::Colon) {
            self.parse_type()?
        } else {
            // For closures without explicit types, use Type::Infer
            Type::Infer
        };

        Ok(Parameter {
            pattern,
            ty,
            span: start_span.merge(&self.previous().span),
        })
    }

    fn parse_closure_from_paren(&mut self, start_span: Span) -> Result<Expr, ParseError> {
        // 已经消费了 '('
        let mut params = Vec::new();

        while !self.check(&TokenKind::RightParen) && !self.is_at_end() {
            params.push(self.parse_parameter()?);
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        self.consume(
            TokenKind::RightParen,
            "Expected ')' after closure parameters",
        )?;

        let return_type = if self.match_token(&TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = Box::new(self.parse_expression()?);

        Ok(Expr::Closure(
            params,
            return_type,
            body,
            start_span.merge(&self.previous().span),
        ))
    }

    // Token 操作辅助方法
    fn current_token(&self) -> &Token {
        &self.tokens[self.current.min(self.tokens.len() - 1)]
    }

    fn current_token_kind(&self) -> &TokenKind {
        &self.current_token().kind
    }

    fn current_span(&self) -> Span {
        self.current_token().span
    }

    fn previous(&self) -> &Token {
        &self.tokens[(self.current - 1).min(self.tokens.len() - 1)]
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current_token_kind(), TokenKind::Eof)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(self.current_token_kind()) == std::mem::discriminant(kind)
        }
    }

    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn match_binary_op(&mut self, kinds: &[TokenKind]) -> Option<BinOp> {
        for kind in kinds {
            if self.check(kind) {
                let op = match kind {
                    TokenKind::Plus => BinOp::Add,
                    TokenKind::Minus => BinOp::Sub,
                    TokenKind::Star => BinOp::Mul,
                    TokenKind::Slash => BinOp::Div,
                    TokenKind::Percent => BinOp::Mod,
                    TokenKind::Equal => BinOp::Equal,
                    TokenKind::NotEqual => BinOp::NotEqual,
                    TokenKind::Less => BinOp::Less,
                    TokenKind::Greater => BinOp::Greater,
                    TokenKind::LessEqual => BinOp::LessEqual,
                    TokenKind::GreaterEqual => BinOp::GreaterEqual,
                    TokenKind::LogicalAnd => BinOp::LogicalAnd,
                    TokenKind::LogicalOr => BinOp::LogicalOr,
                    TokenKind::BitwiseAnd => BinOp::BitwiseAnd,
                    TokenKind::BitwiseOr => BinOp::BitwiseOr,
                    TokenKind::BitwiseXor => BinOp::BitwiseXor,
                    TokenKind::LeftShift => BinOp::LeftShift,
                    TokenKind::RightShift => BinOp::RightShift,
                    _ => continue,
                };
                self.advance();
                return Some(op);
            }
        }
        None
    }

    fn peek_ahead(&self, n: usize) -> Option<&TokenKind> {
        if self.current + n < self.tokens.len() {
            Some(&self.tokens[self.current + n].kind)
        } else {
            None
        }
    }

    fn consume(&mut self, kind: TokenKind, message: &str) -> Result<&Token, ParseError> {
        if self.check(&kind) {
            Ok(self.advance())
        } else {
            Err(ParseError::new(
                format!("{}, found {:?}", message, self.current_token_kind()),
                self.current_span(),
            ))
        }
    }

    fn expect_ident(&mut self, message: &str) -> Result<String, ParseError> {
        if let TokenKind::Ident(name) = self.current_token_kind() {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(ParseError::new(
                format!("{}, found {:?}", message, self.current_token_kind()),
                self.current_span(),
            ))
        }
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;
        self.advance();

        while !self.is_at_end() {
            if matches!(self.previous().kind, TokenKind::Semicolon) {
                return;
            }

            match self.current_token_kind() {
                TokenKind::Fn
                | TokenKind::Struct
                | TokenKind::Enum
                | TokenKind::Let
                | TokenKind::If
                | TokenKind::While
                | TokenKind::For
                | TokenKind::Return
                | TokenKind::Match => return,
                _ => {}
            }

            self.advance();
        }
    }
}

// 为 Expr 实现 span 方法
impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal(_, span) => *span,
            Expr::Ident(_, span) => *span,
            Expr::Binary(_, _, _, span) => *span,
            Expr::Unary(_, _, span) => *span,
            Expr::Call(_, _, span) => *span,
            Expr::MethodCall(_, _, _, span) => *span,
            Expr::FieldAccess(_, _, span) => *span,
            Expr::IndexAccess(_, _, span) => *span,
            Expr::StructLit(_, _, span) => *span,
            Expr::ArrayLit(_, span) => *span,
            Expr::TupleLit(_, span) => *span,
            Expr::Range(_, _, _, span) => *span,
            Expr::Assign(_, _, span) => *span,
            Expr::CompoundAssign(_, _, _, span) => *span,
            Expr::Block(_, span) => *span,
            Expr::If(_, _, _, span) => *span,
            Expr::Match(_, _, span) => *span,
            Expr::While(_, _, span) => *span,
            Expr::For(_, _, _, span) => *span,
            Expr::Break(_, _, span) => *span,
            Expr::Continue(_, span) => *span,
            Expr::Return(_, span) => *span,
            Expr::Closure(_, _, _, span) => *span,
            Expr::Cast(_, _, span) => *span,
            Expr::Ref(_, _, span) => *span,
            Expr::Deref(_, span) => *span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_expr(input: &str) -> Result<Expr, Vec<ParseError>> {
        let lexer = Lexer::new(input);
        let tokens = lexer.tokenize().map_err(|e| {
            vec![ParseError::new(
                format!("Lexer error: {:?}", e),
                Span::new(0, 0, 1, 1),
            )]
        })?;
        let mut parser = Parser::new(tokens);
        parser.parse_expression().map_err(|e| vec![e])
    }

    fn parse_program(input: &str) -> Result<Program, Vec<ParseError>> {
        let lexer = Lexer::new(input);
        let tokens = lexer.tokenize().map_err(|e| {
            vec![ParseError::new(
                format!("Lexer error: {:?}", e),
                Span::new(0, 0, 1, 1),
            )]
        })?;
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn test_parse_literals() {
        assert!(parse_expr("42").is_ok());
        assert!(parse_expr("true").is_ok());
        assert!(parse_expr("false").is_ok());
        assert!(parse_expr("'a'").is_ok());
        assert!(parse_expr("\"hello\"").is_ok());
    }

    #[test]
    fn test_parse_binary_ops() {
        assert!(parse_expr("1 + 2").is_ok());
        assert!(parse_expr("3 * 4 + 5").is_ok());
        assert!(parse_expr("6 + 7 * 8").is_ok());
        assert!(parse_expr("a && b || c").is_ok());
        assert!(parse_expr("x < y && y <= z").is_ok());
    }

    #[test]
    fn test_parse_function() {
        let input = r#"
        fn add(x: i32, y: i32) -> i32 {
            return x + y;
        }
    "#;
        assert!(parse_program(input).is_ok());
    }

    #[test]
    fn test_parse_struct() {
        let input = r#"
        struct Point {
            x: i32,
            y: i32,
        }
    "#;
        assert!(parse_program(input).is_ok());
    }

    #[test]
    fn test_parse_enum() {
        let input = r#"
        enum Option<T> {
            Some(T),
            None,
        }
    "#;
        assert!(parse_program(input).is_ok());
    }

    #[test]
    fn test_parse_if_else() {
        let input = r#"
        fn test() {
            if x > 0 {
                print("positive");
            } else if x < 0 {
                print("negative");
            } else {
                print("zero");
            }
        }
    "#;
        assert!(parse_program(input).is_ok());
    }

    #[test]
    fn test_parse_while_loop() {
        let input = r#"
        fn test() {
            let mut i = 0;
            while i < 10 {
                print(i);
                i = i + 1;
            }
        }
    "#;
        assert!(parse_program(input).is_ok());
    }

    #[test]
    fn test_parse_for_loop() {
        let input = r#"
        fn test() {
            for i in 0..10 {
                print(i);
            }

            let arr = [1, 2, 3, 4, 5];
            for item in arr {
                print(item);
            }
        }
    "#;
        assert!(parse_program(input).is_ok());
    }

    #[test]
    fn test_parse_match() {
        let input = r#"
        fn test(x: Option<i32>) {
            match x {
                Some(n) if n > 0 => print("positive"),
                Some(0) => print("zero"),
                Some(n) => print("negative"),
                None => print("none"),
            }
        }
    "#;
        assert!(parse_program(input).is_ok());
    }

    #[test]
    fn test_parse_closure() {
        let input = r#"
        fn test() {
            let add = |x: i32, y: i32| -> i32 { x + y };
            let mul = |x, y| x * y;
            let print_it = |x| print(x);
        }
    "#;
        assert!(parse_program(input).is_ok());
    }

    #[test]
    fn test_parse_method_call() {
        let input = r#"
        fn test() {
            let s = "hello";
            let len = s.len();
            let result = vec.push(42).pop().unwrap();
        }
    "#;
        assert!(parse_program(input).is_ok());
    }

    #[test]
    fn test_parse_array_and_index() {
        let input = r#"
        fn test() {
            let arr = [1, 2, 3, 4, 5];
            let first = arr[0];
            let matrix = [[1, 2], [3, 4]];
            let elem = matrix[0][1];
            let repeated = [0; 100];
        }
    "#;
        assert!(parse_program(input).is_ok());
    }

    #[test]
    fn test_parse_struct_literal() {
        let input = r#"
        fn test() {
            let p = Point { x: 10, y: 20 };
            let p2 = Point { x, y };  // field shorthand
        }
    "#;
        assert!(parse_program(input).is_ok());
    }

    #[test]
    fn test_parse_references() {
        let input = r#"
        fn test() {
            let x = 42;
            let r = &x;
            let mr = &mut x;
            let value = *r;
        }
    "#;
        assert!(parse_program(input).is_ok());
    }

    #[test]
    fn test_parse_type_cast() {
        let input = r#"
        fn test() {
            let x = 42;
            let y = x as f64;
            let z = (x + 1) as u8;
        }
    "#;
        assert!(parse_program(input).is_ok());
    }

    #[test]
    fn test_parse_complex_types() {
        let input = r#"
        fn test(
            a: [i32; 10],
            b: [i32],
            c: *const i32,
            d: &mut Vec<String>,
            e: (i32, bool, String),
            f: fn(i32, i32) -> i32,
            g: Option<Box<Node>>,
        ) -> Result<(), Error> {
            return Ok(());
        }
    "#;
        assert!(parse_program(input).is_ok());
    }

    #[test]
    fn test_parse_generics() {
        let input = r#"
        fn identity<T>(x: T) -> T {
            return x;
        }

        struct Vec<T> {
            data: *mut T,
            len: usize,
            cap: usize,
        }

        fn map<T, U, F>(vec: Vec<T>, f: F) -> Vec<U> {
            // ...
        }
    "#;
        assert!(parse_program(input).is_ok());
    }

    #[test]
    fn test_error_recovery() {
        let input = r#"
        fn test() {
            let x = ;  // error: missing expression
            let y = 42;  // should still parse this
        }
    "#;

        let result = parse_program(input);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_break_continue_validation() {
        // Should succeed - break/continue inside loop
        let valid = r#"
        fn test() {
            while true {
                if x > 10 {
                    break;
                }
                continue;
            }
        }
    "#;
        assert!(parse_program(valid).is_ok());

        // Should fail - break outside loop
        let invalid = r#"
        fn test() {
            break;
        }
    "#;
        assert!(parse_program(invalid).is_err());
    }
}
