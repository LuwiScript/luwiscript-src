use crate::ast::expr::{BinaryOp, Expr, UnaryOp};
use crate::ast::literal::Literal;
use crate::ast::stmt::Stmt;

pub trait Visitor {
    fn visit_expr(&mut self, expr: &Expr)
    where
        Self: Sized,
    {
        walk_expr(self, expr)
    }
    fn visit_stmt(&mut self, stmt: &Stmt)
    where
        Self: Sized,
    {
        walk_stmt(self, stmt)
    }
    fn visit_literal(&mut self, _lit: &Literal) {}
    fn visit_binary_op(&mut self, _op: BinaryOp, _left: &Expr, _right: &Expr) {}
    fn visit_unary_op(&mut self, _op: UnaryOp, _expr: &Expr) {}
}

pub fn walk_expr<V: Visitor>(v: &mut V, expr: &Expr) {
    match expr {
        Expr::Literal(lit, _) => {
            v.visit_literal(lit);
        }
        Expr::Ident(_, _) => {}
        Expr::Binary { left, right, .. } => {
            v.visit_expr(left);
            v.visit_expr(right);
        }
        Expr::Unary { expr, .. } => {
            v.visit_expr(expr);
        }
        Expr::Await { expr, .. } => {
            v.visit_expr(expr);
        }
        Expr::Call { callee, args, .. } => {
            v.visit_expr(callee);
            for arg in args {
                v.visit_expr(arg);
            }
        }
        Expr::Block { stmts, final_expr, .. } => {
            for s in stmts {
                v.visit_stmt(s);
            }
            if let Some(e) = final_expr {
                v.visit_expr(e);
            }
        }
        Expr::If { cond, then_branch, else_branch, .. } => {
            v.visit_expr(cond);
            v.visit_expr(then_branch);
            if let Some(e) = else_branch {
                v.visit_expr(e);
            }
        }
        Expr::Lambda { body, .. } => {
            v.visit_expr(body);
        }
        Expr::Index { target, index, .. } => {
            v.visit_expr(target);
            v.visit_expr(index);
        }
        Expr::Member { target, .. } => {
            v.visit_expr(target);
        }
        Expr::Match { scrutinee, arms, .. } => {
            v.visit_expr(scrutinee);
            for (_, body) in arms {
                v.visit_expr(body);
            }
        }
        Expr::Tuple { elems, .. } => {
            for e in elems {
                v.visit_expr(e);
            }
        }
        Expr::Array { elems, .. } => {
            for e in elems {
                v.visit_expr(e);
            }
        }
            Expr::StructInit { fields, .. } => {
                for (_, val) in fields {
                    v.visit_expr(val);
                }
            }
            Expr::Range { start, end, .. } => {
                v.visit_expr(start);
                v.visit_expr(end);
            }
            Expr::Placeholder(_) => {}
    }
}

pub fn walk_stmt<V: Visitor>(v: &mut V, stmt: &Stmt) {
    match &stmt.kind {
        crate::ast::stmt::StmtKind::Expr(e) => {
            v.visit_expr(e);
        }
        crate::ast::stmt::StmtKind::Let { init, .. } => {
            if let Some(e) = init {
                v.visit_expr(e);
            }
        }
        crate::ast::stmt::StmtKind::Const { init, .. } => {
            v.visit_expr(init);
        }
        crate::ast::stmt::StmtKind::Func { body, .. } => {
            v.visit_expr(body);
        }
        crate::ast::stmt::StmtKind::Return { value } => {
            if let Some(e) = value {
                v.visit_expr(e);
            }
        }
        crate::ast::stmt::StmtKind::If { cond, then_branch, else_branch } => {
            v.visit_expr(cond);
            v.visit_expr(then_branch);
            if let Some(e) = else_branch {
                v.visit_expr(e);
            }
        }
        crate::ast::stmt::StmtKind::While { cond, body } => {
            v.visit_expr(cond);
            v.visit_expr(body);
        }
        crate::ast::stmt::StmtKind::For { iter, body, .. } => {
            v.visit_expr(iter);
            v.visit_expr(body);
        }
        crate::ast::stmt::StmtKind::Break | crate::ast::stmt::StmtKind::Continue => {}
        crate::ast::stmt::StmtKind::Assign { target, value } => {
            v.visit_expr(target);
            v.visit_expr(value);
        }
        crate::ast::stmt::StmtKind::Import { .. } => {}
        crate::ast::stmt::StmtKind::Struct { .. } => {}
        crate::ast::stmt::StmtKind::Enum { .. } => {}
        crate::ast::stmt::StmtKind::Impl { methods, .. } => {
            for m in methods {
                v.visit_stmt(m);
            }
        }
        crate::ast::stmt::StmtKind::Semi(e) => {
            v.visit_expr(e);
        }
    }
}
