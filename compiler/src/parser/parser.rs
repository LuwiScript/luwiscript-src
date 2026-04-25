use crate::ast::expr::{BinaryOp, Expr, Param, UnaryOp};
use crate::ast::literal::Literal;
use crate::ast::pattern::Pattern;
use crate::ast::r#type::Type;
use crate::ast::span::Span;
use crate::ast::stmt::{EnumVariant, Stmt, StmtKind, StructField};
use crate::lexer::token::{Token, TokenKind};

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

pub type ParseResult<T> = Result<T, ParseError>;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut stmts = Vec::new();
        while !self.at_end() {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or_else(|| self.tokens.last().unwrap())
    }

    fn peek_kind(&self) -> TokenKind {
        self.peek().kind
    }

    fn at_end(&self) -> bool {
        self.peek_kind() == TokenKind::Eof
    }

    fn advance(&mut self) -> Token {
        let tok = self.peek().clone();
        if !self.at_end() {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, kind: TokenKind) -> ParseResult<Token> {
        let tok = self.advance();
        if tok.kind == kind {
            Ok(tok)
        } else {
            Err(ParseError {
                message: format!("expected {}, found '{}'", kind, tok.lexeme),
                span: tok.span,
            })
        }
    }

    fn match_kind(&mut self, kind: TokenKind) -> Option<Token> {
        if self.peek_kind() == kind {
            Some(self.advance())
        } else {
            None
        }
    }

    fn span_since(&self, start: usize) -> Span {
        let start_tok = self.tokens.get(start).unwrap_or_else(|| self.tokens.last().unwrap());
        let end_tok = if self.pos > 0 {
            self.tokens.get(self.pos - 1).unwrap_or_else(|| self.tokens.last().unwrap())
        } else {
            start_tok
        };
        start_tok.span.merge(end_tok.span)
    }

    fn is_expr_start(&self) -> bool {
        matches!(
            self.peek_kind(),
            TokenKind::Int | TokenKind::Float | TokenKind::String
            | TokenKind::True | TokenKind::False | TokenKind::Null
            | TokenKind::Ident | TokenKind::LParen | TokenKind::LBrace
            | TokenKind::LBracket
            | TokenKind::Minus | TokenKind::Not | TokenKind::Amp | TokenKind::Star
            | TokenKind::Spawn | TokenKind::Pipe
        )
    }

    fn parse_stmt(&mut self) -> ParseResult<Stmt> {
        match self.peek_kind() {
            TokenKind::Let => self.parse_let(),
            TokenKind::Const => self.parse_const(),
            TokenKind::Fn => self.parse_fn(),
            TokenKind::Struct => self.parse_struct(),
            TokenKind::Enum => self.parse_enum(),
            TokenKind::Impl => self.parse_impl(),
            TokenKind::Import => self.parse_import(),
            TokenKind::Return => self.parse_return(),
            TokenKind::While => {
                let start = self.pos;
                self.advance();
                let cond = self.parse_expr()?;
                let body = self.parse_block_or_expr()?;
                let span = self.span_since(start);
                Ok(Stmt { kind: StmtKind::While { cond, body }, span })
            }
            TokenKind::For => {
                let start = self.pos;
                self.advance();
                let ident_tok = self.expect(TokenKind::Ident)?;
                self.expect(TokenKind::In)?;
                let iter = self.parse_expr()?;
                let body = self.parse_block_or_expr()?;
                let span = self.span_since(start);
                Ok(Stmt {
                    kind: StmtKind::For { ident: ident_tok.lexeme, iter, body },
                    span,
                })
            }
            TokenKind::Break => {
                let tok = self.advance();
                Ok(Stmt { kind: StmtKind::Break, span: tok.span })
            }
            TokenKind::Continue => {
                let tok = self.advance();
                Ok(Stmt { kind: StmtKind::Continue, span: tok.span })
            }
            _ => self.parse_expr_or_assign_stmt(),
        }
    }

    fn parse_let(&mut self) -> ParseResult<Stmt> {
        let start = self.pos;
        self.expect(TokenKind::Let)?;
        let pattern = self.parse_pattern()?;
        let mut ty = None;
        if self.match_kind(TokenKind::Colon).is_some() {
            ty = Some(self.parse_type()?);
        }
        let mut init = None;
        if self.match_kind(TokenKind::Eq).is_some() {
            init = Some(self.parse_expr()?);
        }
        self.match_kind(TokenKind::Semicolon);
        let span = self.span_since(start);
        Ok(Stmt { kind: StmtKind::Let { pattern, ty, init }, span })
    }

    fn parse_const(&mut self) -> ParseResult<Stmt> {
        let start = self.pos;
        self.expect(TokenKind::Const)?;
        let ident_tok = self.expect(TokenKind::Ident)?;
        let mut ty = None;
        if self.match_kind(TokenKind::Colon).is_some() {
            ty = Some(self.parse_type()?);
        }
        self.expect(TokenKind::Eq)?;
        let init = self.parse_expr()?;
        self.match_kind(TokenKind::Semicolon);
        let span = self.span_since(start);
        Ok(Stmt {
            kind: StmtKind::Const { ident: ident_tok.lexeme, ty, init },
            span,
        })
    }

    fn parse_fn(&mut self) -> ParseResult<Stmt> {
        let start = self.pos;
        self.expect(TokenKind::Fn)?;
        let name_tok = self.expect(TokenKind::Ident)?;
        self.expect(TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(TokenKind::RParen)?;
        let mut ret_type = None;
        if self.match_kind(TokenKind::Arrow).is_some() {
            ret_type = Some(self.parse_type()?);
        }
        let body = self.parse_block_or_expr()?;
        let span = self.span_since(start);
        Ok(Stmt {
            kind: StmtKind::Func {
                name: name_tok.lexeme,
                params,
                ret_type,
                body,
            },
            span,
        })
    }

    fn parse_params(&mut self) -> ParseResult<Vec<Param>> {
        let mut params = Vec::new();
        if self.peek_kind() == TokenKind::RParen {
            return Ok(params);
        }
        loop {
            let ident_tok = self.expect(TokenKind::Ident)?;
            self.expect(TokenKind::Colon)?;
            let ty = self.parse_type()?;
            params.push(Param {
                ident: ident_tok.lexeme,
                ty,
                span: ident_tok.span,
            });
            if self.match_kind(TokenKind::Comma).is_none() {
                break;
            }
        }
        Ok(params)
    }

    fn parse_struct(&mut self) -> ParseResult<Stmt> {
        let start = self.pos;
        self.expect(TokenKind::Struct)?;
        let name_tok = self.expect(TokenKind::Ident)?;
        self.expect(TokenKind::LBrace)?;
        let mut fields = Vec::new();
        while self.peek_kind() != TokenKind::RBrace {
            let f_name = self.expect(TokenKind::Ident)?;
            self.expect(TokenKind::Colon)?;
            let f_ty = self.parse_type()?;
            let default = if self.match_kind(TokenKind::Eq).is_some() {
                Some(self.parse_expr()?)
            } else {
                None
            };
            fields.push(StructField { name: f_name.lexeme, ty: f_ty, default });
            self.match_kind(TokenKind::Comma);
        }
        self.expect(TokenKind::RBrace)?;
        let span = self.span_since(start);
        Ok(Stmt {
            kind: StmtKind::Struct { name: name_tok.lexeme, fields },
            span,
        })
    }

    fn parse_enum(&mut self) -> ParseResult<Stmt> {
        let start = self.pos;
        self.expect(TokenKind::Enum)?;
        let name_tok = self.expect(TokenKind::Ident)?;
        self.expect(TokenKind::LBrace)?;
        let mut variants = Vec::new();
        while self.peek_kind() != TokenKind::RBrace {
            let v_name = self.expect(TokenKind::Ident)?;
            let fields = if self.match_kind(TokenKind::LParen).is_some() {
                let mut fs = Vec::new();
                while self.peek_kind() != TokenKind::RParen {
                    fs.push(self.parse_type()?);
                    self.match_kind(TokenKind::Comma);
                }
                self.expect(TokenKind::RParen)?;
                fs
            } else {
                Vec::new()
            };
            variants.push(EnumVariant { name: v_name.lexeme, fields });
            self.match_kind(TokenKind::Comma);
        }
        self.expect(TokenKind::RBrace)?;
        let span = self.span_since(start);
        Ok(Stmt {
            kind: StmtKind::Enum { name: name_tok.lexeme, variants },
            span,
        })
    }

    fn parse_impl(&mut self) -> ParseResult<Stmt> {
        let start = self.pos;
        self.expect(TokenKind::Impl)?;
        let target = self.parse_type()?;
        self.expect(TokenKind::LBrace)?;
        let mut methods = Vec::new();
        while self.peek_kind() != TokenKind::RBrace {
            methods.push(self.parse_fn()?);
        }
        self.expect(TokenKind::RBrace)?;
        let span = self.span_since(start);
        Ok(Stmt { kind: StmtKind::Impl { target, methods }, span })
    }

    fn parse_import(&mut self) -> ParseResult<Stmt> {
        let start = self.pos;
        self.expect(TokenKind::Import)?;
        let path_tok = self.expect(TokenKind::Ident)?;
        let mut path = path_tok.lexeme;
        while self.match_kind(TokenKind::ColonColon).is_some() {
            let next = self.expect(TokenKind::Ident)?;
            path = format!("{path}::{}", next.lexeme);
        }
        let alias = if self.match_kind(TokenKind::As).is_some() {
            Some(self.expect(TokenKind::Ident)?.lexeme)
        } else {
            None
        };
        self.match_kind(TokenKind::Semicolon);
        let span = self.span_since(start);
        Ok(Stmt { kind: StmtKind::Import { path, alias }, span })
    }

    fn parse_return(&mut self) -> ParseResult<Stmt> {
        let start = self.pos;
        self.expect(TokenKind::Return)?;
        let value = if !self.at_end()
            && !matches!(self.peek_kind(), TokenKind::Semicolon | TokenKind::RBrace)
        {
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.match_kind(TokenKind::Semicolon);
        let span = self.span_since(start);
        Ok(Stmt { kind: StmtKind::Return { value }, span })
    }

    fn parse_expr_or_assign_stmt(&mut self) -> ParseResult<Stmt> {
        let start = self.pos;
        let expr = self.parse_expr()?;
        if self.match_kind(TokenKind::Eq).is_some() {
            let value = self.parse_expr()?;
            self.match_kind(TokenKind::Semicolon);
            let span = self.span_since(start);
            return Ok(Stmt { kind: StmtKind::Assign { target: expr, value }, span });
        }
        let has_semi = self.match_kind(TokenKind::Semicolon).is_some();
        let span = self.span_since(start);
        if has_semi {
            Ok(Stmt { kind: StmtKind::Semi(expr), span })
        } else {
            Ok(Stmt { kind: StmtKind::Expr(expr), span })
        }
    }

    fn parse_expr(&mut self) -> ParseResult<Expr> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_and()?;
        while self.peek_kind() == TokenKind::OrOr {
            self.advance();
            let right = self.parse_and()?;
            let span = left.span().merge(right.span());
            left = Expr::Binary { op: BinaryOp::Or, left: Box::new(left), right: Box::new(right), span };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_equality()?;
        while self.peek_kind() == TokenKind::AndAnd {
            self.advance();
            let right = self.parse_equality()?;
            let span = left.span().merge(right.span());
            left = Expr::Binary { op: BinaryOp::And, left: Box::new(left), right: Box::new(right), span };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_comparison()?;
        while matches!(self.peek_kind(), TokenKind::EqEq | TokenKind::Neq) {
            let op = match self.advance().kind {
                TokenKind::EqEq => BinaryOp::Eq,
                TokenKind::Neq => BinaryOp::Neq,
                _ => unreachable!(),
            };
            let right = self.parse_comparison()?;
            let span = left.span().merge(right.span());
            left = Expr::Binary { op, left: Box::new(left), right: Box::new(right), span };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_range()?;
        while matches!(self.peek_kind(), TokenKind::Lt | TokenKind::Le | TokenKind::Gt | TokenKind::Ge) {
            let op = match self.advance().kind {
                TokenKind::Lt => BinaryOp::Lt,
                TokenKind::Le => BinaryOp::Le,
                TokenKind::Gt => BinaryOp::Gt,
                TokenKind::Ge => BinaryOp::Ge,
                _ => unreachable!(),
            };
            let right = self.parse_range()?;
            let span = left.span().merge(right.span());
            left = Expr::Binary { op, left: Box::new(left), right: Box::new(right), span };
        }
        Ok(left)
    }

    fn parse_range(&mut self) -> ParseResult<Expr> {
        let left = self.parse_addition()?;
        if self.peek_kind() == TokenKind::DotDot {
            self.advance();
            let right = self.parse_addition()?;
            let span = left.span().merge(right.span());
            return Ok(Expr::Range { start: Box::new(left), end: Box::new(right), span });
        }
        Ok(left)
    }

    fn parse_addition(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_multiplication()?;
        while matches!(self.peek_kind(), TokenKind::Plus | TokenKind::Minus) {
            let op = match self.advance().kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                _ => unreachable!(),
            };
            let right = self.parse_multiplication()?;
            let span = left.span().merge(right.span());
            left = Expr::Binary { op, left: Box::new(left), right: Box::new(right), span };
        }
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> ParseResult<Expr> {
        let mut left = self.parse_unary()?;
        while matches!(self.peek_kind(), TokenKind::Star | TokenKind::Slash | TokenKind::Percent) {
            let op = match self.advance().kind {
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                TokenKind::Percent => BinaryOp::Rem,
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            let span = left.span().merge(right.span());
            left = Expr::Binary { op, left: Box::new(left), right: Box::new(right), span };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> ParseResult<Expr> {
        match self.peek_kind() {
            TokenKind::Minus => {
                let tok = self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary { op: UnaryOp::Neg, expr: Box::new(expr), span: tok.span })
            }
            TokenKind::Not => {
                let tok = self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary { op: UnaryOp::Not, expr: Box::new(expr), span: tok.span })
            }
            TokenKind::Amp => {
                let tok = self.advance();
                let _is_mut = self.match_kind(TokenKind::Mut);
                let expr = self.parse_unary()?;
                Ok(Expr::Unary { op: UnaryOp::Ref, expr: Box::new(expr), span: tok.span })
            }
            TokenKind::Star => {
                let tok = self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary { op: UnaryOp::Deref, expr: Box::new(expr), span: tok.span })
            }
            _ => self.parse_call(),
        }
    }

    fn parse_call(&mut self) -> ParseResult<Expr> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek_kind() {
                TokenKind::LParen => {
                    self.advance();
                    let mut args = Vec::new();
                    if self.peek_kind() != TokenKind::RParen {
                        args.push(self.parse_expr()?);
                        while self.match_kind(TokenKind::Comma).is_some() {
                            args.push(self.parse_expr()?);
                        }
                    }
                    let rparen = self.expect(TokenKind::RParen)?;
                    let span = expr.span().merge(rparen.span);
                    expr = Expr::Call { callee: Box::new(expr), args, span };
                }
                TokenKind::Dot => {
                    self.advance();
                    let field_tok = self.expect(TokenKind::Ident)?;
                    let span = expr.span().merge(field_tok.span);
                    expr = Expr::Member { target: Box::new(expr), field: field_tok.lexeme, span };
                }
                TokenKind::LBracket => {
                    self.advance();
                    let index = self.parse_expr()?;
                    let rbracket = self.expect(TokenKind::RBracket)?;
                    let span = expr.span().merge(rbracket.span);
                    expr = Expr::Index { target: Box::new(expr), index: Box::new(index), span };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> ParseResult<Expr> {
        let tok = self.peek().clone();
        match tok.kind {
            TokenKind::Int => {
                self.advance();
                let n = tok.lexeme.parse::<i64>().map_err(|e| ParseError {
                    message: format!("invalid integer: {e}"),
                    span: tok.span,
                })?;
                Ok(Expr::Literal(Literal::Int(n), tok.span))
            }
            TokenKind::Float => {
                self.advance();
                let n = tok.lexeme.parse::<f64>().map_err(|e| ParseError {
                    message: format!("invalid float: {e}"),
                    span: tok.span,
                })?;
                Ok(Expr::Literal(Literal::Float(n), tok.span))
            }
            TokenKind::String => {
                self.advance();
                Ok(Expr::Literal(Literal::String(tok.lexeme), tok.span))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(true), tok.span))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(false), tok.span))
            }
            TokenKind::Null => {
                self.advance();
                Ok(Expr::Literal(Literal::Null, tok.span))
            }
            TokenKind::Ident => {
                self.advance();
                if self.peek_kind() == TokenKind::LBrace {
                    let ahead = self.tokens.get(self.pos + 1).map(|t| t.kind);
                    let ahead2 = self.tokens.get(self.pos + 2).map(|t| t.kind);
                    if ahead == Some(TokenKind::Ident) && ahead2 == Some(TokenKind::Colon) {
                        let name = tok.lexeme.clone();
                        self.advance();
                        let mut fields = Vec::new();
                        while self.peek_kind() != TokenKind::RBrace {
                            let f_name = self.expect(TokenKind::Ident)?;
                            self.expect(TokenKind::Colon)?;
                            let f_val = self.parse_expr()?;
                            fields.push((f_name.lexeme, f_val));
                            self.match_kind(TokenKind::Comma);
                        }
                        let rbrace = self.expect(TokenKind::RBrace)?;
                        let span = tok.span.merge(rbrace.span);
                        return Ok(Expr::StructInit { name, fields, span });
                    }
                }
                Ok(Expr::Ident(tok.lexeme, tok.span))
            }
            TokenKind::LBrace => self.parse_block(),
            TokenKind::LParen => {
                self.advance();
                if self.peek_kind() == TokenKind::RParen {
                    self.advance();
                    return Ok(Expr::Tuple { elems: Vec::new(), span: tok.span });
                }
                let first = self.parse_expr()?;
                if self.peek_kind() == TokenKind::Comma {
                    let mut elems = vec![first];
                    while self.match_kind(TokenKind::Comma).is_some() {
                        if self.peek_kind() == TokenKind::RParen { break; }
                        elems.push(self.parse_expr()?);
                    }
                    let rparen = self.expect(TokenKind::RParen)?;
                    let span = tok.span.merge(rparen.span);
                    return Ok(Expr::Tuple { elems, span });
                }
                self.expect(TokenKind::RParen)?;
                Ok(first)
            }
            TokenKind::LBracket => {
                self.advance();
                let mut elems = Vec::new();
                while self.peek_kind() != TokenKind::RBracket {
                    elems.push(self.parse_expr()?);
                    self.match_kind(TokenKind::Comma);
                }
                let rbracket = self.expect(TokenKind::RBracket)?;
                let span = tok.span.merge(rbracket.span);
                Ok(Expr::Array { elems, span })
            }
            TokenKind::If => {
                self.advance();
                let cond = self.parse_expr()?;
                let then_branch = self.parse_block_or_expr()?;
                let else_branch = if self.match_kind(TokenKind::Else).is_some() {
                    Some(self.parse_block_or_expr()?)
                } else {
                    None
                };
                let last = self.tokens.get(self.pos.saturating_sub(1)).unwrap_or(&tok);
                let span = tok.span.merge(last.span);
                Ok(Expr::If {
                    cond: Box::new(cond),
                    then_branch: Box::new(then_branch),
                    else_branch: else_branch.map(Box::new),
                    span,
                })
            }
            TokenKind::Match => self.parse_match(),
            TokenKind::Pipe => self.parse_lambda(),
            TokenKind::Spawn => {
                self.advance();
                let inner = self.parse_call()?;
                Ok(inner)
            }
            _ => Err(ParseError {
                message: format!("unexpected token '{}'", tok.lexeme),
                span: tok.span,
            }),
        }
    }

    fn parse_block(&mut self) -> ParseResult<Expr> {
        let lbrace = self.expect(TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        let mut final_expr = None;
        while self.peek_kind() != TokenKind::RBrace && !self.at_end() {
        let is_simple_expr = self.is_expr_start()
            && !matches!(
                self.peek_kind(),
                TokenKind::Let | TokenKind::Const | TokenKind::Fn
                | TokenKind::Struct | TokenKind::Enum | TokenKind::Impl
                | TokenKind::Import | TokenKind::Return
                | TokenKind::While | TokenKind::For
            );
        if is_simple_expr {
                let expr = self.parse_expr()?;
                if self.match_kind(TokenKind::Eq).is_some() {
                    let value = self.parse_expr()?;
                    self.match_kind(TokenKind::Semicolon);
                    let span = expr.span().merge(value.span());
                    stmts.push(Stmt { kind: StmtKind::Assign { target: expr, value }, span });
                } else if self.peek_kind() == TokenKind::RBrace {
                    final_expr = Some(Box::new(expr));
                    break;
                } else if self.match_kind(TokenKind::Semicolon).is_some() {
                    let span = expr.span();
                    stmts.push(Stmt { kind: StmtKind::Semi(expr), span });
                } else {
                    let span = expr.span();
                    stmts.push(Stmt { kind: StmtKind::Expr(expr), span });
                }
                continue;
            }
            stmts.push(self.parse_stmt()?);
        }
        let rbrace = self.expect(TokenKind::RBrace)?;
        let span = lbrace.span.merge(rbrace.span);
        Ok(Expr::Block { stmts, final_expr, span })
    }

    fn parse_block_or_expr(&mut self) -> ParseResult<Expr> {
        if self.peek_kind() == TokenKind::LBrace {
            self.parse_block()
        } else {
            self.parse_expr()
        }
    }

    fn parse_match(&mut self) -> ParseResult<Expr> {
        let start_tok = self.advance();
        let scrutinee = self.parse_expr()?;
        self.expect(TokenKind::LBrace)?;
        let mut arms = Vec::new();
        while self.peek_kind() != TokenKind::RBrace {
            let pattern = self.parse_pattern()?;
            if self.match_kind(TokenKind::FatArrow).is_none() {
                self.expect(TokenKind::Colon)?;
            }
            let body = self.parse_expr()?;
            self.match_kind(TokenKind::Comma);
            arms.push((pattern, body));
        }
        let rbrace = self.expect(TokenKind::RBrace)?;
        let span = start_tok.span.merge(rbrace.span);
        Ok(Expr::Match { scrutinee: Box::new(scrutinee), arms, span })
    }

    fn parse_lambda(&mut self) -> ParseResult<Expr> {
        let start = self.pos;
        self.expect(TokenKind::Pipe)?;
        let mut params = Vec::new();
        while self.peek_kind() != TokenKind::Pipe {
            let ident_tok = self.expect(TokenKind::Ident)?;
            let ty = if self.match_kind(TokenKind::Colon).is_some() {
                self.parse_type()?
            } else {
                Type::Void(Span::zero())
            };
            params.push(Param { ident: ident_tok.lexeme, ty, span: ident_tok.span });
            self.match_kind(TokenKind::Comma);
        }
        self.expect(TokenKind::Pipe)?;
        let ret_type = if self.match_kind(TokenKind::Arrow).is_some() {
            Some(self.parse_type()?)
        } else {
            None
        };
        let body = self.parse_block_or_expr()?;
        let span = self.span_since(start);
        Ok(Expr::Lambda { params, ret_type, body: Box::new(body), span })
    }

    fn parse_pattern(&mut self) -> ParseResult<Pattern> {
        match self.peek_kind() {
            TokenKind::Ident => {
                let tok = self.advance();
                if self.peek_kind() == TokenKind::LBrace {
                    self.advance();
                    let mut fields = Vec::new();
                    while self.peek_kind() != TokenKind::RBrace {
                        let f_name = self.expect(TokenKind::Ident)?;
                        let pat = if self.match_kind(TokenKind::Colon).is_some() {
                            self.parse_pattern()?
                        } else {
                            Pattern::Ident(f_name.lexeme.clone(), f_name.span)
                        };
                        fields.push((f_name.lexeme, pat));
                        self.match_kind(TokenKind::Comma);
                    }
                    let rbrace = self.expect(TokenKind::RBrace)?;
                    let span = tok.span.merge(rbrace.span);
                    Ok(Pattern::Struct { name: tok.lexeme, fields, span })
                } else {
                    Ok(Pattern::Ident(tok.lexeme, tok.span))
                }
            }
            TokenKind::Underscore => {
                let tok = self.advance();
                Ok(Pattern::Wildcard(tok.span))
            }
            TokenKind::DotDot => {
                let tok = self.advance();
                Ok(Pattern::Rest(tok.span))
            }
            TokenKind::LParen => {
                let tok = self.advance();
                let mut pats = Vec::new();
                while self.peek_kind() != TokenKind::RParen {
                    pats.push(self.parse_pattern()?);
                    self.match_kind(TokenKind::Comma);
                }
                let rparen = self.expect(TokenKind::RParen)?;
                let span = tok.span.merge(rparen.span);
                Ok(Pattern::Tuple(pats, span))
            }
            TokenKind::Int | TokenKind::Float | TokenKind::String
            | TokenKind::True | TokenKind::False | TokenKind::Null => {
                let tok = self.advance();
                let lit = self.token_to_literal(&tok)?;
                Ok(Pattern::Literal(lit, tok.span))
            }
            _ => Err(ParseError {
                message: format!("expected pattern, found '{}'", self.peek().lexeme),
                span: self.peek().span,
            }),
        }
    }

    fn token_to_literal(&self, tok: &Token) -> ParseResult<Literal> {
        match tok.kind {
            TokenKind::Int => Ok(Literal::Int(tok.lexeme.parse::<i64>().unwrap_or(0))),
            TokenKind::Float => Ok(Literal::Float(tok.lexeme.parse::<f64>().unwrap_or(0.0))),
            TokenKind::String => Ok(Literal::String(tok.lexeme.clone())),
            TokenKind::True => Ok(Literal::Bool(true)),
            TokenKind::False => Ok(Literal::Bool(false)),
            TokenKind::Null => Ok(Literal::Null),
            _ => Err(ParseError {
                message: format!("'{}' is not a literal", tok.lexeme),
                span: tok.span,
            }),
        }
    }

    fn parse_type(&mut self) -> ParseResult<Type> {
        let mut ty = self.parse_type_base()?;
        if self.match_kind(TokenKind::Question).is_some() {
            let last = self.tokens.get(self.pos.saturating_sub(1)).unwrap();
            let span = ty.span().merge(last.span);
            ty = Type::Optional { inner: Box::new(ty), span };
        }
        if self.match_kind(TokenKind::Amp).is_some() {
            let is_mut = self.match_kind(TokenKind::Mut).is_some();
            let last = self.tokens.get(self.pos.saturating_sub(1)).unwrap();
            let span = ty.span().merge(last.span);
            ty = Type::Reference { inner: Box::new(ty), mutable: is_mut, span };
        }
        Ok(ty)
    }

    fn parse_type_base(&mut self) -> ParseResult<Type> {
        let tok = self.peek().clone();
        match tok.kind {
            TokenKind::Ident => {
                self.advance();
                match tok.lexeme.as_str() {
                    "int" => Ok(Type::Int(tok.span)),
                    "float" => Ok(Type::Float(tok.span)),
                    "bool" => Ok(Type::Bool(tok.span)),
                    "string" => Ok(Type::String(tok.span)),
                    "null" => Ok(Type::Null(tok.span)),
                    "void" => Ok(Type::Void(tok.span)),
                    name => {
                        if self.peek_kind() == TokenKind::Lt {
                            self.advance();
                            let mut args = Vec::new();
                            args.push(self.parse_type()?);
                            while self.match_kind(TokenKind::Comma).is_some() {
                                args.push(self.parse_type()?);
                            }
                            let gt = self.expect(TokenKind::Gt)?;
                            let span = tok.span.merge(gt.span);
                            Ok(Type::Generic { name: name.to_string(), args, span })
                        } else {
                            Ok(Type::Named { name: name.to_string(), span: tok.span })
                        }
                    }
                }
            }
            TokenKind::LParen => {
                self.advance();
                let mut params = Vec::new();
                if self.peek_kind() != TokenKind::RParen {
                    params.push(self.parse_type()?);
                    while self.match_kind(TokenKind::Comma).is_some() {
                        params.push(self.parse_type()?);
                    }
                }
            self.expect(TokenKind::RParen)?;
            if self.match_kind(TokenKind::Arrow).is_some() {
                    let ret = self.parse_type()?;
                    let last = self.tokens.get(self.pos.saturating_sub(1)).unwrap();
                    let span = tok.span.merge(last.span);
                    Ok(Type::Func { params, ret: Box::new(ret), span })
                } else if params.len() == 1 {
                    Ok(params.into_iter().next().unwrap())
                } else {
                    let last = self.tokens.get(self.pos.saturating_sub(1)).unwrap();
                    let span = tok.span.merge(last.span);
                    Ok(Type::Tuple(params, span))
                }
            }
            TokenKind::LBracket => {
                self.advance();
                let elem = self.parse_type()?;
                self.expect(TokenKind::RBracket)?;
                let last = self.tokens.get(self.pos.saturating_sub(1)).unwrap();
                let span = tok.span.merge(last.span);
                Ok(Type::Array { elem: Box::new(elem), span })
            }
            _ => Err(ParseError {
                message: format!("expected type, found '{}'", tok.lexeme),
                span: tok.span,
            }),
        }
    }
}

impl Expr {
    fn span(&self) -> Span {
        match self {
            Expr::Literal(_, s) => *s,
            Expr::Ident(_, s) => *s,
            Expr::Binary { span, .. } => *span,
            Expr::Unary { span, .. } => *span,
            Expr::Call { span, .. } => *span,
            Expr::Block { span, .. } => *span,
            Expr::If { span, .. } => *span,
            Expr::Lambda { span, .. } => *span,
            Expr::Index { span, .. } => *span,
            Expr::Member { span, .. } => *span,
            Expr::Match { span, .. } => *span,
            Expr::Tuple { span, .. } => *span,
            Expr::Array { span, .. } => *span,
            Expr::StructInit { span, .. } => *span,
            Expr::Range { span, .. } => *span,
            Expr::Placeholder(s) => *s,
        }
    }
}
