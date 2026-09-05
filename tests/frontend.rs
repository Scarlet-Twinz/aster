use aster::{ast::{BinaryOp, Expr, Stmt}, parse_source};

#[test]
fn parses_variable_and_arithmetic() {
    let program = parse_source("let answer = 40 + 2 * 1;").expect("source should parse");

    assert_eq!(program.statements.len(), 1);
    match &program.statements[0] {
        Stmt::Let { name, initializer } => {
            assert_eq!(name, "answer");
            match initializer {
                Expr::Binary { operator, .. } => assert_eq!(*operator, BinaryOp::Add),
                _ => panic!("expected binary expression"),
            }
        }
        _ => panic!("expected let statement"),
    }
}

#[test]
fn parses_function_calls_and_conditionals() {
    let source = r#"
        fn greet(name) {
            print(name);
        }

        if true {
            greet("aster");
        } else {
            print("no");
        }
    "#;

    let program = parse_source(source).expect("source should parse");
    assert_eq!(program.statements.len(), 2);
}

#[test]
fn reports_lexical_errors() {
    assert!(parse_source("let x = @;").is_err());
}
