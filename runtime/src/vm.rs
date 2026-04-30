use std::collections::HashMap;

use crate::chunk::{Chunk, Constant, Op};
use crate::value::Value;

/// A saved call frame so we can return from functions.
#[derive(Debug, Clone)]
struct CallFrame {
    chunk_idx: usize,
    ip: usize,
    locals: Vec<Value>,
}

/// The LuwiScript virtual machine.
///
/// Executes a set of [`Chunk`]s produced by the compiler's codegen
/// phase. The VM is stack-based with a separate locals vector and a
/// call stack for function invocations.
pub struct Vm {
    chunks: Vec<Chunk>,
    pub(crate) stack: Vec<Value>,
    locals: Vec<Value>,
    call_stack: Vec<CallFrame>,
    ip: usize,
    chunk_idx: usize,
}

impl Vm {
    pub fn new(chunks: Vec<Chunk>) -> Self {
        Vm {
            chunks,
            stack: Vec::new(),
            locals: Vec::new(),
            call_stack: Vec::new(),
            ip: 0,
            chunk_idx: 0,
        }
    }

    /// Run the main chunk (index 0) to completion.
    pub fn run(&mut self) -> Result<(), String> {
        if self.chunks.is_empty() {
            return Ok(());
        }
        loop {
            let ops = self.chunks[self.chunk_idx].ops.clone();
            if self.ip >= ops.len() {
                return Ok(());
            }
            let op = ops[self.ip].clone();
            self.ip += 1;

            match op {
                // ── Push literals ───────────────────────────────
                Op::PushNull => self.stack.push(Value::Null),
                Op::PushInt(n) => self.stack.push(Value::Int(n)),
                Op::PushFloat(n) => self.stack.push(Value::Float(n)),
                Op::PushBool(b) => self.stack.push(Value::Bool(b)),
                Op::PushString(s) => self.stack.push(Value::String(s)),

                // ── Variables ───────────────────────────────────
                Op::LoadLocal(idx) => {
                    if idx < self.locals.len() {
                        self.stack.push(self.locals[idx].clone());
                    } else {
                        self.stack.push(Value::Null);
                    }
                }
                Op::StoreLocal(idx) => {
                    let val = self.stack.pop().ok_or("stack underflow")?;
                    if idx < self.locals.len() {
                        self.locals[idx] = val;
                    } else {
                        self.locals.resize(idx + 1, Value::Null);
                        self.locals[idx] = val;
                    }
                }
                Op::LoadGlobal(_) | Op::StoreGlobal(_) => {}

                // ── Arithmetic / comparison ─────────────────────
                Op::Add => self.binary_int_op(|a, b| a + b, |a, b| a + b)?,
                Op::Sub => self.binary_int_op(|a, b| a - b, |a, b| a - b)?,
                Op::Mul => self.binary_int_op(|a, b| a * b, |a, b| a * b)?,
                Op::Div => self.binary_int_op(|a, b| a / b, |a, b| a / b)?,
                Op::Rem => self.binary_int_op(|a, b| a % b, |a, b| a % b)?,
                Op::Eq => {
                    let (a, b) = self.pop_two()?;
                    self.stack.push(Value::Bool(a == b));
                }
                Op::Neq => {
                    let (a, b) = self.pop_two()?;
                    self.stack.push(Value::Bool(a != b));
                }
                Op::Lt => self.cmp_op(|a, b| a < b)?,
                Op::Le => self.cmp_op(|a, b| a <= b)?,
                Op::Gt => self.cmp_op(|a, b| a > b)?,
                Op::Ge => self.cmp_op(|a, b| a >= b)?,
                Op::And => {
                    let (a, b) = self.pop_two()?;
                    self.stack.push(Value::Bool(a.is_truthy() && b.is_truthy()));
                }
                Op::Or => {
                    let (a, b) = self.pop_two()?;
                    self.stack.push(Value::Bool(a.is_truthy() || b.is_truthy()));
                }
                Op::Neg => {
                    let val = self.stack.pop().ok_or("stack underflow")?;
                    match val {
                        Value::Int(n) => self.stack.push(Value::Int(-n)),
                        Value::Float(n) => self.stack.push(Value::Float(-n)),
                        _ => return Err("cannot negate non-numeric".into()),
                    }
                }
                Op::Not => {
                    let val = self.stack.pop().ok_or("stack underflow")?;
                    self.stack.push(Value::Bool(!val.is_truthy()));
                }

                // ── Control flow ───────────────────────────────
                Op::Jump(addr) => self.ip = addr,
                Op::JumpIfFalse(addr) => {
                    let val = self.stack.pop().ok_or("stack underflow")?;
                    if !val.is_truthy() { self.ip = addr; }
                }
                Op::JumpIfTrue(addr) => {
                    let val = self.stack.pop().ok_or("stack underflow")?;
                    if val.is_truthy() { self.ip = addr; }
                }
                Op::Call { chunk_idx, argc } => {
                    let mut args = Vec::new();
                    for _ in 0..argc {
                        args.push(self.stack.pop().ok_or("stack underflow")?);
                    }
                    args.reverse();
                    let frame = CallFrame {
                        chunk_idx: self.chunk_idx,
                        ip: self.ip,
                        locals: std::mem::take(&mut self.locals),
                    };
                    self.call_stack.push(frame);
                    self.chunk_idx = chunk_idx;
                    self.locals = args;
                    self.ip = 0;
                }
                Op::Return => {
                    let ret_val = self.stack.pop().unwrap_or(Value::Null);
                    if let Some(frame) = self.call_stack.pop() {
                        self.chunk_idx = frame.chunk_idx;
                        self.ip = frame.ip;
                        self.locals = frame.locals;
                        self.stack.push(ret_val);
                    } else {
                        self.stack.push(ret_val);
                        return Ok(());
                    }
                }

                // ── I/O ────────────────────────────────────────
                Op::Print => {
                    let val = self.stack.pop().ok_or("stack underflow")?;
                    print!("{val}");
                }
                Op::Println => {
                    let val = self.stack.pop().ok_or("stack underflow")?;
                    println!("{val}");
                }
                Op::Len => {
                    let val = self.stack.pop().ok_or("stack underflow")?;
                    match val {
                        Value::Array(elems) => self.stack.push(Value::Int(elems.len() as i64)),
                        Value::String(s) => self.stack.push(Value::Int(s.len() as i64)),
                        _ => return Err("len() requires array or string".into()),
                    }
                }

                // ── Stack manipulation ──────────────────────────
                Op::Halt => return Ok(()),
                Op::Pop => { self.stack.pop(); }
                Op::Dup => {
                    if let Some(val) = self.stack.last() {
                        self.stack.push(val.clone());
                    }
                }

                // ── Indexing ───────────────────────────────────
                Op::IndexGet => {
                    let idx = self.stack.pop().ok_or("stack underflow")?;
                    let target = self.stack.pop().ok_or("stack underflow")?;
                    match (&target, &idx) {
                        (Value::Array(elems), Value::Int(i)) => {
                            let i = *i as usize;
                            if i < elems.len() {
                                self.stack.push(elems[i].clone());
                            } else {
                                return Err(format!("index {i} out of bounds"));
                            }
                        }
                        (Value::String(s), Value::Int(i)) => {
                            let i = *i as usize;
                            if i < s.len() {
                                self.stack.push(Value::String(s[i..i + 1].to_string()));
                            } else {
                                return Err(format!("index {i} out of bounds"));
                            }
                        }
                        _ => return Err("invalid index operation".into()),
                    }
                }
                Op::IndexSet => {
                    let val = self.stack.pop().ok_or("stack underflow")?;
                    let idx = self.stack.pop().ok_or("stack underflow")?;
                    let mut target = self.stack.pop().ok_or("stack underflow")?;
                    match (&mut target, &idx) {
                        (Value::Array(elems), Value::Int(i)) => {
                            let i = *i as usize;
                            if i < elems.len() {
                                elems[i] = val;
                                self.stack.push(target);
                            } else {
                                return Err(format!("index {i} out of bounds for array assignment"));
                            }
                        }
                        _ => return Err("invalid index set operation".into()),
                    }
                }

                // ── Struct / member access ──────────────────────
                Op::MemberGet(idx) => {
                    let target = self.stack.pop().ok_or("stack underflow")?;
                    match &target {
                        Value::Struct(fields) => {
                            if let Some(key) = self.chunks[self.chunk_idx].constants.get(idx) {
                                if let Constant::String(field_name) = key {
                                    if let Some(val) = fields.get(field_name) {
                                        self.stack.push(val.clone());
                                    } else {
                                        return Err(format!("struct has no field '{field_name}'"));
                                    }
                                } else {
                                    return Err("MemberGet constant is not a string".into());
                                }
                            } else {
                                return Err(format!("MemberGet constant index {idx} out of bounds"));
                            }
                        }
                        Value::String(s) => {
                            if let Some(key) = self.chunks[self.chunk_idx].constants.get(idx) {
                                if let Constant::String(method_name) = key {
                                    match method_name.as_str() {
                                        "len" => {
                                            self.stack.push(Value::Int(s.len() as i64));
                                        }
                                        _ => return Err(format!("string has no method '{method_name}'")),
                                    }
                                } else {
                                    return Err("MemberGet constant is not a string".into());
                                }
                            } else {
                                return Err(format!("MemberGet constant index {idx} out of bounds"));
                            }
                        }
                        Value::Array(arr) => {
                            if let Some(key) = self.chunks[self.chunk_idx].constants.get(idx) {
                                if let Constant::String(method_name) = key {
                                    match method_name.as_str() {
                                        "len" => {
                                            self.stack.push(Value::Int(arr.len() as i64));
                                        }
                                        _ => return Err(format!("array has no method '{method_name}'")),
                                    }
                                } else {
                                    return Err("MemberGet constant is not a string".into());
                                }
                            } else {
                                return Err(format!("MemberGet constant index {idx} out of bounds"));
                            }
                        }
                        _ => return Err(format!("cannot access field on {}", target.type_name())),
                    }
                }
                Op::MemberSet(idx) => {
                    let val = self.stack.pop().ok_or("stack underflow")?;
                    let mut target = self.stack.pop().ok_or("stack underflow")?;
                    match &mut target {
                        Value::Struct(fields) => {
                            if let Some(key) = self.chunks[self.chunk_idx].constants.get(idx) {
                                if let Constant::String(field_name) = key {
                                    // Allow setting both existing and new fields.
                                    // Existing-field-only restriction can be added
                                    // once struct definitions are tracked at runtime.
                                    fields.insert(field_name.clone(), val);
                                    self.stack.push(target);
                                } else {
                                    return Err("MemberSet constant is not a string".into());
                                }
                            } else {
                                return Err(format!("MemberSet constant index {idx} out of bounds"));
                            }
                        }
                        _ => return Err(format!("cannot set field on {}", target.type_name())),
                    }
                }

                // ── Composite constructors ──────────────────────
                Op::MakeArray(n) => {
                    let start = self.stack.len().saturating_sub(n);
                    let elems: Vec<Value> = self.stack.drain(start..).collect();
                    self.stack.push(Value::Array(elems));
                }
                Op::MakeTuple(n) => {
                    let start = self.stack.len().saturating_sub(n);
                    let elems: Vec<Value> = self.stack.drain(start..).collect();
                    self.stack.push(Value::Tuple(elems));
                }
                Op::MakeStruct { name_idx: _, field_count, field_name_indices } => {
                    let mut field_names = Vec::with_capacity(field_count);
                    for &fi in &field_name_indices {
                        if let Some(c) = self.chunks[self.chunk_idx].constants.get(fi) {
                            if let Constant::String(s) = c {
                                field_names.push(s.clone());
                            } else {
                                return Err("MakeStruct field constant is not a string".into());
                            }
                        } else {
                            return Err(format!("MakeStruct field constant index {fi} out of bounds"));
                        }
                    }
                    let start = self.stack.len().saturating_sub(field_count);
                    let vals: Vec<Value> = self.stack.drain(start..).collect();
                    let mut fields = HashMap::new();
                    for (i, val) in vals.into_iter().enumerate() {
                        if i < field_names.len() {
                            fields.insert(field_names[i].clone(), val);
                        }
                    }
                    self.stack.push(Value::Struct(fields));
                }
                Op::MakeRange => {
                    let end = self.stack.pop().ok_or("stack underflow")?;
                    let start = self.stack.pop().ok_or("stack underflow")?;
                    let (s, e) = match (start, end) {
                        (Value::Int(s), Value::Int(e)) => (s, e),
                        _ => return Err("range bounds must be int".into()),
                    };
                    let elems: Vec<Value> = (s..e).map(Value::Int).collect();
                    self.stack.push(Value::Array(elems));
                }
            }
        }
    }

    // ── Helpers ────────────────────────────────────────────────

    fn pop_two(&mut self) -> Result<(Value, Value), String> {
        let b = self.stack.pop().ok_or("stack underflow")?;
        let a = self.stack.pop().ok_or("stack underflow")?;
        Ok((a, b))
    }

    fn binary_int_op<F1, F2>(&mut self, int_fn: F1, float_fn: F2) -> Result<(), String>
    where
        F1: Fn(i64, i64) -> i64,
        F2: Fn(f64, f64) -> f64,
    {
        let (a, b) = self.pop_two()?;
        match (&a, &b) {
            (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Int(int_fn(*x, *y))),
            (Value::Float(x), Value::Float(y)) => self.stack.push(Value::Float(float_fn(*x, *y))),
            (Value::Float(x), Value::Int(y)) => {
                self.stack.push(Value::Float(float_fn(*x, *y as f64)))
            }
            (Value::Int(x), Value::Float(y)) => {
                self.stack.push(Value::Float(float_fn(*x as f64, *y)))
            }
            (Value::String(x), Value::String(y)) if matches!(int_fn(0, 0), 0) => {
                self.stack.push(Value::String(format!("{x}{y}")));
            }
            _ => return Err(format!("cannot apply op to {} and {}", a.type_name(), b.type_name())),
        }
        Ok(())
    }

    fn cmp_op<F>(&mut self, f: F) -> Result<(), String>
    where
        F: Fn(i64, i64) -> bool,
    {
        let (a, b) = self.pop_two()?;
        match (&a, &b) {
            (Value::Int(x), Value::Int(y)) => self.stack.push(Value::Bool(f(*x, *y))),
            (Value::Float(x), Value::Float(y)) => {
                let result = if f(0, 1) {
                    *x > *y
                } else if f(1, 0) {
                    *x < *y
                } else if f(0, 0) {
                    *x <= *y
                } else {
                    *x >= *y
                };
                self.stack.push(Value::Bool(result));
            }
            _ => self.stack.push(Value::Bool(false)),
        }
        Ok(())
    }
}

impl Default for Vm {
    fn default() -> Self {
        Vm::new(Vec::new())
    }
}
