use std::fs;
use std::path::PathBuf;

use crate::ast::serialize;
use crate::codegen::bytecode::{Chunk, CodeGen, Op};
use crate::diagnostics::report::report_diagnostics;
use crate::lexer::lexer::Lexer;
use crate::lexer::token::TokenKind;
use crate::parser::parser::Parser;
use crate::typechecker::checker::TypeChecker;

#[derive(Debug, Clone)]
pub struct Config {
    pub input: PathBuf,
    pub output: Option<PathBuf>,
    pub emit_ast: bool,
    pub emit_bytecode: bool,
    pub run: bool,
    pub no_typecheck: bool,
}

pub fn run(args: Vec<String>) -> Result<(), String> {
    let config = parse_args(&args)?;

    let source = fs::read_to_string(&config.input)
        .map_err(|e| format!("error reading '{}': {}", config.input.display(), e))?;

    let file_name = config.input.to_string_lossy().to_string();

    let tokens = {
        let mut lexer = Lexer::new(&source, 0);
        lexer.tokenize()
    };

    if tokens.iter().any(|t| t.kind == TokenKind::Error) {
        for t in &tokens {
            if t.kind == TokenKind::Error {
                eprintln!("\x1b[31merror\x1b[0m: invalid token '{}'", t.lexeme);
            }
        }
        return Err("lexer errors".into());
    }

    let stmts = {
        let mut parser = Parser::new(tokens);
        parser.parse().map_err(|e| {
            eprintln!("\x1b[31merror\x1b[0m: {} at {}", e.message, e.span);
            "parse error".to_string()
        })?
    };

    if config.emit_ast {
        for stmt in &stmts {
            println!("{}", serialize::serialize_stmt(stmt));
        }
    }

    if !config.no_typecheck {
        let mut tc = TypeChecker::new();
        if let Err(diags) = tc.check(&stmts) {
            report_diagnostics(&diags, &source, &file_name);
            return Err("typecheck errors".into());
        }
    }

    let chunks = {
        let mut codegen = CodeGen::new();
        codegen.compile(&stmts)
    };

    if config.emit_bytecode {
        for chunk in &chunks {
            disassemble_chunk(chunk);
        }
    }

    let output_path = config.output.clone().unwrap_or_else(|| {
        let stem = config.input.file_stem().unwrap().to_string_lossy().to_string();
        config.input.with_file_name(format!("{stem}.lwb"))
    });

    let bytes = serialize_bytecode(&chunks);
    fs::write(&output_path, &bytes)
        .map_err(|e| format!("error writing '{}': {}", output_path.display(), e))?;

    eprintln!("\x1b[32mCompiled\x1b[0m {} -> {}", config.input.display(), output_path.display());

    if config.run {
        run_bytecode(&chunks);
    }

    Ok(())
}

fn parse_args(args: &[String]) -> Result<Config, String> {
    let mut config = Config {
        input: PathBuf::new(),
        output: None,
        emit_ast: false,
        emit_bytecode: false,
        run: false,
        no_typecheck: false,
    };
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" => {
                i += 1;
                if i >= args.len() {
                    return Err("expected output path after -o".into());
                }
                config.output = Some(PathBuf::from(&args[i]));
            }
            "--emit-ast" => config.emit_ast = true,
            "--emit-bytecode" => config.emit_bytecode = true,
            "--run" | "-r" => config.run = true,
            "--no-typecheck" => config.no_typecheck = true,
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            p if p.starts_with('-') => {
                return Err(format!("unknown option '{}'", p));
            }
            _ => {
                if config.input.as_os_str().is_empty() {
                    config.input = PathBuf::from(&args[i]);
                } else {
                    return Err(format!("unexpected argument '{}'", args[i]));
                }
            }
        }
        i += 1;
    }
    if config.input.as_os_str().is_empty() {
        return Err("no input file specified".into());
    }
    Ok(config)
}

fn print_help() {
    eprintln!("luwic - LuwiScript compiler");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  luwic <input.lw> [options]");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  -o <path>          Output file path");
    eprintln!("  --emit-ast         Print AST to stdout");
    eprintln!("  --emit-bytecode    Disassemble bytecode to stdout");
    eprintln!("  -r, --run          Run after compilation");
    eprintln!("  --no-typecheck     Skip type checking");
    eprintln!("  -h, --help         Show this help");
}

fn disassemble_chunk(chunk: &Chunk) {
    eprintln!("== {} ==", chunk.name);
    for (i, op) in chunk.ops.iter().enumerate() {
        eprintln!("{i:04} {:?}", op);
    }
}

fn serialize_bytecode(chunks: &[Chunk]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"LWBC");
    bytes.extend_from_slice(&(chunks.len() as u32).to_le_bytes());
    for chunk in chunks {
        let name_bytes = chunk.name.as_bytes();
        bytes.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(name_bytes);
        bytes.extend_from_slice(&(chunk.ops.len() as u32).to_le_bytes());
        for op in &chunk.ops {
            serialize_op(op, &mut bytes);
        }
    }
    bytes
}

fn serialize_op(op: &Op, bytes: &mut Vec<u8>) {
    match op {
        Op::PushNull => bytes.push(0x00),
        Op::PushInt(n) => { bytes.push(0x01); bytes.extend_from_slice(&n.to_le_bytes()); }
        Op::PushFloat(n) => { bytes.push(0x02); bytes.extend_from_slice(&n.to_le_bytes()); }
        Op::PushBool(b) => { bytes.push(0x03); bytes.push(if *b { 1 } else { 0 }); }
        Op::PushString(s) => {
            bytes.push(0x04);
            let sb = s.as_bytes();
            bytes.extend_from_slice(&(sb.len() as u32).to_le_bytes());
            bytes.extend_from_slice(sb);
        }
        Op::LoadLocal(idx) => { bytes.push(0x10); bytes.extend_from_slice(&(*idx as u32).to_le_bytes()); }
        Op::StoreLocal(idx) => { bytes.push(0x11); bytes.extend_from_slice(&(*idx as u32).to_le_bytes()); }
        Op::LoadGlobal(idx) => { bytes.push(0x12); bytes.extend_from_slice(&(*idx as u32).to_le_bytes()); }
        Op::StoreGlobal(idx) => { bytes.push(0x13); bytes.extend_from_slice(&(*idx as u32).to_le_bytes()); }
        Op::Add => bytes.push(0x20),
        Op::Sub => bytes.push(0x21),
        Op::Mul => bytes.push(0x22),
        Op::Div => bytes.push(0x23),
        Op::Rem => bytes.push(0x24),
        Op::Eq => bytes.push(0x25),
        Op::Neq => bytes.push(0x26),
        Op::Lt => bytes.push(0x27),
        Op::Le => bytes.push(0x28),
        Op::Gt => bytes.push(0x29),
        Op::Ge => bytes.push(0x2A),
        Op::And => bytes.push(0x2B),
        Op::Or => bytes.push(0x2C),
        Op::Neg => bytes.push(0x2D),
        Op::Not => bytes.push(0x2E),
        Op::Jump(addr) => { bytes.push(0x30); bytes.extend_from_slice(&(*addr as u32).to_le_bytes()); }
        Op::JumpIfFalse(addr) => { bytes.push(0x31); bytes.extend_from_slice(&(*addr as u32).to_le_bytes()); }
        Op::JumpIfTrue(addr) => { bytes.push(0x32); bytes.extend_from_slice(&(*addr as u32).to_le_bytes()); }
            Op::Call { chunk_idx, argc } => { bytes.push(0x40); bytes.extend_from_slice(&(*chunk_idx as u32).to_le_bytes()); bytes.extend_from_slice(&(*argc as u32).to_le_bytes()); }
        Op::Return => bytes.push(0x50),
        Op::Print => bytes.push(0x60),
        Op::Println => bytes.push(0x61),
        Op::Len => bytes.push(0x62),
        Op::Halt => bytes.push(0xFF),
        Op::Pop => bytes.push(0x70),
        Op::Dup => bytes.push(0x71),
        Op::IndexGet => bytes.push(0x80),
        Op::IndexSet => bytes.push(0x81),
        Op::MemberGet(idx) => { bytes.push(0x82); bytes.extend_from_slice(&(*idx as u32).to_le_bytes()); }
        Op::MakeArray(n) => { bytes.push(0x90); bytes.extend_from_slice(&(*n as u32).to_le_bytes()); }
        Op::MakeTuple(n) => { bytes.push(0x91); bytes.extend_from_slice(&(*n as u32).to_le_bytes()); }
            Op::MakeStruct { field_count } => { bytes.push(0x92); bytes.extend_from_slice(&(*field_count as u32).to_le_bytes()); }
            Op::MakeRange => { bytes.push(0x93); }
    }
}

fn run_bytecode(chunks: &[Chunk]) {
    if chunks.is_empty() {
        return;
    }
    let mut vm = Vm::new(chunks);
    if let Err(e) = vm.run() {
        eprintln!("\x1b[31mruntime error\x1b[0m: {e}");
    }
}

struct Vm<'a> {
    chunks: &'a [Chunk],
    stack: Vec<Value>,
    locals: Vec<Value>,
    call_stack: Vec<CallFrame>,
    ip: usize,
    chunk_idx: usize,
}

#[derive(Debug, Clone)]
struct CallFrame {
    chunk_idx: usize,
    ip: usize,
    locals: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq)]
enum Value {
    Null,
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Array(Vec<Value>),
    Tuple(Vec<Value>),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Int(n) => write!(f, "{n}"),
            Value::Float(n) => write!(f, "{n}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::String(s) => write!(f, "{s}"),
            Value::Array(elems) => {
                write!(f, "[")?;
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{e}")?;
                }
                write!(f, "]")
            }
            Value::Tuple(elems) => {
                write!(f, "(")?;
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{e}")?;
                }
                write!(f, ")")
            }
        }
    }
}

impl<'a> Vm<'a> {
    fn new(chunks: &'a [Chunk]) -> Self {
        Vm {
            chunks,
            stack: Vec::new(),
            locals: Vec::new(),
            call_stack: Vec::new(),
            ip: 0,
            chunk_idx: 0,
        }
    }

    fn run(&mut self) -> Result<(), String> {
        loop {
            let ops = self.chunks[self.chunk_idx].ops.clone();
            if self.ip >= ops.len() {
                return Ok(());
            }
            let op = ops[self.ip].clone();
            self.ip += 1;

            match op {
                Op::PushNull => self.stack.push(Value::Null),
                Op::PushInt(n) => self.stack.push(Value::Int(n)),
                Op::PushFloat(n) => self.stack.push(Value::Float(n)),
                Op::PushBool(b) => self.stack.push(Value::Bool(b)),
                Op::PushString(s) => self.stack.push(Value::String(s)),
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
                Op::Halt => return Ok(()),
                Op::Pop => { self.stack.pop(); }
                Op::Dup => {
                    if let Some(val) = self.stack.last() {
                        self.stack.push(val.clone());
                    }
                }
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
                Op::IndexSet => {}
                Op::MemberGet(_) => {}
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
                Op::MakeStruct { .. } => {}
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
            (Value::Float(x), Value::Int(y)) => self.stack.push(Value::Float(float_fn(*x, *y as f64))),
            (Value::Int(x), Value::Float(y)) => self.stack.push(Value::Float(float_fn(*x as f64, *y))),
            (Value::String(x), Value::String(y)) if matches!(int_fn(0, 0), 0) => {
                self.stack.push(Value::String(format!("{x}{y}")));
            }
            _ => return Err(format!("cannot apply op to {} and {}", a, b)),
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

impl Value {
    fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Int(n) => *n != 0,
            Value::Float(n) => *n != 0.0,
            Value::Bool(b) => *b,
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.is_empty(),
            Value::Tuple(t) => !t.is_empty(),
        }
    }
}
