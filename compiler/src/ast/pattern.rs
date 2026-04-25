use crate::ast::literal::Literal;
use crate::ast::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Ident(String, Span),
    Wildcard(Span),
    Literal(Literal, Span),
    Tuple(Vec<Pattern>, Span),
    Struct {
        name: String,
        fields: Vec<(String, Pattern)>,
        span: Span,
    },
    Rest(Span),
}
