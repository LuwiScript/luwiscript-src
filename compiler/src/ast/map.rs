use std::collections::HashMap;

use crate::ast::span::Span;

pub type SymbolId = usize;

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub id: SymbolId,
    pub name: String,
    pub ty: crate::ast::r#type::Type,
    pub span: Span,
    pub kind: SymbolKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Var,
    Const,
    Func { params: Vec<crate::ast::expr::Param> },
    Struct { fields: Vec<String> },
    Enum { variants: Vec<String> },
    Module,
}

#[derive(Debug, Clone)]
pub struct SymbolMap {
    symbols: HashMap<SymbolId, Symbol>,
    by_name: HashMap<String, SymbolId>,
    next_id: SymbolId,
}

impl SymbolMap {
    pub fn new() -> Self {
        SymbolMap {
            symbols: HashMap::new(),
            by_name: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn insert(&mut self, name: String, ty: crate::ast::r#type::Type, span: Span, kind: SymbolKind) -> SymbolId {
        let id = self.next_id;
        self.next_id += 1;
        let sym = Symbol { id, name: name.clone(), ty, span, kind };
        self.symbols.insert(id, sym);
        self.by_name.insert(name, id);
        id
    }

    pub fn get_by_id(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.get(&id)
    }

    pub fn get_by_name(&self, name: &str) -> Option<&Symbol> {
        self.by_name.get(name).and_then(|id| self.symbols.get(id))
    }

    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Symbol> {
        self.symbols.values()
    }
}

impl Default for SymbolMap {
    fn default() -> Self {
        Self::new()
    }
}
