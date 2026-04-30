# LuwiScript Project Knowledge Base

## Project Overview
LuwiScript is a programming language with a compiler (CLI: `luwic`) and a standalone runtime crate (`luwi-runtime`).
The compiler parses `.lw` files, type-checks, and emits bytecode (`.lwb`).
The runtime executes the bytecode via a stack-based VM.

## Architecture

### Workspace Structure
```
luwiscript-src/
├── compiler/         # luwi-script crate (lib + bin: luwic)
│   └── src/
│       ├── ast/      # AST types, serializer, pattern
│       ├── codegen/  # bytecode.rs — CodeGen (re-exports Op/Chunk/Constant from runtime)
│       ├── diagnostics/ # Error reporting with codespan
│       ├── driver/   # driver.rs — CLI, compilation pipeline, bytecode serialization
│       ├── lexer/    # Lexer, Token types
│       ├── parser/   # Recursive-descent parser
│       └── typechecker/ # Type checker with diagnostics
├── runtime/          # luwi-runtime crate
│   └── src/
│       ├── chunk.rs  # Op enum, Constant enum, Chunk struct (canonical bytecode types)
│       ├── value.rs  # Value enum (Int, Float, Bool, String, Null, Array, Tuple, Struct, EnumVariant, Fn)
│       ├── vm.rs     # Vm struct — stack-based execution engine
│       ├── scheduler.rs # Coroutine scheduler stub
│       ├── stdlib.rs # Standard library stub
│       └── lib.rs    # Re-exports
├── toolchain/
│   ├── fmt/          # luwfmt — formatter
│   └── linter/       # luwlinter — linter
└── examples/         # .lw example files
```

### Key Design Decisions
- **Canonical bytecode types live in `luwi-runtime`**: `Op`, `Constant`, `Chunk` are defined in `runtime/src/chunk.rs`. The compiler's `codegen/bytecode.rs` re-exports them via `pub use luwi_runtime::{Chunk, Constant, Op};`.
- **VM in runtime**: The `Vm` struct in `runtime/src/vm.rs` is the sole bytecode executor. The compiler's `driver.rs` creates a `Vm` instance for `--run` mode (the old inline VM has been removed).
- **Value::Struct(HashMap<String, Value>)**: Structs are represented as `HashMap<String, Value>` in the VM. This supports dynamic field access and mutation.

## Build Commands
```bash
cargo build                  # Build entire workspace
cargo build -p luwi-runtime  # Build only runtime
cargo run --bin luwic -- <file.lw> --run --no-typecheck  # Compile & run
cargo test --workspace       # Run all tests
cargo clippy --workspace     # Lint
```

## Runtime VM Details

### Op Dispatch (Struct-related)
- `Op::MakeStruct { name_idx, field_count, field_name_indices }` — pops `field_count` values from the stack, maps them to field names via `field_name_indices` (indices into `chunk.constants`), pushes `Value::Struct(HashMap)`.
- `Op::MemberGet(const_idx)` — pops the target value, resolves field name from `chunk.constants[const_idx]`, pushes the field value. Works on `Value::Struct`, `Value::String` (`.len`), `Value::Array` (`.len`).
- `Op::MemberSet(const_idx)` — pops value and target, updates the struct's HashMap, pushes the mutated struct (functional update pattern).

### Error Handling
- `MemberGet` on a struct with a missing field → runtime error "no field 'X' on struct"
- `MemberGet` on unsupported types → runtime error "member access not supported"
- `MemberSet` on non-struct → runtime error "member set not supported on type"

## Testing
- Runtime tests are in `runtime/src/vm_test.rs` — 11 tests covering MakeStruct, MemberGet, MemberSet, built-in member access, and Value traits.
- No compiler-level tests yet. Integration testing is via example files.

## Known Limitations
- `scheduler.rs` and `stdlib.rs` are stubs only.
- `ForMember` (immutable member assignment like `p.x = 31`) relies on `MemberSet` returning the mutated struct, which the codegen then stores back to the local. This is a functional-update pattern.
- Enums are not yet implemented in the runtime (`Value::EnumVariant` exists but no opcode creates it).
- `Value::Fn` exists but closure/lambda support is not implemented.
- Type checker has limited support for struct types.