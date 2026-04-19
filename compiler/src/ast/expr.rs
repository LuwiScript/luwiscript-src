// luwi-script/compiler/src/ast/expr.rs

use crate::ast::literal::Literal;
use crate::ast::r#type::Type;
use crate::ast::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // Literais
    Literal(Literal, Span),

    // Identificador
    Ident(String, Span),

    // Binário: a + b, a == b, etc.
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },

    // Unário: -a, !a, etc.
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },

    // Chamada de função: f(a, b)
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },

    // Bloco de expressões (usado como corpo de função, if, etc.)
    Block {
        stmts: Vec<Stmt>,
        final_expr: Option<Box<Expr>>, // última expressão opcional
        span: Span,
    },

    // If/else como expressão
    If {
        cond: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
        span: Span,
    },

    // Lambda/fechamento
    Lambda {
        params: Vec<Param>,
        ret_type: Option<Type>,
        body: Box<Expr>,
        span: Span,
    },

    // Acesso a campo/indice
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },

    // Acesso a campo de struct
    Member {
        target: Box<Expr>,
        field: String,
        span: Span,
    },

    // Placeholder que você pode expandir
    Placeholder(Span),
}

// Stmt está em outro módulo, mas expr.rs pode usar uma definição mínima enquanto isso.
#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    Expr(Expr),
    Let { ident: String, init: Expr, span: Span },
    Semi(Expr),
}

// Parâmetro de função
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub ident: String,
    pub ty: Type,
    pub span: Span,
}

// Operadores binários
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,    // +
    Sub,    // -
    Mul,    // *
    Div,    // /
    Rem,    // %

    Eq,     // ==
    Neq,    // !=
    Lt,     // <
    Le,     // <=
    Gt,     // >
    Ge,     // >=

    And,    // &&
    Or,     // ||

    Assign, // =
}

// Operadores unários
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,   // -
    Not,   // !
    Ref,   // &
    Deref, // *
}
