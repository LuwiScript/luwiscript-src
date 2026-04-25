use crate::ast::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Null,
}

impl Literal {
    pub fn spanned(self, span: Span) -> crate::ast::expr::Expr {
        crate::ast::expr::Expr::Literal(self, span)
    }

    pub fn ty_name(&self) -> &'static str {
        match self {
            Literal::Int(_) => "int",
            Literal::Float(_) => "float",
            Literal::Bool(_) => "bool",
            Literal::String(_) => "string",
            Literal::Null => "null",
        }
    }
}
