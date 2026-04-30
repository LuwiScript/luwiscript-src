use std::collections::HashMap;
use std::fmt;

/// Runtime values stored on the VM stack and in locals.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Array(Vec<Value>),
    Tuple(Vec<Value>),
    /// A struct instance: a map of field names to values.
    /// This is the foundational composite type — everything (objects,
    /// records, modules) can be built on top of `Struct`.
    Struct(HashMap<String, Value>),
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Int(n) => *n != 0,
            Value::Float(n) => *n != 0.0,
            Value::Bool(b) => *b,
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.is_empty(),
            Value::Tuple(t) => !t.is_empty(),
            Value::Struct(_) => true,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Bool(_) => "bool",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Tuple(_) => "tuple",
            Value::Struct(_) => "struct",
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Int(n) => write!(f, "{n}"),
            Value::Float(n) => write!(f, "{n}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Array(elems) => {
                write!(f, "[")?;
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{e}")?;
                }
                write!(f, "]")
            }
            Value::Tuple(elems) => {
                write!(f, "(")?;
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{e}")?;
                }
                write!(f, ")")
            }
            Value::Struct(fields) => {
                write!(f, "{{ ")?;
                let mut first = true;
                for (k, v) in fields {
                    if !first { write!(f, ", ")?; }
                    first = false;
                    write!(f, "{k}: {v}")?;
                }
                write!(f, " }}")
            }
        }
    }
}