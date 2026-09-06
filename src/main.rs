use std::{env, fs, process};

use aster::{analyze_source, parse_source, CompileError};

fn main() {
    let mut args = env::args().skip(1);
    let first = args.next();

    let (check_only, path) = match first.as_deref() {
        Some("--check") => (true, args.next()),
        Some(path) => (false, Some(path.to_string())),
        None => (false, None),
    };

    let path = match path {
        Some(path) => path,
        None => {
            eprintln!("Usage: aster [--check] <file.aster>");
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

    let result = if check_only {
        analyze_source(&source)
    } else {
        parse_source(&source)
    };

    match result {
        Ok(program) => {
            if check_only {
                println!("ASTER: semantic check passed ({} top-level statement(s)).", program.statements.len());
            } else {
                println!("Parsed {} top-level statement(s).", program.statements.len());
                println!("{:#?}", program);
            }
        }
        Err(error) => {
            eprintln!("ASTER: compilation failed\n{error:#?}");
            if matches!(error, CompileError::Semantic(_)) {
                process::exit(1);
            }
            process::exit(1);
        }
    }
}
