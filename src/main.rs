use std::{env, fs, process};

use aster::{analyze_source, disassemble_source, execute_source, parse_source, CompileError};

fn main() {
    let mut args = env::args().skip(1);
    let first = args.next();

    let (mode, path) = match first.as_deref() {
        Some("--check") => ("check", args.next()),
        Some("--dump-ast") => ("ast", args.next()),
        Some("--dump-bytecode") => ("bytecode", args.next()),
        Some("--help") | Some("-h") => {
            print_usage();
            return;
        }
        Some(path) => ("run", Some(path.to_string())),
        None => ("run", None),
    };

    let path = match path {
        Some(path) => path,
        None => {
            print_usage();
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
        "bytecode" => match disassemble_source(&source) {
            Ok(bytecode) => print!("{}", bytecode),
            Err(error) => fail(error),
        },
        _ => match execute_source(&source) {
            Ok(_) => {}
            Err(error) => fail(error),
        },
    }
}

fn print_usage() {
    println!(
        "Usage: aster [--check|--dump-ast|--dump-bytecode|--help] <file.aster>\n\n\
         Modes:\n\
           <file.aster>       Compile and execute the program\n\
           --check            Run semantic analysis only\n\
           --dump-ast         Parse and print the AST\n\
           --dump-bytecode    Compile and print VM bytecode\n\
           --help             Show this help"
    );
}

fn fail(error: CompileError) -> ! {
    eprintln!("ASTER: compilation failed\n{error:#?}");
    process::exit(1);
}
