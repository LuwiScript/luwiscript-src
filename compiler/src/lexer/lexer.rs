use crate::ast::span::Span;
use crate::lexer::token::{Token, TokenKind, keyword_or_ident};

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    file_id: usize,
}

impl Lexer {
    pub fn new(source: &str, file_id: usize) -> Self {
        Lexer {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            file_id,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            let is_eof = tok.kind == TokenKind::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        tokens
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    fn peek_ahead(&self, n: usize) -> Option<char> {
        self.source.get(self.pos + n).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.source.get(self.pos).copied();
        if let Some(c) = ch {
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        ch
    }

    fn make_span(&self, start: usize, start_line: usize, start_col: usize) -> Span {
        Span::new(start, self.pos, start_line, start_col)
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek() {
                Some(' ') | Some('\t') | Some('\r') | Some('\n') => {
                    self.advance();
                }
                Some('/') => {
                    if self.peek_ahead(1) == Some('/') {
                        while let Some(c) = self.peek() {
                            if c == '\n' { break; }
                            self.advance();
                        }
                    } else if self.peek_ahead(1) == Some('*') {
                        self.advance();
                        self.advance();
                        let mut depth = 1u32;
                        while depth > 0 {
                            match self.peek() {
                                None => break,
                                Some('/') if self.peek_ahead(1) == Some('*') => {
                                    self.advance();
                                    self.advance();
                                    depth += 1;
                                }
                                Some('*') if self.peek_ahead(1) == Some('/') => {
                                    self.advance();
                                    self.advance();
                                    depth -= 1;
                                }
                                Some(_) => { self.advance(); }
                            }
                        }
                    } else {
                        break;
                    }
                }
                Some('#') => {
                    if self.peek_ahead(1) == Some('!') && self.pos == 0 {
                        while let Some(c) = self.peek() {
                            if c == '\n' { break; }
                            self.advance();
                        }
                    } else if self.peek_ahead(1) == Some('[') {
                        while let Some(c) = self.peek() {
                            if c == '\n' { break; }
                            self.advance();
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        let start = self.pos;
        let start_line = self.line;
        let start_col = self.col;

        let ch = match self.peek() {
            Some(c) => c,
            None => return Token::eof(self.make_span(start, start_line, start_col)),
        };

        match ch {
            '0'..='9' => self.lex_number(start, start_line, start_col),
            '"' | '\'' => self.lex_string(start, start_line, start_col, ch),
            c if c.is_alphabetic() || c == '_' => self.lex_ident(start, start_line, start_col),
            _ => self.lex_symbol(start, start_line, start_col),
        }
    }

    fn lex_number(&mut self, start: usize, start_line: usize, start_col: usize) -> Token {
        let mut is_float = false;
        let mut lexeme = String::new();

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                lexeme.push(self.advance().unwrap());
            } else if c == '.' && self.peek_ahead(1).map_or(false, |n| n.is_ascii_digit()) {
                is_float = true;
                lexeme.push(self.advance().unwrap());
                lexeme.push(self.advance().unwrap());
            } else if c == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let span = self.make_span(start, start_line, start_col);
        let kind = if is_float { TokenKind::Float } else { TokenKind::Int };
        Token::new(kind, lexeme, span)
    }

    fn lex_string(&mut self, start: usize, start_line: usize, start_col: usize, quote: char) -> Token {
        let mut lexeme = String::new();
        lexeme.push(self.advance().unwrap());

        let mut content = String::new();
        loop {
            match self.peek() {
                None => {
                    let span = self.make_span(start, start_line, start_col);
                    return Token::new(TokenKind::Error, lexeme, span);
                }
                Some('\\') => {
                    lexeme.push(self.advance().unwrap());
                    if let Some(c) = self.advance() {
                        lexeme.push(c);
                        match c {
                            'n' => content.push('\n'),
                            't' => content.push('\t'),
                            'r' => content.push('\r'),
                            '\\' => content.push('\\'),
                            '\'' => content.push('\''),
                            '"' => content.push('"'),
                            '0' => content.push('\0'),
                            _ => {
                                content.push('\\');
                                content.push(c);
                            }
                        }
                    }
                }
                Some(c) if c == quote => {
                    lexeme.push(self.advance().unwrap());
                    break;
                }
                Some(c) => {
                    lexeme.push(c);
                    content.push(c);
                    self.advance();
                }
            }
        }

        let span = self.make_span(start, start_line, start_col);
        Token::new(TokenKind::String, content, span)
    }

    fn lex_ident(&mut self, start: usize, start_line: usize, start_col: usize) -> Token {
        let mut lexeme = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                lexeme.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        let span = self.make_span(start, start_line, start_col);
        let kind = keyword_or_ident(&lexeme);
        Token::new(kind, lexeme, span)
    }

    fn lex_symbol(&mut self, start: usize, start_line: usize, start_col: usize) -> Token {
        let ch = self.advance().unwrap();
        let span = self.make_span(start, start_line, start_col);

        let kind = match ch {
            '+' => TokenKind::Plus,
            '-' => {
                if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::Arrow
                } else {
                    TokenKind::Minus
                }
            }
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::EqEq
                } else if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::FatArrow
                } else {
                    TokenKind::Eq
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::Neq
                } else {
                    TokenKind::Not
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::Le
                } else {
                    TokenKind::Lt
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::Ge
                } else {
                    TokenKind::Gt
                }
            }
            '&' => {
                if self.peek() == Some('&') {
                    self.advance();
                    TokenKind::AndAnd
                } else {
                    TokenKind::Amp
                }
            }
            '|' => {
                if self.peek() == Some('|') {
                    self.advance();
                    TokenKind::OrOr
                } else {
                    TokenKind::Pipe
                }
            }
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            ',' => TokenKind::Comma,
            '.' => {
                if self.peek() == Some('.') {
                    self.advance();
                    TokenKind::DotDot
                } else {
                    TokenKind::Dot
                }
            }
            ':' => {
                if self.peek() == Some(':') {
                    self.advance();
                    TokenKind::ColonColon
                } else {
                    TokenKind::Colon
                }
            }
            ';' => TokenKind::Semicolon,
            '_' => TokenKind::Underscore,
            '?' => TokenKind::Question,
            _ => TokenKind::Error,
        };

        let lexeme = self.source[start..self.pos].iter().collect();
        Token::new(kind, lexeme, span)
    }
}
