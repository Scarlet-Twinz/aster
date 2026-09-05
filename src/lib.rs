pub mod ast;
pub mod lexer;
pub mod parser;
pub mod token;

use ast::Program;
use lexer::{LexError, Lexer};
use parser::{ParseError, Parser};

#[derive(Debug)]
pub enum CompileError {
    Lex(Vec<LexError>),
    Parse(Vec<ParseError>),
}

pub fn parse_source(source: &str) -> Result<Program, CompileError> {
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(|error| CompileError::Lex(vec![error]))?;

    Parser::new(tokens)
        .parse()
        .map_err(CompileError::Parse)
}
