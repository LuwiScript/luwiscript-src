use crate::ast::expr::{BinaryOp, Expr, UnaryOp};
use crate::ast::literal::Literal;
use crate::ast::stmt::{Stmt, StmtKind};

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    PushNull,
    PushInt(i64),
    PushFloat(f64),
    PushBool(bool),
    PushString(String),
    LoadLocal(usize),
    StoreLocal(usize),
    LoadGlobal(usize),
    StoreGlobal(usize),
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Neq,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Neg,
    Not,
    Jump(usize),
    JumpIfFalse(usize),
    JumpIfTrue(usize),
    Call { chunk_idx: usize, argc: usize },
    Return,
    Print,
    Println,
    Len,
    Halt,
    Pop,
    Dup,
    IndexGet,
    IndexSet,
    MemberGet(usize),
    MakeArray(usize),
    MakeTuple(usize),
    MakeStruct { field_count: usize },
    MakeRange,
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub ops: Vec<Op>,
    pub constants: Vec<Constant>,
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum Constant {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Null,
}

impl Chunk {
    pub fn new(name: impl Into<String>) -> Self {
        Chunk { ops: Vec::new(), constants: Vec::new(), name: name.into() }
    }

    pub fn emit(&mut self, op: Op) -> usize {
        let idx = self.ops.len();
        self.ops.push(op);
        idx
    }

    pub fn patch(&mut self, idx: usize, op: Op) {
        self.ops[idx] = op;
    }

    pub fn add_constant(&mut self, c: Constant) -> usize {
        let idx = self.constants.len();
        self.constants.push(c);
        idx
    }
}

pub struct CodeGen {
    chunks: Vec<Chunk>,
    locals: Vec<String>,
    local_depth: Vec<usize>,
    current_depth: usize,
    func_names: Vec<String>,
}

impl CodeGen {
    pub fn new() -> Self {
        CodeGen {
            chunks: Vec::new(),
            locals: Vec::new(),
            local_depth: Vec::new(),
            current_depth: 0,
            func_names: Vec::new(),
        }
    }

    pub fn compile(&mut self, stmts: &[Stmt]) -> Vec<Chunk> {
        let mut main_chunk = Chunk::new("main");
        for stmt in stmts {
            self.compile_stmt(stmt, &mut main_chunk);
        }
        main_chunk.emit(Op::Halt);
        self.chunks.insert(0, main_chunk);
        std::mem::take(&mut self.chunks)
    }

    fn begin_scope(&mut self) {
        self.current_depth += 1;
    }

    fn end_scope(&mut self, _chunk: &mut Chunk) {
        self.current_depth -= 1;
        while !self.locals.is_empty()
            && self.local_depth.last().copied() > Some(self.current_depth)
        {
            self.locals.pop();
            self.local_depth.pop();
        }
    }

    fn resolve_local(&self, name: &str) -> Option<usize> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local == name {
                return Some(i);
            }
        }
        None
    }

    fn declare_local(&mut self, name: &str) {
        self.locals.push(name.to_string());
        self.local_depth.push(self.current_depth);
    }

    fn compile_stmt(&mut self, stmt: &Stmt, chunk: &mut Chunk) {
        match &stmt.kind {
        StmtKind::Let { pattern, init, .. } => {
            if let Some(init_expr) = init {
                self.compile_expr(init_expr, chunk);
            } else {
                chunk.emit(Op::PushNull);
            }
            if let crate::ast::pattern::Pattern::Ident(name, _) = pattern {
                let idx = self.locals.len();
                self.declare_local(name);
                chunk.emit(Op::StoreLocal(idx));
            } else {
                chunk.emit(Op::Pop);
            }
        }
        StmtKind::Const { ident, init, .. } => {
            self.compile_expr(init, chunk);
            let idx = self.locals.len();
            self.declare_local(ident);
            chunk.emit(Op::StoreLocal(idx));
        }
        StmtKind::Func { name, params, body, .. } => {
            let func_chunk = Chunk::new(name.clone());
            let prev_locals = std::mem::take(&mut self.locals);
            let prev_depth = std::mem::take(&mut self.local_depth);
            let prev_curr = self.current_depth;
            self.current_depth = 0;
            for p in params {
                self.declare_local(&p.ident);
            }
            let mut fc = func_chunk;
            self.compile_expr(body, &mut fc);
            fc.emit(Op::Return);
            let _func_idx = self.chunks.len() + 1;
            self.chunks.push(fc);
            self.func_names.push(name.clone());
            self.locals = prev_locals;
            self.local_depth = prev_depth;
            self.current_depth = prev_curr;
            chunk.emit(Op::PushNull);
            let idx = self.locals.len();
            self.declare_local(name);
            chunk.emit(Op::StoreLocal(idx));
        }
            StmtKind::Expr(e) => {
                self.compile_expr(e, chunk);
            }
            StmtKind::Semi(e) => {
                self.compile_expr(e, chunk);
                chunk.emit(Op::Pop);
            }
            StmtKind::Return { value } => {
                if let Some(v) = value {
                    self.compile_expr(v, chunk);
                } else {
                    chunk.emit(Op::PushNull);
                }
                chunk.emit(Op::Return);
            }
            StmtKind::If { cond, then_branch, else_branch } => {
                self.compile_expr(cond, chunk);
                let jump_false = chunk.emit(Op::JumpIfFalse(0));
                chunk.emit(Op::Pop);
                self.compile_expr(then_branch, chunk);
                let jump_end = chunk.emit(Op::Jump(0));
                chunk.patch(jump_false, Op::JumpIfFalse(chunk.ops.len()));
                chunk.emit(Op::Pop);
                if let Some(e) = else_branch {
                    self.compile_expr(e, chunk);
                } else {
                    chunk.emit(Op::PushNull);
                }
                chunk.patch(jump_end, Op::Jump(chunk.ops.len()));
            }
            StmtKind::While { cond, body } => {
                let loop_start = chunk.ops.len();
                self.compile_expr(cond, chunk);
                let jump_out = chunk.emit(Op::JumpIfFalse(0));
                chunk.emit(Op::Pop);
                self.compile_expr(body, chunk);
                chunk.emit(Op::Pop);
                chunk.emit(Op::Jump(loop_start));
                chunk.patch(jump_out, Op::JumpIfFalse(chunk.ops.len()));
                chunk.emit(Op::Pop);
                chunk.emit(Op::PushNull);
            }
            StmtKind::For { ident, iter, body } => {
                self.compile_expr(iter, chunk);
                let arr_idx = self.locals.len();
                self.declare_local(&format!("__for_arr_{arr_idx}"));
                chunk.emit(Op::StoreLocal(arr_idx));
                let idx_idx = self.locals.len();
                self.declare_local(&format!("__for_idx_{idx_idx}"));
                chunk.emit(Op::PushInt(0));
                chunk.emit(Op::StoreLocal(idx_idx));
                let var_idx = self.locals.len();
                self.declare_local(ident);
                self.begin_scope();
                let loop_start = chunk.ops.len();
                chunk.emit(Op::LoadLocal(idx_idx));
                chunk.emit(Op::LoadLocal(arr_idx));
                chunk.emit(Op::Len);
                chunk.emit(Op::Lt);
                let jump_out = chunk.emit(Op::JumpIfFalse(0));
                chunk.emit(Op::LoadLocal(arr_idx));
                chunk.emit(Op::LoadLocal(idx_idx));
                chunk.emit(Op::IndexGet);
                chunk.emit(Op::StoreLocal(var_idx));
                self.compile_expr(body, chunk);
                chunk.emit(Op::Pop);
                chunk.emit(Op::LoadLocal(idx_idx));
                chunk.emit(Op::PushInt(1));
                chunk.emit(Op::Add);
                chunk.emit(Op::StoreLocal(idx_idx));
                chunk.emit(Op::Jump(loop_start));
                chunk.patch(jump_out, Op::JumpIfFalse(chunk.ops.len()));
                chunk.emit(Op::PushNull);
                self.end_scope(chunk);
            }
            StmtKind::Break | StmtKind::Continue => {}
            StmtKind::Assign { target, value } => {
                self.compile_expr(value, chunk);
                if let Expr::Ident(name, _) = target {
                    if let Some(idx) = self.resolve_local(name) {
                        chunk.emit(Op::StoreLocal(idx));
                    }
                }
            }
            StmtKind::Struct { .. } | StmtKind::Enum { .. } | StmtKind::Impl { .. } | StmtKind::Import { .. } => {}
        }
    }

    fn compile_expr(&mut self, expr: &Expr, chunk: &mut Chunk) {
        match expr {
            Expr::Literal(lit, _) => {
                match lit {
                    Literal::Int(n) => { chunk.emit(Op::PushInt(*n)); }
                    Literal::Float(n) => { chunk.emit(Op::PushFloat(*n)); }
                    Literal::Bool(b) => { chunk.emit(Op::PushBool(*b)); }
                    Literal::String(s) => { chunk.emit(Op::PushString(s.clone())); }
                    Literal::Null => { chunk.emit(Op::PushNull); }
                }
            }
            Expr::Ident(name, _) => {
                if let Some(idx) = self.resolve_local(name) {
                    chunk.emit(Op::LoadLocal(idx));
                } else {
                    chunk.emit(Op::PushNull);
                }
            }
            Expr::Binary { op, left, right, .. } => {
                self.compile_expr(left, chunk);
                self.compile_expr(right, chunk);
                let opcode = match op {
                    BinaryOp::Add => Op::Add,
                    BinaryOp::Sub => Op::Sub,
                    BinaryOp::Mul => Op::Mul,
                    BinaryOp::Div => Op::Div,
                    BinaryOp::Rem => Op::Rem,
                    BinaryOp::Eq => Op::Eq,
                    BinaryOp::Neq => Op::Neq,
                    BinaryOp::Lt => Op::Lt,
                    BinaryOp::Le => Op::Le,
                    BinaryOp::Gt => Op::Gt,
                    BinaryOp::Ge => Op::Ge,
                    BinaryOp::And => Op::And,
                    BinaryOp::Or => Op::Or,
                    BinaryOp::Assign => Op::Eq,
                };
                chunk.emit(opcode);
            }
            Expr::Unary { op, expr, .. } => {
                self.compile_expr(expr, chunk);
                match op {
                    UnaryOp::Neg => { chunk.emit(Op::Neg); }
                    UnaryOp::Not => { chunk.emit(Op::Not); }
                    UnaryOp::Ref | UnaryOp::Deref => { chunk.emit(Op::PushNull); }
                }
            }
            Expr::Call { callee, args, .. } => {
                if let Expr::Ident(name, _) = callee.as_ref() {
                    match name.as_str() {
                        "print" | "println" => {
                            if let Some(arg) = args.first() {
                                self.compile_expr(arg, chunk);
                            } else {
                                chunk.emit(Op::PushNull);
                            }
                            if name == "println" {
                                chunk.emit(Op::Println);
                            } else {
                                chunk.emit(Op::Print);
                            }
                            chunk.emit(Op::PushNull);
                            return;
                        }
                        "len" => {
                            if let Some(arg) = args.first() {
                                self.compile_expr(arg, chunk);
                            }
                            chunk.emit(Op::Len);
                            return;
                        }
            _ => {}
            }
            let func_idx = self.func_names.iter().position(|n| n == name)
                .map(|i| i + 1)
                .unwrap_or(0);
            for a in args {
                self.compile_expr(a, chunk);
            }
            chunk.emit(Op::Call { chunk_idx: func_idx, argc: args.len() });
        } else {
                    self.compile_expr(callee, chunk);
                    for a in args {
                        self.compile_expr(a, chunk);
                    }
                    chunk.emit(Op::Call { chunk_idx: 0, argc: args.len() });
                }
            }
            Expr::Block { stmts, final_expr, .. } => {
                self.begin_scope();
                for s in stmts {
                    self.compile_stmt(s, chunk);
                }
                if let Some(e) = final_expr {
                    self.compile_expr(e, chunk);
                } else {
                    chunk.emit(Op::PushNull);
                }
                self.end_scope(chunk);
            }
            Expr::If { cond, then_branch, else_branch, .. } => {
                self.compile_expr(cond, chunk);
                let jump_false = chunk.emit(Op::JumpIfFalse(0));
                chunk.emit(Op::Pop);
                self.compile_expr(then_branch, chunk);
                let jump_end = chunk.emit(Op::Jump(0));
                chunk.patch(jump_false, Op::JumpIfFalse(chunk.ops.len()));
                chunk.emit(Op::Pop);
                if let Some(e) = else_branch {
                    self.compile_expr(e, chunk);
                } else {
                    chunk.emit(Op::PushNull);
                }
                chunk.patch(jump_end, Op::Jump(chunk.ops.len()));
            }
            Expr::Lambda { .. } => {
                chunk.emit(Op::PushNull);
            }
            Expr::Index { target, index, .. } => {
                self.compile_expr(target, chunk);
                self.compile_expr(index, chunk);
                chunk.emit(Op::IndexGet);
            }
            Expr::Member { target, .. } => {
                self.compile_expr(target, chunk);
                chunk.emit(Op::PushNull);
            }
            Expr::Match { scrutinee, arms, .. } => {
                self.compile_expr(scrutinee, chunk);
                if let Some((_, body)) = arms.first() {
                    self.compile_expr(body, chunk);
                } else {
                    chunk.emit(Op::PushNull);
                }
            }
            Expr::Tuple { elems, .. } => {
                for e in elems {
                    self.compile_expr(e, chunk);
                }
                chunk.emit(Op::MakeTuple(elems.len()));
            }
            Expr::Array { elems, .. } => {
                for e in elems {
                    self.compile_expr(e, chunk);
                }
                chunk.emit(Op::MakeArray(elems.len()));
            }
            Expr::StructInit { fields, .. } => {
                for (_, val) in fields {
                    self.compile_expr(val, chunk);
                }
                chunk.emit(Op::MakeStruct { field_count: fields.len() });
            }
            Expr::Range { start, end, .. } => {
                self.compile_expr(start, chunk);
                self.compile_expr(end, chunk);
                chunk.emit(Op::MakeRange);
            }
            Expr::Placeholder(_) => {
                chunk.emit(Op::PushNull);
            },
        }
    }
}

impl Default for CodeGen {
    fn default() -> Self {
        Self::new()
    }
}
