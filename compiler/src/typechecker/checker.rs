use std::collections::HashMap;

use crate::ast::expr::{BinaryOp, Expr, Param, UnaryOp};
use crate::ast::literal::Literal;
use crate::ast::r#type::Type;
use crate::ast::span::Span;
use crate::ast::stmt::{Stmt, StmtKind};
use crate::diagnostics::error::{Diagnostic, Diagnostics};

#[derive(Debug, Clone)]
struct Scope {
    vars: HashMap<String, Type>,
    mutable_vars: HashMap<String, bool>,
}

impl Scope {
    fn new() -> Self {
        Scope { vars: HashMap::new(), mutable_vars: HashMap::new() }
    }

    fn insert(&mut self, name: String, ty: Type, mutable: bool) {
        self.mutable_vars.insert(name.clone(), mutable);
        self.vars.insert(name, ty);
    }

    fn get(&self, name: &str) -> Option<&Type> {
        self.vars.get(name)
    }

    fn is_mutable(&self, name: &str) -> bool {
        self.mutable_vars.get(name).copied().unwrap_or(false)
    }
}

pub struct TypeChecker {
    scopes: Vec<Scope>,
    functions: HashMap<String, (Vec<Param>, Option<Type>)>,
    structs: HashMap<String, HashMap<String, Type>>,
    diagnostics: Diagnostics,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut tc = TypeChecker {
            scopes: vec![Scope::new()],
            functions: HashMap::new(),
            structs: HashMap::new(),
            diagnostics: Diagnostics::new(),
        };
        tc.register_builtins();
        tc
    }

    fn register_builtins(&mut self) {
        let builtins = [
        ("print", vec!["msg"], vec![Type::Any(Span::zero())], Type::Void(Span::zero())),
        ("println", vec!["msg"], vec![Type::Any(Span::zero())], Type::Void(Span::zero())),
            ("len", vec!["val"], vec![Type::Any(Span::zero())], Type::Int(Span::zero())),
            ("to_string", vec!["val"], vec![Type::Int(Span::zero())], Type::String(Span::zero())),
        ];
        for (name, _, param_types, ret) in builtins {
            let params = param_types.into_iter().enumerate().map(|(i, ty)| Param {
                ident: format!("arg{i}"),
                ty,
                span: Span::zero(),
            }).collect();
            self.functions.insert(name.to_string(), (params, Some(ret)));
        }
    }

    pub fn check(&mut self, stmts: &[Stmt]) -> Result<(), Diagnostics> {
        for stmt in stmts {
            self.check_stmt(stmt);
        }
        if self.diagnostics.has_errors() {
            Err(std::mem::take(&mut self.diagnostics))
        } else {
            Ok(())
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    fn declare_var(&mut self, name: &str, ty: &Type, mutable: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), ty.clone(), mutable);
        }
    }

    fn lookup_var(&self, name: &str) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty.clone());
            }
        }
        None
    }

    fn lookup_var_mutable(&self, name: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if scope.get(name).is_some() {
                return scope.is_mutable(name);
            }
        }
        false
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Let { pattern, ty, init } => {
            if let Some(init_expr) = init {
                let init_ty = self.infer_expr(init_expr);
                if let Some(ann_ty) = ty
                    && !self.types_compatible(ann_ty, &init_ty)
                {
                    self.diagnostics.emit(Diagnostic::error_at(
                        format!("type mismatch: expected {}, found {}", ann_ty.name(), init_ty.name()),
                        stmt.span,
                    ));
                }
                self.bind_pattern(pattern, &init_ty);
            } else if let Some(ann_ty) = ty {
                self.bind_pattern(pattern, ann_ty);
            }
            }
        StmtKind::Const { ident, ty, init } => {
            let init_ty = self.infer_expr(init);
            if let Some(ann_ty) = ty
                && !self.types_compatible(ann_ty, &init_ty)
            {
                self.diagnostics.emit(Diagnostic::error_at(
                    format!("type mismatch in const: expected {}, found {}", ann_ty.name(), init_ty.name()),
                    stmt.span,
                ));
            }
            self.declare_var(ident, &init_ty, false);
        }
            StmtKind::Func { name, params, ret_type, body, .. } => {
                self.functions.insert(name.clone(), (params.clone(), ret_type.clone()));
                self.push_scope();
                for p in params {
                    self.declare_var(&p.ident, &p.ty, false);
                }
                let body_ty = self.infer_expr(body);
            if let Some(ret) = ret_type
                && !self.is_void(&body_ty) && !self.types_compatible(ret, &body_ty) && !self.is_void(ret)
            {
                self.diagnostics.emit(Diagnostic::warning_at(
                    format!("function '{}' body type {} may not match declared return type {}", name, body_ty.name(), ret.name()),
                    stmt.span,
                ));
            }
                self.pop_scope();
            }
            StmtKind::Struct { name, fields } => {
                let mut field_map = HashMap::new();
                for f in fields {
                    field_map.insert(f.name.clone(), f.ty.clone());
                }
                self.structs.insert(name.clone(), field_map);
            }
            StmtKind::Enum { .. } => {}
            StmtKind::Impl { target, methods } => {
                for m in methods {
                    self.check_stmt(m);
                }
                let _ = target;
            }
            StmtKind::Import { .. } => {}
            StmtKind::Return { value } => {
                if let Some(v) = value {
                    self.infer_expr(v);
                }
            }
            StmtKind::If { cond, then_branch, else_branch } => {
                let cond_ty = self.infer_expr(cond);
                if !self.is_bool(&cond_ty) {
                    self.diagnostics.emit(Diagnostic::error_at(
                        format!("if condition must be bool, found {}", cond_ty.name()),
                        stmt.span,
                    ));
                }
                self.infer_expr(then_branch);
                if let Some(e) = else_branch {
                    self.infer_expr(e);
                }
            }
            StmtKind::While { cond, body } => {
                let cond_ty = self.infer_expr(cond);
                if !self.is_bool(&cond_ty) {
                    self.diagnostics.emit(Diagnostic::error_at(
                        format!("while condition must be bool, found {}", cond_ty.name()),
                        stmt.span,
                    ));
                }
                self.infer_expr(body);
            }
            StmtKind::For { ident, iter, body, .. } => {
                let iter_ty = self.infer_expr(iter);
                let elem_ty = match &iter_ty {
                    Type::Array { elem, .. } => elem.as_ref().clone(),
                    _ => Type::Any(stmt.span),
                };
                self.push_scope();
                self.declare_var(ident, &elem_ty, false);
                self.infer_expr(body);
                self.pop_scope();
            }
            StmtKind::Break | StmtKind::Continue => {}
            StmtKind::Assign { target, value } => {
                if let Expr::Ident(name, span) = target
                    && !self.lookup_var_mutable(name)
                {
                    self.diagnostics.emit(Diagnostic::error_at(
                        format!("cannot assign to immutable variable '{name}', declare with 'mut'"),
                        *span,
                    ));
                }
                let target_ty = self.infer_expr(target);
                let value_ty = self.infer_expr(value);
                if !self.types_compatible(&target_ty, &value_ty) {
                    self.diagnostics.emit(Diagnostic::error_at(
                        format!("cannot assign {} to {}", value_ty.name(), target_ty.name()),
                        stmt.span,
                    ));
                }
            }
            StmtKind::Expr(e) => { self.infer_expr(e); }
            StmtKind::Semi(e) => { self.infer_expr(e); }
        }
    }

    fn infer_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Literal(lit, span) => match lit {
                Literal::Int(_) => Type::Int(*span),
                Literal::Float(_) => Type::Float(*span),
                Literal::Bool(_) => Type::Bool(*span),
                Literal::String(_) => Type::String(*span),
                Literal::Null => Type::Null(*span),
            },
            Expr::Ident(name, span) => {
                if let Some(ty) = self.lookup_var(name) {
                    ty
                } else {
                    self.diagnostics.emit(Diagnostic::error_at(
                        format!("undefined variable '{name}'"),
                        *span,
                    ));
                    Type::Void(*span)
                }
            }
            Expr::Binary { op, left, right, span } => {
                let lt = self.infer_expr(left);
                let rt = self.infer_expr(right);
                match op {
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Rem => {
                        if self.is_numeric(&lt) && self.is_numeric(&rt) {
                            if self.is_float(&lt) || self.is_float(&rt) {
                                Type::Float(*span)
                            } else {
                                Type::Int(*span)
                            }
                        } else if self.is_string(&lt) && self.is_string(&rt) && *op == BinaryOp::Add {
                            Type::String(*span)
                        } else {
                            self.diagnostics.emit(Diagnostic::error_at(
                                format!("cannot apply {:?} to {} and {}", op, lt.name(), rt.name()),
                                *span,
                            ));
                            Type::Void(*span)
                        }
                    }
                    BinaryOp::Eq | BinaryOp::Neq | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                        Type::Bool(*span)
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        if !self.is_bool(&lt) || !self.is_bool(&rt) {
                            self.diagnostics.emit(Diagnostic::error_at(
                                "logical operators require bool operands",
                                *span,
                            ));
                        }
                        Type::Bool(*span)
                    }
                    BinaryOp::Assign => {
                        if !self.types_compatible(&lt, &rt) {
                            self.diagnostics.emit(Diagnostic::error_at(
                                format!("cannot assign {} to {}", rt.name(), lt.name()),
                                *span,
                            ));
                        }
                        lt
                    }
                }
            }
            Expr::Unary { op, expr, span } => {
                let inner = self.infer_expr(expr);
                match op {
                    UnaryOp::Neg => {
                        if !self.is_numeric(&inner) {
                            self.diagnostics.emit(Diagnostic::error_at(
                                format!("cannot negate {}", inner.name()),
                                *span,
                            ));
                        }
                        inner
                    }
                    UnaryOp::Not => {
                        if !self.is_bool(&inner) {
                            self.diagnostics.emit(Diagnostic::error_at(
                                format!("cannot apply ! to {}", inner.name()),
                                *span,
                            ));
                        }
                        Type::Bool(*span)
                    }
                    UnaryOp::Ref => Type::Reference { inner: Box::new(inner), mutable: false, span: *span },
                    UnaryOp::Deref => inner,
                }
            }
            Expr::Await { expr, .. } => self.infer_expr(expr),
            Expr::Call { callee, args, span } => {
                if let Expr::Ident(name, _) = callee.as_ref() {
                    if let Some((params, ret)) = self.functions.get(name).cloned() {
                        if args.len() != params.len() {
                            self.diagnostics.emit(Diagnostic::error_at(
                                format!("function '{}' expects {} args, got {}", name, params.len(), args.len()),
                                *span,
                            ));
                        }
                        for (i, arg) in args.iter().enumerate() {
                            let arg_ty = self.infer_expr(arg);
                if let Some(p) = params.get(i)
                    && !self.types_compatible(&p.ty, &arg_ty)
                {
                    self.diagnostics.emit(Diagnostic::error_at(
                        format!("arg {} of '{}': expected {}, got {}", i + 1, name, p.ty.name(), arg_ty.name()),
                        *span,
                    ));
                }
                        }
                        ret.unwrap_or(Type::Void(*span))
                    } else {
                        for a in args {
                            self.infer_expr(a);
                        }
                        Type::Void(*span)
                    }
                } else {
                    let callee_ty = self.infer_expr(callee);
                    for a in args {
                        self.infer_expr(a);
                    }
                    if let Type::Func { ret, .. } = &callee_ty {
                        ret.as_ref().clone()
                    } else {
                        Type::Void(*span)
                    }
                }
            }
            Expr::Block { stmts, final_expr, span } => {
                self.push_scope();
                for s in stmts {
                    self.check_stmt(s);
                }
                let ty = if let Some(e) = final_expr {
                    self.infer_expr(e)
                } else {
                    Type::Void(*span)
                };
                self.pop_scope();
                ty
            }
            Expr::If { cond, then_branch, else_branch, span } => {
                let cond_ty = self.infer_expr(cond);
                if !self.is_bool(&cond_ty) {
                    self.diagnostics.emit(Diagnostic::error_at("if condition must be bool", *span));
                }
                let then_ty = self.infer_expr(then_branch);
                if let Some(e) = else_branch {
                    let else_ty = self.infer_expr(e);
                    if !self.types_compatible(&then_ty, &else_ty) {
                        self.diagnostics.emit(Diagnostic::warning_at(
                            "if/else branches have different types",
                            *span,
                        ));
                    }
                }
                then_ty
            }
            Expr::Lambda { params, ret_type, body, span } => {
                self.push_scope();
                for p in params {
                    self.declare_var(&p.ident, &p.ty, false);
                }
                let body_ty = self.infer_expr(body);
                self.pop_scope();
                let ret = ret_type.as_ref().unwrap_or(&body_ty).clone();
                Type::Func {
                    params: params.iter().map(|p| p.ty.clone()).collect(),
                    ret: Box::new(ret),
                    span: *span,
                }
            }
            Expr::Index { target, index, span } => {
                let target_ty = self.infer_expr(target);
                let index_ty = self.infer_expr(index);
                if !self.is_int(&index_ty) {
                    self.diagnostics.emit(Diagnostic::error_at("index must be int", *span));
                }
                match &target_ty {
                    Type::Array { elem, .. } => elem.as_ref().clone(),
                    _ => {
                        self.diagnostics.emit(Diagnostic::error_at(
                            format!("cannot index into {}", target_ty.name()),
                            *span,
                        ));
                        Type::Void(*span)
                    }
                }
            }
            Expr::Member { target, field, span } => {
                let target_ty = self.infer_expr(target);
                match &target_ty {
                    Type::Named { name, .. } => {
                        if let Some(fields) = self.structs.get(name) {
                            if let Some(f_ty) = fields.get(field) {
                                f_ty.clone()
                            } else {
                                self.diagnostics.emit(Diagnostic::error_at(
                                    format!("struct '{name}' has no field '{field}'"),
                                    *span,
                                ));
                                Type::Void(*span)
                            }
                        } else {
                            Type::Void(*span)
                        }
                    }
                    _ => {
                        self.diagnostics.emit(Diagnostic::error_at(
                            format!("cannot access field '{field}' on {}", target_ty.name()),
                            *span,
                        ));
                        Type::Void(*span)
                    }
                }
            }
            Expr::Match { scrutinee, arms, span } => {
                let _scrut_ty = self.infer_expr(scrutinee);
                let mut result_ty = None;
                for (pat, body) in arms {
                    let _ = pat;
                    let body_ty = self.infer_expr(body);
                    if result_ty.is_none() {
                        result_ty = Some(body_ty);
                    }
                }
                result_ty.unwrap_or(Type::Void(*span))
            }
            Expr::Tuple { elems, span } => {
                let types = elems.iter().map(|e| self.infer_expr(e)).collect();
                Type::Tuple(types, *span)
            }
            Expr::Array { elems, span } => {
                let elem_ty = elems.first().map(|e| self.infer_expr(e)).unwrap_or(Type::Void(*span));
                Type::Array { elem: Box::new(elem_ty), span: *span }
            }
        Expr::StructInit { name, fields, span } => {
            if let Some(field_map) = self.structs.get(name).cloned() {
                for (f_name, val) in fields {
                    let val_ty = self.infer_expr(val);
                    if let Some(expected_ty) = field_map.get(f_name) {
                        if !self.types_compatible(expected_ty, &val_ty) {
                            self.diagnostics.emit(Diagnostic::error_at(
                                format!("struct '{name}' field '{f_name}': expected {}, found {}", expected_ty.name(), val_ty.name()),
                                *span,
                            ));
                        }
                    } else {
                        self.diagnostics.emit(Diagnostic::error_at(
                            format!("struct '{name}' has no field '{f_name}'"),
                            *span,
                        ));
                    }
                }
                let defined_fields: std::collections::HashSet<&String> = field_map.keys().collect();
                let init_fields: std::collections::HashSet<&String> = fields.iter().map(|(n, _)| n).collect();
                for missing in defined_fields.difference(&init_fields) {
                    self.diagnostics.emit(Diagnostic::warning_at(
                        format!("struct '{name}' field '{missing}' not initialized"),
                        *span,
                    ));
                }
            } else {
                self.diagnostics.emit(Diagnostic::error_at(
                    format!("undefined struct '{name}'"),
                    *span,
                ));
                for (_, val) in fields {
                    self.infer_expr(val);
                }
            }
            Type::Named { name: name.clone(), span: *span }
        }
            Expr::Range { start, end, span } => {
                self.infer_expr(start);
                self.infer_expr(end);
                Type::Array { elem: Box::new(Type::Int(*span)), span: *span }
            }
            Expr::Placeholder(span) => Type::Void(*span),
        }
    }

    fn bind_pattern(&mut self, pat: &crate::ast::pattern::Pattern, ty: &Type) {
        use crate::ast::pattern::Pattern;
        match pat {
            Pattern::Ident(name, _) => {
                self.declare_var(name, ty, false);
            }
            Pattern::MutIdent(name, _) => {
                self.declare_var(name, ty, true);
            }
            Pattern::Wildcard(_) => {}
            Pattern::Tuple(pats, _) => {
                if let Type::Tuple(types, _) = ty {
                    for (p, t) in pats.iter().zip(types.iter()) {
                        self.bind_pattern(p, t);
                    }
                }
            }
            Pattern::Struct { name, fields, .. } => {
                if let Some(field_map) = self.structs.get(name).cloned() {
                    for (f_name, f_pat) in fields {
                        if let Some(f_ty) = field_map.get(f_name) {
                            self.bind_pattern(f_pat, f_ty);
                        }
                    }
                }
            }
            Pattern::Rest(_) => {}
            Pattern::Literal(_, _) => {}
        }
    }

    fn types_compatible(&self, expected: &Type, found: &Type) -> bool {
        let en = expected.name();
        let fn_ = found.name();
        if en == "any" { return true; }
        if en == fn_ { return true; }
        if en == "float" && fn_ == "int" { return true; }
        if en == "null" { return true; }
        if self.is_void(expected) { return true; }
        false
    }

    fn is_numeric(&self, ty: &Type) -> bool {
        matches!(ty, Type::Int(_) | Type::Float(_))
    }

    fn is_int(&self, ty: &Type) -> bool {
        matches!(ty, Type::Int(_))
    }

    fn is_float(&self, ty: &Type) -> bool {
        matches!(ty, Type::Float(_))
    }

    fn is_bool(&self, ty: &Type) -> bool {
        matches!(ty, Type::Bool(_))
    }

    fn is_string(&self, ty: &Type) -> bool {
        matches!(ty, Type::String(_))
    }

    fn is_void(&self, ty: &Type) -> bool {
        matches!(ty, Type::Void(_))
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
