use crate::ast::expr::{Expr, Param};
use crate::ast::pattern::Pattern;
use crate::ast::r#type::Type;
use crate::ast::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    Expr(Expr),
    Let {
        pattern: Pattern,
        ty: Option<Type>,
        init: Option<Expr>,
    },
    Const {
        ident: String,
        ty: Option<Type>,
        init: Expr,
    },
    Func {
        name: String,
        params: Vec<Param>,
        ret_type: Option<Type>,
        body: Expr,
        is_async: bool,
    },
    Return {
        value: Option<Expr>,
    },
    If {
        cond: Expr,
        then_branch: Expr,
        else_branch: Option<Expr>,
    },
    While {
        cond: Expr,
        body: Expr,
    },
    For {
        ident: String,
        iter: Expr,
        body: Expr,
    },
    Break,
    Continue,
    Assign {
        target: Expr,
        value: Expr,
    },
    Import {
        path: String,
        alias: Option<String>,
    },
    Struct {
        name: String,
        fields: Vec<StructField>,
    },
    Enum {
        name: String,
        variants: Vec<EnumVariant>,
    },
    Impl {
        target: Type,
        methods: Vec<Stmt>,
    },
    Semi(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub ty: Type,
    pub default: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Vec<Type>,
}
