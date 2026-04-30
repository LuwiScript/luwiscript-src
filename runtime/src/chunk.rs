/// Bytecode opcodes emitted by the compiler and executed by the VM.
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    // ── Push literals ──────────────────────────────────────────
    PushNull,
    PushInt(i64),
    PushFloat(f64),
    PushBool(bool),
    PushString(String),

    // ── Variables ──────────────────────────────────────────────
    LoadLocal(usize),
    StoreLocal(usize),
    LoadGlobal(usize),
    StoreGlobal(usize),

    // ── Arithmetic / comparison ────────────────────────────────
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

    // ── Control flow ───────────────────────────────────────────
    Jump(usize),
    JumpIfFalse(usize),
    JumpIfTrue(usize),
    Call { chunk_idx: usize, argc: usize },
    Return,

    // ── I/O ────────────────────────────────────────────────────
    Print,
    Println,
    Len,

    // ── Stack manipulation ─────────────────────────────────────
    Halt,
    Pop,
    Dup,

    // ── Indexing ───────────────────────────────────────────────
    IndexGet,
    IndexSet,

    // ── Struct / member access ─────────────────────────────────
    /// Push the value of field at constant-pool index `idx` from the
    /// struct on top of the stack.
    MemberGet(usize),
    /// Pop a value and a struct from the stack, set the field at
    /// constant-pool index `idx`, and push the modified struct back.
    MemberSet(usize),

    // ── Composite constructors ─────────────────────────────────
    MakeArray(usize),
    MakeTuple(usize),
    /// Pop `field_count` values from the stack and pair them with the
    /// field names identified by `field_name_indices` (constant-pool
    /// indices) to build a `Value::Struct`.
    MakeStruct {
        name_idx: usize,
        field_count: usize,
        field_name_indices: Vec<usize>,
    },
    MakeRange,
}

/// Compile-time constants stored in a chunk's constant pool.
#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Null,
}

/// A compiled bytecode chunk: a sequence of ops plus a constant pool.
#[derive(Debug, Clone)]
pub struct Chunk {
    pub ops: Vec<Op>,
    pub constants: Vec<Constant>,
    pub name: String,
}

impl Chunk {
    pub fn new(name: impl Into<String>) -> Self {
        Chunk {
            ops: Vec::new(),
            constants: Vec::new(),
            name: name.into(),
        }
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