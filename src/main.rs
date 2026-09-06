use std::{env, fs, process};

use aster::{analyze_source, execute_source, parse_source, CompileError};

fn main() {
    let mut args = env::args().skip(1);
    let first = args.next();

    let (mode, path) = match first.as_deref() {
        Some("--check") => ("check", args.next()),
        Some("--dump-ast") => ("ast", args.next()),
        Some(path) => ("run", Some(path.to_string())),
        None => ("run", None),
    };

    let path = match path {
        Some(path) => path,
        None => {
            eprintln!("Usage: aster [--check|--dump-ast] <file.aster>");
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

    match mode {
        "check" => match analyze_source(&source) {
            Ok(program) => println!(
                "ASTER: semantic check passed ({} top-level statement(s)).",
                program.statements.len()
            ),
            Err(error) => fail(error),
        },
        "ast" => match parse_source(&source) {
            Ok(program) => println!("{:#?}", program),
            Err(error) => fail(error),
        },
        _ => match execute_source(&source) {
            Ok(_) => {}
            Err(error) => fail(error),
        },
    }
}

fn fail(error: CompileError) -> ! {
    eprintln!("ASTER: compilation failed\n{error:#?}");
    process::exit(1);
}
