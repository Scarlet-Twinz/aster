use std::{env, fs, io::{self, BufRead, Write}, process};

use aster::{analyze_source, disassemble_source, execute_source, parse_source, CompileError};

fn main() {
    let mut args = env::args().skip(1);
    let first = args.next();

    if matches!(first.as_deref(), Some("--repl")) {
        repl();
        return;
    }

    let (mode, path) = match first.as_deref() {
        Some("--check") => ("check", args.next()),
        Some("--dump-ast") => ("ast", args.next()),
        Some("--dump-bytecode") => ("bytecode", args.next()),
        Some("--help") | Some("-h") => { print_usage(); return; }
        Some(path) => ("run", Some(path.to_string())),
        None => ("run", None),
    };

    let path = match path { Some(path) => path, None => { print_usage(); process::exit(2); } };
    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => { eprintln!("ASTER: could not read '{}': {}", path, error); process::exit(1); }
    };

    match mode {
        "check" => match analyze_source(&source) {
            Ok(program) => println!("ASTER: semantic check passed ({} top-level statement(s)).", program.statements.len()),
            Err(error) => fail(error),
        },
        "ast" => match parse_source(&source) { Ok(program) => println!("{:#?}", program), Err(error) => fail(error) },
        "bytecode" => match disassemble_source(&source) { Ok(bytecode) => print!("{}", bytecode), Err(error) => fail(error) },
        _ => match execute_source(&source) { Ok(_) => {}, Err(error) => fail(error) },
    }
}

fn repl() {
    println!("ASTER REPL v0.1");
    println!("Enter ASTER statements. Type :help for commands or :quit to exit.");
    let stdin = io::stdin();
    loop {
        print!("aster> ");
        let _ = io::stdout().flush();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line).is_err() { break; }
        let input = line.trim();
        match input {
            ":quit" | ":exit" => break,
            ":help" => println!(":help  show commands\n:quit  exit the REPL"),
            "" => continue,
            _ => match execute_source(input) { Ok(_) => {}, Err(error) => eprintln!("ASTER: {error}") },
        }
    }
}

fn print_usage() {
    println!("Usage: aster [--check|--dump-ast|--dump-bytecode|--repl|--help] <file.aster>\n\nModes:\n  <file.aster>       Compile and execute the program\n  --check            Run semantic analysis only\n  --dump-ast         Parse and print the AST\n  --dump-bytecode    Compile and print VM bytecode\n  --repl             Start the interactive REPL\n  --help             Show this help");
}

fn fail(error: CompileError) -> ! { eprintln!("ASTER: compilation failed\n{error}"); process::exit(1); }
