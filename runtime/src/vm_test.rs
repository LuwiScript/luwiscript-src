use std::collections::HashMap;

use crate::chunk::{Chunk, Constant, Op};
use crate::value::Value;
use crate::vm::Vm;

/// Helper: build a minimal chunk with the given ops and constants.
fn test_chunk(ops: Vec<Op>, constants: Vec<Constant>) -> Vec<Chunk> {
    let mut c = Chunk::new("test");
    c.ops = ops;
    c.constants = constants;
    c.emit(Op::Halt);
    vec![c]
}

/// Helper: run a chunk and return the top-of-stack value.
fn run_chunk(chunks: Vec<Chunk>) -> Result<Value, String> {
    let mut vm = Vm::new(chunks);
    vm.run()?;
    vm.stack.pop().ok_or_else(|| "stack empty".into())
}

// ── MakeStruct ─────────────────────────────────────────────────

#[test]
fn make_struct_empty() {
    let chunks = test_chunk(
        vec![Op::MakeStruct {
            name_idx: 0,
            field_count: 0,
            field_name_indices: vec![],
        }],
        vec![Constant::String("Empty".into())],
    );
    let val = run_chunk(chunks).unwrap();
    match val {
        Value::Struct(fields) => assert!(fields.is_empty()),
        _ => panic!("expected Struct, got {:?}", val),
    }
}

#[test]
fn make_struct_with_fields() {
    let chunks = test_chunk(
        vec![
            Op::PushInt(42),
            Op::PushString("hello".into()),
            Op::MakeStruct {
                name_idx: 0,
                field_count: 2,
                field_name_indices: vec![1, 2],
            },
        ],
        vec![
            Constant::String("MyStruct".into()),
            Constant::String("x".into()),
            Constant::String("label".into()),
        ],
    );
    let val = run_chunk(chunks).unwrap();
    match val {
        Value::Struct(fields) => {
            assert_eq!(fields.get("x"), Some(&Value::Int(42)));
            assert_eq!(fields.get("label"), Some(&Value::String("hello".into())));
        }
        _ => panic!("expected Struct, got {:?}", val),
    }
}

// ── MemberGet ──────────────────────────────────────────────────

#[test]
fn member_get_on_struct() {
    let chunks = test_chunk(
        vec![
            Op::PushInt(10),
            Op::PushInt(20),
            Op::MakeStruct {
                name_idx: 0,
                field_count: 2,
                field_name_indices: vec![1, 2],
            },
            Op::MemberGet(2), // field "y"
        ],
        vec![
            Constant::String("Point".into()),
            Constant::String("x".into()),
            Constant::String("y".into()),
        ],
    );
    let val = run_chunk(chunks).unwrap();
    assert_eq!(val, Value::Int(20));
}

#[test]
fn member_get_missing_field_errors() {
    let chunks = test_chunk(
        vec![
            Op::PushInt(10),
            Op::MakeStruct {
                name_idx: 0,
                field_count: 1,
                field_name_indices: vec![1],
            },
            Op::MemberGet(2), // constant index 2 = "z", not a field
        ],
        vec![
            Constant::String("Point".into()),
            Constant::String("x".into()),
            Constant::String("z".into()),
        ],
    );
    let result = run_chunk(chunks);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("no field"));
}

// ── MemberSet ──────────────────────────────────────────────────

#[test]
fn member_set_on_struct() {
    let chunks = test_chunk(
        vec![
            Op::PushInt(1),
            Op::PushInt(2),
            Op::MakeStruct {
                name_idx: 0,
                field_count: 2,
                field_name_indices: vec![1, 2],
            },
            // Set field "x" to 99
            Op::PushInt(99),
            Op::MemberSet(1), // field "x"
            // Now read field "x" back
            Op::MemberGet(1), // field "x"
        ],
        vec![
            Constant::String("Point".into()),
            Constant::String("x".into()),
            Constant::String("y".into()),
        ],
    );
    let val = run_chunk(chunks).unwrap();
    assert_eq!(val, Value::Int(99));
}

// ── MemberGet on built-in types ────────────────────────────────

#[test]
fn member_get_string_len() {
    let chunks = test_chunk(
        vec![
            Op::PushString("hello".into()),
            Op::MemberGet(0), // "len"
        ],
        vec![Constant::String("len".into())],
    );
    let val = run_chunk(chunks).unwrap();
    assert_eq!(val, Value::Int(5));
}

#[test]
fn member_get_array_len() {
    let chunks = test_chunk(
        vec![
            Op::PushInt(1),
            Op::PushInt(2),
            Op::PushInt(3),
            Op::MakeArray(3),
            Op::MemberGet(0), // "len"
        ],
        vec![Constant::String("len".into())],
    );
    let val = run_chunk(chunks).unwrap();
    assert_eq!(val, Value::Int(3));
}

// ── Value traits ───────────────────────────────────────────────

#[test]
fn value_struct_display() {
    let mut fields = HashMap::new();
    fields.insert("x".into(), Value::Int(1));
    fields.insert("y".into(), Value::Int(2));
    let v = Value::Struct(fields);
    let s = format!("{v}");
    assert!(s.contains("x: 1"));
    assert!(s.contains("y: 2"));
}

#[test]
fn value_struct_is_truthy() {
    let v = Value::Struct(HashMap::new());
    assert!(v.is_truthy());
}

#[test]
fn value_struct_equality() {
    let mut a = HashMap::new();
    a.insert("x".into(), Value::Int(1));
    let mut b = HashMap::new();
    b.insert("x".into(), Value::Int(1));
    assert_eq!(Value::Struct(a), Value::Struct(b.clone()));

    let mut c = HashMap::new();
    c.insert("x".into(), Value::Int(2));
    assert_ne!(Value::Struct(c), Value::Struct(b));
}

#[test]
fn value_struct_type_name() {
    let v = Value::Struct(HashMap::new());
    assert_eq!(v.type_name(), "struct");
}