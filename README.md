# ASTER

**General-purpose programming language and compiler built from scratch in Rust.**

ASTER implements a visible source-to-execution pipeline: lexical analysis, parsing, AST construction, semantic analysis, bytecode generation, and execution on a stack-based virtual machine.

## Compilation Pipeline

```text
Source
  │
  ▼
Lexer → Tokens
  │
  ▼
Parser → AST
  │
  ▼
Semantic Analysis
  │
  ▼
Bytecode Compiler
  │
  ▼
Stack VM + Call Frames
  │
  ▼
Execution
```

The CLI exposes the intermediate representations so compilation is inspectable rather than a black box.

## Language Features

- Lexer and token model
- Recursive-descent parser with operator precedence
- Variables and assignments
- Explicit variable type annotations
- Blocks and conditionals
- Functions, parameters, calls, and returns
- Lexical scopes
- `number`, `string`, `bool`, `void`, and `unknown` types
- Duplicate declaration detection
- Undefined variable/function detection
- Function arity checking
- Operator and assignment type checking
- Non-boolean conditional validation

## Runtime

- Custom bytecode instruction set
- Bytecode compiler
- Stack-based virtual machine
- Global variables
- Local variable slots
- Function call frames
- Recursive function calls
- Conditional jumps
- Runtime values
- Bytecode disassembler

## Developer Experience

- Semantic `--check`
- `--dump-ast`
- `--dump-bytecode`
- Interactive `--repl`
- Structured compiler diagnostics
- Source locations in lexer/parser errors
- Regression tests across frontend, semantics, execution, and disassembly
- GitHub Actions CI

## Example

```text
let answer: number = 40 + 2;

if answer > 0 {
    print(answer);
}
```

Execution produces:

```text
42
```

## CLI

```bash
cargo run -- examples/hello.aster
cargo run -- --check examples/hello.aster
cargo run -- --dump-ast examples/hello.aster
cargo run -- --dump-bytecode examples/hello.aster
cargo run -- --repl
```

## Architecture

```text
AST
 │
 ├── semantic/type checking
 │
 ▼
Bytecode
 │
 ▼
Stack VM
 │
 ├── local slots
 ├── call frames
 └── recursive calls
```

ASTER uses Rust-managed values and call frames. The project therefore explores a stack-machine runtime model while relying on Rust for memory safety in the compiler/runtime implementation.

## Repository Structure

```text
aster/
├── src/
│   ├── ast.rs
│   ├── bytecode.rs
│   ├── disassembler.rs
│   ├── lexer.rs
│   ├── lib.rs
│   ├── main.rs
│   ├── parser.rs
│   ├── semantic.rs
│   ├── token.rs
│   ├── type_system.rs
│   └── vm.rs
├── examples/hello.aster
├── tests/
│   ├── disassembler.rs
│   ├── execution.rs
│   ├── frontend.rs
│   └── semantic.rs
├── docs/architecture.md
└── .github/workflows/ci.yml
```

## Getting Started

Prerequisite: a current stable Rust toolchain.

```bash
git clone https://github.com/Scarlet-Twinz/aster.git
cd aster
cargo build
cargo run -- examples/hello.aster
```

## Quality Gates

```bash
cargo fmt --all -- --check
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release --all-features
```

GitHub Actions runs the core formatting, Clippy, test, and release-build checks.

## Current Implementation

ASTER currently includes the lexer, parser, AST, semantic/type analysis, bytecode generation, stack VM, local variables, call frames, recursion, bytecode disassembly, CLI inspection, REPL, diagnostics, regression tests, and CI validation.

Future language work can include modules/imports, richer function types, closures, a larger standard library, and more advanced runtime facilities.

## Engineering Focus

ASTER is an implementation study in:

- language design;
- parsing and ASTs;
- lexical scoping and semantic validation;
- intermediate representation design;
- bytecode generation;
- virtual-machine execution;
- function frames and recursion;
- diagnostics and regression testing; and
- Rust systems programming.

## License

MIT

## Author

**Anthony Emmanuella Mmasinachi**

Full-stack and systems engineer focused on backend infrastructure, distributed systems, networking, compilers, databases, AI integration, and systems programming.
