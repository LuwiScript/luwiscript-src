use crate::ast::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int(Span),
    Float(Span),
    Bool(Span),
    String(Span),
    Null(Span),
    Void(Span),
    Named {
        name: String,
        span: Span,
    },
    Generic {
        name: String,
        args: Vec<Type>,
        span: Span,
    },
    Tuple(Vec<Type>, Span),
    Array {
        elem: Box<Type>,
        span: Span,
    },
    Func {
        params: Vec<Type>,
        ret: Box<Type>,
        span: Span,
    },
    Optional {
        inner: Box<Type>,
        span: Span,
    },
    Reference {
        inner: Box<Type>,
        mutable: bool,
        span: Span,
    },
    Any(Span),
}

impl Type {
    pub fn span(&self) -> &Span {
        match self {
            Type::Int(s) => s,
            Type::Float(s) => s,
            Type::Bool(s) => s,
            Type::String(s) => s,
            Type::Null(s) => s,
            Type::Void(s) => s,
            Type::Named { span, .. } => span,
            Type::Generic { span, .. } => span,
            Type::Tuple(_, s) => s,
            Type::Array { span, .. } => span,
            Type::Func { span, .. } => span,
            Type::Optional { span, .. } => span,
            Type::Reference { span, .. } => span,
            Type::Any(s) => s,
        }
    }

    pub fn name(&self) -> String {
        match self {
            Type::Int(_) => "int".into(),
            Type::Float(_) => "float".into(),
            Type::Bool(_) => "bool".into(),
            Type::String(_) => "string".into(),
            Type::Null(_) => "null".into(),
            Type::Void(_) => "void".into(),
            Type::Named { name, .. } => name.clone(),
            Type::Generic { name, args, .. } => {
                let params = args.iter().map(|a| a.name()).collect::<Vec<_>>().join(", ");
                format!("{name}<{params}>")
            }
            Type::Tuple(elems, _) => {
                let inner = elems.iter().map(|e| e.name()).collect::<Vec<_>>().join(", ");
                format!("({inner})")
            }
            Type::Array { elem, .. } => format!("[{}]", elem.name()),
            Type::Func { params, ret, .. } => {
                let ps = params.iter().map(|p| p.name()).collect::<Vec<_>>().join(", ");
                format!("fn({ps}) -> {}", ret.name())
            }
            Type::Optional { inner, .. } => format!("{}?", inner.name()),
            Type::Reference { inner, mutable, .. } => {
                if *mutable {
                    format!("&mut {}", inner.name())
                } else {
                    format!("&{}", inner.name())
                }
            }
            Type::Any(_) => "any".into(),
        }
    }
}
