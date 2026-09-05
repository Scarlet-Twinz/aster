# ASTER Compiler Architecture

ASTER is intentionally built as a sequence of explicit compiler stages.

```text
ASTER source
    │
    ▼
┌──────────┐
│  Lexer   │  characters → tokens
└────┬─────┘
     ▼
┌──────────┐
│  Parser  │  tokens → AST
└────┬─────┘
     ▼
┌──────────┐
│   AST    │  structured program representation
└────┬─────┘
     ▼
┌──────────┐
│ Semantic │  names, scopes, types, diagnostics
│ Analysis │
└────┬─────┘
     ▼
┌──────────┐
│ Bytecode │  AST → executable instructions
│ Compiler │
└────┬─────┘
     ▼
┌──────────┐
│    VM    │  instruction execution + runtime
└──────────┘
```

## Current implementation

The first implementation milestone contains a lexer, recursive-descent parser, AST, CLI, examples, and frontend tests.

### Lexer

The lexer tracks source position and produces tokens for:

- identifiers and keywords
- numbers and strings
- arithmetic operators
- comparison and equality operators
- logical operators
- delimiters
- comments

### Parser

The parser uses recursive descent with explicit precedence levels. Assignment is the lowest expression level, followed by logical operators, equality, comparisons, arithmetic, unary expressions, and calls.

This gives expressions such as `40 + 2 * 3` the expected precedence without relying on a parser generator.

## Next architectural milestone

The next major component is semantic analysis. It will introduce lexical scopes, symbol tables, name resolution, and diagnostics without mixing semantic rules into the parser.
