use crate::ast::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub args: Vec<String>,
    pub span: Span,
}

impl Attribute {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Attribute {
            name: name.into(),
            args: Vec::new(),
            span,
        }
    }
}
