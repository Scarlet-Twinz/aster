# ASTER

**ASTER** is a general-purpose programming language and compiler built from scratch in Rust.

The project is being developed as a real compiler implementation rather than a syntax-only demo:

`source → lexer → parser → AST → semantic analysis → bytecode → virtual machine`

## Current status

### Phase 1 — Language foundation

- [x] Rust project structure
- [x] Token model
- [x] Lexer
- [x] Abstract syntax tree
- [x] Recursive-descent parser
- [x] Expressions and operator precedence
- [x] Variable declarations
- [x] Assignments
- [x] Blocks and conditional statements
- [x] Function declarations and calls
- [x] Parser regression tests

### Phase 2 — Semantics

- [x] Type model (`number`, `string`, `bool`, `void`, `unknown`)
- [x] Lexical scope tracking
- [x] Symbol lookup
- [x] Duplicate declaration detection
- [x] Undefined variable detection
- [x] Function arity checking
- [x] Operator type checking
- [x] Semantic diagnostics
- [x] Semantic regression tests
- [ ] Explicit type annotations
- [ ] Return-type inference and checking
- [ ] Modules and imports

### Phase 3 — Execution

- [ ] Bytecode instruction set
- [ ] Bytecode compiler
- [ ] Virtual machine
- [ ] Runtime values
- [ ] Function call frames
- [ ] Memory management

### Phase 4 — Developer experience

- [x] Basic compiler CLI
- [x] Semantic `--check` mode
- [ ] REPL
- [ ] Standard library
- [ ] Rich source spans and diagnostics
- [ ] Comprehensive example programs
- [ ] Benchmarks

## Example

```text
let answer = 40 + 2;

if answer > 0 {
    print(answer);
}
```

Run the parser:

```bash
cargo run -- examples/hello.aster
```

Run semantic checking without dumping the AST:

```bash
cargo run -- --check examples/hello.aster
```

Run the test suite:

```bash
cargo test
```

## Compiler architecture

ASTER is deliberately staged so each compiler phase has a clear responsibility:

1. **Lexer** converts source text into tokens and reports lexical errors.
2. **Parser** converts tokens into an AST using recursive descent and precedence-aware expression parsing.
3. **Semantic analyzer** resolves lexical scopes and validates names, function arity, and basic expression types.
4. **Bytecode compiler** will lower the validated AST into a compact instruction stream.
5. **Virtual machine** will execute bytecode using explicit runtime values and call frames.

See [`docs/architecture.md`](docs/architecture.md) for the longer design notes.

## Design goals

- Clear compiler architecture
- Deterministic and useful diagnostics
- Strong test coverage
- Small, composable implementation stages
- No hidden runtime magic
- Correctness before feature count

## Development

Requires Rust 1.75+.

```bash
cargo fmt --all
cargo test --all-targets
cargo build --release
```

ASTER is an educational and engineering project focused on understanding how programming languages and runtimes work from first principles.
