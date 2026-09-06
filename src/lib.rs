pub mod ast;
pub mod bytecode;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod token;
pub mod type_system;
pub mod vm;

use ast::Program;
use bytecode::{compile, BytecodeError, Chunk};
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

pub fn parse_source(source: &str) -> Result<Program, CompileError> {
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(|error| CompileError::Lex(vec![error]))?;

    Parser::new(tokens)
        .parse()
        .map_err(CompileError::Parse)
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
