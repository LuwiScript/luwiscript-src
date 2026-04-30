use crate::ast::literal::Literal;
use crate::ast::r#type::Type;
use crate::ast::span::Span;
use crate::ast::stmt::Stmt;
use crate::ast::pattern::Pattern;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal, Span),

    Ident(String, Span),

    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },

    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },

    Await {
        expr: Box<Expr>,
        span: Span,
    },

    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },

    Block {
        stmts: Vec<Stmt>,
        final_expr: Option<Box<Expr>>,
        span: Span,
    },

    If {
        cond: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
        span: Span,
    },

    Lambda {
        params: Vec<Param>,
        ret_type: Option<Type>,
        body: Box<Expr>,
        span: Span,
    },

    Index {
        target: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },

    Member {
        target: Box<Expr>,
        field: String,
        span: Span,
    },

    Match {
        scrutinee: Box<Expr>,
        arms: Vec<(Pattern, Expr)>,
        span: Span,
    },

    Tuple {
        elems: Vec<Expr>,
        span: Span,
    },

    Array {
        elems: Vec<Expr>,
        span: Span,
    },

    StructInit {
        name: String,
        fields: Vec<(String, Expr)>,
        span: Span,
    },

    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        span: Span,
    },

    Placeholder(Span),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub ident: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Neq,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Assign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
    Ref,
    Deref,
}
