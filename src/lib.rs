pub mod ast;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod token;
pub mod type_system;

use ast::Program;
use lexer::{LexError, Lexer};
use parser::{ParseError, Parser};
use semantic::{analyze, SemanticError};

#[derive(Debug)]
pub enum CompileError {
    Lex(Vec<LexError>),
    Parse(Vec<ParseError>),
    Semantic(Vec<SemanticError>),
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
