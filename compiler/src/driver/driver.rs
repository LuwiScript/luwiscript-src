use std::fs;
use std::path::PathBuf;

use crate::ast::serialize;
use crate::codegen::bytecode::{Chunk, CodeGen, Op};
use crate::diagnostics::report::report_diagnostics;
use crate::lexer::lexer::Lexer;
use crate::lexer::token::TokenKind;
use crate::parser::parser::Parser;
use crate::typechecker::checker::TypeChecker;
use luwi_runtime::Vm;

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
        Op::Call { chunk_idx, argc } => {
            bytes.push(0x40);
            bytes.extend_from_slice(&(*chunk_idx as u32).to_le_bytes());
            bytes.extend_from_slice(&(*argc as u32).to_le_bytes());
        }
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
        Op::MemberSet(idx) => { bytes.push(0x83); bytes.extend_from_slice(&(*idx as u32).to_le_bytes()); }
        Op::MakeArray(n) => { bytes.push(0x90); bytes.extend_from_slice(&(*n as u32).to_le_bytes()); }
        Op::MakeTuple(n) => { bytes.push(0x91); bytes.extend_from_slice(&(*n as u32).to_le_bytes()); }
        Op::MakeStruct { name_idx, field_count, field_name_indices } => {
            bytes.push(0x92);
            bytes.extend_from_slice(&(*name_idx as u32).to_le_bytes());
            bytes.extend_from_slice(&(*field_count as u32).to_le_bytes());
            bytes.extend_from_slice(&(field_name_indices.len() as u32).to_le_bytes());
            for fi in field_name_indices {
                bytes.extend_from_slice(&(*fi as u32).to_le_bytes());
            }
        }
        Op::MakeRange => { bytes.push(0x93); }
    }
}

fn run_bytecode(chunks: &[Chunk]) {
    let mut vm = Vm::new(chunks.to_vec());
    if let Err(e) = vm.run() {
        eprintln!("\x1b[31mruntime error\x1b[0m: {e}");
    }
}
