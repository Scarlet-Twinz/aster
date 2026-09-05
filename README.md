# ASTER

**ASTER** is a general-purpose programming language and compiler built from scratch in Rust.

The project is designed as a real compiler implementation rather than a syntax-only demo. It will evolve through a complete pipeline:

`source → lexer → parser → AST → semantic analysis → bytecode → virtual machine`

## Current status

### Phase 1 — Language foundation

- [x] Rust project structure
- [x] Token model
- [x] Lexer
- [x] Abstract syntax tree
- [x] Recursive-descent parser
- [x] Expressions and precedence
- [x] Variable declarations
- [x] Blocks and conditional statements
- [x] Function declarations and calls
- [ ] Type system
- [ ] Semantic analysis

### Planned phases

**Phase 2 — Semantics**

- Static type system
- Scope resolution
- Symbol tables
- Semantic diagnostics
- Modules and imports

**Phase 3 — Execution**

- Bytecode instruction set
- Compiler/code generation
- Virtual machine
- Runtime values
- Memory management

**Phase 4 — Developer experience**

- REPL
- Standard library
- CLI tooling
- Documentation
- Comprehensive tests
- Benchmarks and example programs

## Example

```text
let answer = 40 + 2;

if answer > 0 {
    print(answer);
}
```

## Design goals

- Clear compiler architecture
- Deterministic and useful diagnostics
- Strong test coverage
- Small, composable implementation stages
- No hidden runtime magic

## Development

Requires Rust 1.75+.

```bash
cargo test
cargo run -- examples/hello.aster
```

ASTER is an educational and engineering project focused on understanding how programming languages and runtimes work from first principles.
