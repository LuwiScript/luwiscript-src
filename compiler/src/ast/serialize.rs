use crate::ast::expr::{BinaryOp, Expr, Param, UnaryOp};
use crate::ast::literal::Literal;
use crate::ast::stmt::{EnumVariant, Stmt, StmtKind, StructField};

pub fn serialize_expr(expr: &Expr) -> String {
    match expr {
        Expr::Literal(lit, _) => serialize_literal(lit),
        Expr::Ident(name, _) => name.clone(),
        Expr::Binary { op, left, right, .. } => {
            format!("({} {} {})", serialize_expr(left), serialize_binop(op), serialize_expr(right))
        }
        Expr::Unary { op, expr, .. } => {
            format!("({}{})", serialize_unaryop(op), serialize_expr(expr))
        }
        Expr::Await { expr, .. } => format!("await {}", serialize_expr(expr)),
        Expr::Call { callee, args, .. } => {
            let a = args.iter().map(serialize_expr).collect::<Vec<_>>().join(", ");
            format!("{}({})", serialize_expr(callee), a)
        }
        Expr::Block { stmts, final_expr, .. } => {
            let mut out = String::from("{ ");
            for s in stmts {
                out.push_str(&serialize_stmt(s));
                out.push_str("; ");
            }
            if let Some(e) = final_expr {
                out.push_str(&serialize_expr(e));
            }
            out.push_str(" }");
            out
        }
        Expr::If { cond, then_branch, else_branch, .. } => {
            let mut out = format!("if {} {}", serialize_expr(cond), serialize_expr(then_branch));
            if let Some(e) = else_branch {
                out.push_str(&format!(" else {}", serialize_expr(e)));
            }
            out
        }
        Expr::Lambda { params, body, .. } => {
            let p = params.iter().map(|Param { ident, ty, .. }| format!("{}: {}", ident, ty.name())).collect::<Vec<_>>().join(", ");
            format!("|{}| {}", p, serialize_expr(body))
        }
        Expr::Index { target, index, .. } => {
            format!("{}[{}]", serialize_expr(target), serialize_expr(index))
        }
        Expr::Member { target, field, .. } => {
            format!("{}.{}", serialize_expr(target), field)
        }
        Expr::Match { scrutinee, arms, .. } => {
            let mut out = format!("match {} {{ ", serialize_expr(scrutinee));
            for (pat, body) in arms {
                out.push_str(&format!("{} => {}, ", serialize_pattern(pat), serialize_expr(body)));
            }
            out.push('}');
            out
        }
        Expr::Tuple { elems, .. } => {
            let e = elems.iter().map(serialize_expr).collect::<Vec<_>>().join(", ");
            format!("({e})")
        }
        Expr::Array { elems, .. } => {
            let e = elems.iter().map(serialize_expr).collect::<Vec<_>>().join(", ");
            format!("[{e}]")
        }
            Expr::StructInit { name, fields, .. } => {
                let f = fields.iter().map(|(n, v)| format!("{n}: {}", serialize_expr(v))).collect::<Vec<_>>().join(", ");
                format!("{name} {{ {f} }}")
            }
            Expr::Range { start, end, .. } => {
                format!("{}..{}", serialize_expr(start), serialize_expr(end))
            }
            Expr::Placeholder(_) => "_".into(),
    }
}

pub fn serialize_stmt(stmt: &Stmt) -> String {
    match &stmt.kind {
        StmtKind::Expr(e) => serialize_expr(e),
        StmtKind::Let { pattern, ty, init } => {
            let mut out = format!("let {}", serialize_pattern(pattern));
            if let Some(t) = ty {
                out.push_str(&format!(": {}", t.name()));
            }
            if let Some(e) = init {
                out.push_str(&format!(" = {}", serialize_expr(e)));
            }
            out
        }
        StmtKind::Const { ident, ty, init } => {
            let mut out = format!("const {ident}");
            if let Some(t) = ty {
                out.push_str(&format!(": {}", t.name()));
            }
            out.push_str(&format!(" = {}", serialize_expr(init)));
            out
        }
        StmtKind::Func { name, params, ret_type, body, is_async } => {
            let p = params.iter().map(|Param { ident, ty, .. }| format!("{}: {}", ident, ty.name())).collect::<Vec<_>>().join(", ");
            let mut out = if *is_async { format!("async fn {name}({p})") } else { format!("fn {name}({p})") };
            if let Some(r) = ret_type {
                out.push_str(&format!(" -> {}", r.name()));
            }
            out.push_str(&format!(" {}", serialize_expr(body)));
            out
        }
        StmtKind::Return { value } => {
            match value {
                Some(e) => format!("return {}", serialize_expr(e)),
                None => "return".into(),
            }
        }
        StmtKind::If { cond, then_branch, else_branch } => {
            let mut out = format!("if {} {}", serialize_expr(cond), serialize_expr(then_branch));
            if let Some(e) = else_branch {
                out.push_str(&format!(" else {}", serialize_expr(e)));
            }
            out
        }
        StmtKind::While { cond, body } => {
            format!("while {} {}", serialize_expr(cond), serialize_expr(body))
        }
        StmtKind::For { ident, iter, body } => {
            format!("for {ident} in {} {}", serialize_expr(iter), serialize_expr(body))
        }
        StmtKind::Break => "break".into(),
        StmtKind::Continue => "continue".into(),
        StmtKind::Assign { target, value } => {
            format!("{} = {}", serialize_expr(target), serialize_expr(value))
        }
        StmtKind::Import { path, alias } => {
            match alias {
                Some(a) => format!("import {path} as {a}"),
                None => format!("import {path}"),
            }
        }
        StmtKind::Struct { name, fields } => {
            let f = fields.iter().map(|StructField { name, ty, .. }| format!("  {name}: {}", ty.name())).collect::<Vec<_>>().join("\n");
            format!("struct {name} {{\n{f}\n}}")
        }
        StmtKind::Enum { name, variants } => {
            let v = variants.iter().map(|EnumVariant { name, fields }| {
                if fields.is_empty() {
                    format!("  {name}")
                } else {
                    let f = fields.iter().map(|t| t.name()).collect::<Vec<_>>().join(", ");
                    format!("  {name}({f})")
                }
            }).collect::<Vec<_>>().join("\n");
            format!("enum {name} {{\n{v}\n}}")
        }
        StmtKind::Impl { target, methods } => {
            let m = methods.iter().map(serialize_stmt).collect::<Vec<_>>().join("\n");
            format!("impl {} {{\n{m}\n}}", target.name())
        }
        StmtKind::Semi(e) => format!("{};", serialize_expr(e)),
    }
}

fn serialize_literal(lit: &Literal) -> String {
    match lit {
        Literal::Int(n) => n.to_string(),
        Literal::Float(n) => n.to_string(),
        Literal::Bool(b) => b.to_string(),
        Literal::String(s) => format!("\"{s}\""),
        Literal::Null => "null".into(),
    }
}

fn serialize_binop(op: &BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Rem => "%",
        BinaryOp::Eq => "==",
        BinaryOp::Neq => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Le => "<=",
        BinaryOp::Gt => ">",
        BinaryOp::Ge => ">=",
        BinaryOp::And => "&&",
        BinaryOp::Or => "||",
        BinaryOp::Assign => "=",
    }
}

fn serialize_unaryop(op: &UnaryOp) -> &'static str {
    match op {
        UnaryOp::Neg => "-",
        UnaryOp::Not => "!",
        UnaryOp::Ref => "&",
        UnaryOp::Deref => "*",
    }
}

use crate::ast::pattern::Pattern;

fn serialize_pattern(pat: &Pattern) -> String {
    match pat {
        Pattern::Ident(name, _) => name.clone(),
        Pattern::MutIdent(name, _) => format!("mut {name}"),
        Pattern::Wildcard(_) => "_".into(),
        Pattern::Literal(lit, _) => serialize_literal(lit),
        Pattern::Tuple(pats, _) => {
            let p = pats.iter().map(serialize_pattern).collect::<Vec<_>>().join(", ");
            format!("({p})")
        }
        Pattern::Struct { name, fields, .. } => {
            let f = fields.iter().map(|(n, p)| format!("{n}: {}", serialize_pattern(p))).collect::<Vec<_>>().join(", ");
            format!("{name} {{ {f} }}")
        }
        Pattern::Rest(_) => "..".into(),
    }
}
