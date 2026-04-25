use crate::ast::span::Span;

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: Level,
    pub code: Option<String>,
    pub message: String,
    pub span: Option<Span>,
    pub hints: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Error,
    Warning,
    Note,
}

impl Diagnostic {
    pub fn error(msg: impl Into<String>) -> Self {
        Diagnostic { level: Level::Error, code: None, message: msg.into(), span: None, hints: Vec::new() }
    }

    pub fn error_at(msg: impl Into<String>, span: Span) -> Self {
        Diagnostic { level: Level::Error, code: None, message: msg.into(), span: Some(span), hints: Vec::new() }
    }

    pub fn warning(msg: impl Into<String>) -> Self {
        Diagnostic { level: Level::Warning, code: None, message: msg.into(), span: None, hints: Vec::new() }
    }

    pub fn warning_at(msg: impl Into<String>, span: Span) -> Self {
        Diagnostic { level: Level::Warning, code: None, message: msg.into(), span: Some(span), hints: Vec::new() }
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hints.push(hint.into());
        self
    }

    pub fn is_error(&self) -> bool {
        self.level == Level::Error
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostics {
    items: Vec<Diagnostic>,
    has_errors: bool,
}

impl Diagnostics {
    pub fn new() -> Self {
        Diagnostics { items: Vec::new(), has_errors: false }
    }

    pub fn emit(&mut self, diag: Diagnostic) {
        if diag.is_error() {
            self.has_errors = true;
        }
        self.items.push(diag);
    }

    pub fn has_errors(&self) -> bool {
        self.has_errors
    }

    pub fn items(&self) -> &[Diagnostic] {
        &self.items
    }

    pub fn into_items(self) -> Vec<Diagnostic> {
        self.items
    }
}

impl Default for Diagnostics {
    fn default() -> Self {
        Self::new()
    }
}
