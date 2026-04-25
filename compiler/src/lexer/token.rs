use crate::ast::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    // Literals
    Int,
    Float,
    String,
    Bool,
    Null,

    // Identifiers and keywords
    Ident,
    // Keywords
    Fn,
    Let,
    Const,
    If,
    Else,
    While,
    For,
    In,
    Return,
    Break,
    Continue,
    Struct,
    Enum,
    Impl,
    Import,
    As,
    Match,
    True,
    False,
    Mut,
    Ref,
    Spawn,
    Await,
    Async,
    Pub,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Eq,
    EqEq,
    Neq,
    Lt,
    Le,
    Gt,
    Ge,
    AndAnd,
    OrOr,
    Not,
    Amp,
    Pipe,
    Arrow,
    FatArrow,

    // Delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    // Punctuation
    Comma,
    Dot,
    Colon,
    ColonColon,
    Semicolon,
    Underscore,
    DotDot,
    Question,

    // Special
    Eof,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, span: Span) -> Self {
        Token { kind, lexeme, span }
    }

    pub fn eof(span: Span) -> Self {
        Token { kind: TokenKind::Eof, lexeme: String::new(), span }
    }
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TokenKind::Int => "integer",
            TokenKind::Float => "float",
            TokenKind::String => "string",
            TokenKind::Bool => "bool",
            TokenKind::Null => "null",
            TokenKind::Ident => "identifier",
            TokenKind::Fn => "fn",
            TokenKind::Let => "let",
            TokenKind::Const => "const",
            TokenKind::If => "if",
            TokenKind::Else => "else",
            TokenKind::While => "while",
            TokenKind::For => "for",
            TokenKind::In => "in",
            TokenKind::Return => "return",
            TokenKind::Break => "break",
            TokenKind::Continue => "continue",
            TokenKind::Struct => "struct",
            TokenKind::Enum => "enum",
            TokenKind::Impl => "impl",
            TokenKind::Import => "import",
            TokenKind::As => "as",
            TokenKind::Match => "match",
            TokenKind::True => "true",
            TokenKind::False => "false",
            TokenKind::Mut => "mut",
            TokenKind::Ref => "ref",
            TokenKind::Spawn => "spawn",
            TokenKind::Await => "await",
            TokenKind::Async => "async",
            TokenKind::Pub => "pub",
            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Star => "*",
            TokenKind::Slash => "/",
            TokenKind::Percent => "%",
            TokenKind::Eq => "=",
            TokenKind::EqEq => "==",
            TokenKind::Neq => "!=",
            TokenKind::Lt => "<",
            TokenKind::Le => "<=",
            TokenKind::Gt => ">",
            TokenKind::Ge => ">=",
            TokenKind::AndAnd => "&&",
            TokenKind::OrOr => "||",
            TokenKind::Not => "!",
            TokenKind::Amp => "&",
            TokenKind::Pipe => "|",
            TokenKind::Arrow => "->",
            TokenKind::FatArrow => "=>",
            TokenKind::LParen => "(",
            TokenKind::RParen => ")",
            TokenKind::LBrace => "{",
            TokenKind::RBrace => "}",
            TokenKind::LBracket => "[",
            TokenKind::RBracket => "]",
            TokenKind::Comma => ",",
            TokenKind::Dot => ".",
            TokenKind::Colon => ":",
            TokenKind::ColonColon => "::",
            TokenKind::Semicolon => ";",
            TokenKind::Underscore => "_",
            TokenKind::DotDot => "..",
            TokenKind::Question => "?",
            TokenKind::Eof => "EOF",
            TokenKind::Error => "error",
        };
        write!(f, "{s}")
    }
}

pub fn keyword_or_ident(lexeme: &str) -> TokenKind {
    match lexeme {
        "fn" => TokenKind::Fn,
        "let" => TokenKind::Let,
        "const" => TokenKind::Const,
        "if" => TokenKind::If,
        "else" => TokenKind::Else,
        "while" => TokenKind::While,
        "for" => TokenKind::For,
        "in" => TokenKind::In,
        "return" => TokenKind::Return,
        "break" => TokenKind::Break,
        "continue" => TokenKind::Continue,
        "struct" => TokenKind::Struct,
        "enum" => TokenKind::Enum,
        "impl" => TokenKind::Impl,
        "import" => TokenKind::Import,
        "as" => TokenKind::As,
        "match" => TokenKind::Match,
        "true" => TokenKind::True,
        "false" => TokenKind::False,
        "mut" => TokenKind::Mut,
        "ref" => TokenKind::Ref,
        "spawn" => TokenKind::Spawn,
        "await" => TokenKind::Await,
        "async" => TokenKind::Async,
        "pub" => TokenKind::Pub,
        "null" => TokenKind::Null,
        _ => TokenKind::Ident,
    }
}
