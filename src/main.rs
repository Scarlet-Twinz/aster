use std::{env, fs, process};

use aster::parse_source;

fn main() {
    let path = match env::args().nth(1) {
        Some(path) => path,
        None => {
            eprintln!("Usage: aster <file.aster>");
            process::exit(2);
        }
    };

    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("ASTER: could not read '{}': {}", path, error);
            process::exit(1);
        }
    };

    match parse_source(&source) {
        Ok(program) => {
            println!("Parsed {} top-level statement(s).", program.statements.len());
            println!("{:#?}", program);
        }
        Err(error) => {
            eprintln!("ASTER: compilation failed\n{error:#?}");
            process::exit(1);
        }
    }
}
