pub mod ast;
pub mod bytecode;
pub mod disassembler;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod token;
pub mod type_system;
pub mod vm;

use ast::Program;
use bytecode::{compile, BytecodeError, Chunk};
use disassembler::disassemble;
use lexer::{LexError, Lexer};
use parser::{ParseError, Parser};
use semantic::{analyze, SemanticError};
use vm::{run, VmError};

#[derive(Debug)]
pub enum CompileError {
    Lex(Vec<LexError>),
    Parse(Vec<ParseError>),
    Semantic(Vec<SemanticError>),
    Bytecode(BytecodeError),
    Runtime(VmError),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lex(errors) => { writeln!(f, "lexical error(s):")?; for e in errors { writeln!(f, "  {}:{}: {}", e.line, e.column, e.message)?; } }
            Self::Parse(errors) => { writeln!(f, "parse error(s):")?; for e in errors { writeln!(f, "  {}:{}: {}", e.line, e.column, e.message)?; } }
            Self::Semantic(errors) => { writeln!(f, "semantic error(s):")?; for e in errors { writeln!(f, "  {}", e.message)?; } }
            Self::Bytecode(e) => write!(f, "bytecode error: {}", e.message)?,
            Self::Runtime(e) => write!(f, "runtime error: {}", e.message)?,
        }
        Ok(())
    }
}

impl std::error::Error for CompileError {}

pub fn parse_source(source: &str) -> Result<Program, CompileError> {
    let tokens = Lexer::new(source).tokenize().map_err(|error| CompileError::Lex(vec![error]))?;
    Parser::new(tokens).parse().map_err(CompileError::Parse)
}

pub fn analyze_source(source: &str) -> Result<Program, CompileError> {
    let program = parse_source(source)?;
    analyze(&program).map_err(CompileError::Semantic)?;
    Ok(program)
}

pub fn compile_source(source: &str) -> Result<Chunk, CompileError> {
    let program = analyze_source(source)?;
    compile(&program).map_err(CompileError::Bytecode)
}

pub fn execute_source(source: &str) -> Result<Vec<String>, CompileError> {
    let chunk = compile_source(source)?;
    run(&chunk).map_err(CompileError::Runtime)
}

pub fn disassemble_source(source: &str) -> Result<String, CompileError> {
    let chunk = compile_source(source)?;
    Ok(disassemble(&chunk))
}
