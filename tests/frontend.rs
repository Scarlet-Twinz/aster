use aster::{ast::{BinaryOp, Expr, Stmt}, parse_source};

#[test]
fn parses_variable_and_arithmetic() {
    let program = parse_source("let answer = 40 + 2 * 1;").expect("source should parse");
    assert_eq!(program.statements.len(), 1);
    match &program.statements[0] {
        Stmt::Let { name, type_annotation, initializer } => {
            assert_eq!(name, "answer");
            assert_eq!(*type_annotation, None);
            match initializer { Expr::Binary { operator, .. } => assert_eq!(*operator, BinaryOp::Add), _ => panic!("expected binary expression") }
        }
        _ => panic!("expected let statement"),
    }
}

#[test]
fn parses_explicit_type_annotation() {
    let program = parse_source("let answer: number = 42;").expect("typed let should parse");
    match &program.statements[0] {
        Stmt::Let { type_annotation, .. } => assert_eq!(*type_annotation, Some(aster::type_system::Type::Number)),
        _ => panic!("expected let statement"),
    }
}

#[test]
fn parses_function_calls_and_conditionals() {
    let source = r#"fn greet(name) { print(name); } if true { greet("aster"); } else { print("no"); }"#;
    let program = parse_source(source).expect("source should parse");
    assert_eq!(program.statements.len(), 2);
}

#[test]
fn parses_assignment_after_declaration() {
    let source = "let answer = 40; answer = answer + 2;";
    let program = parse_source(source).expect("source should parse");
    assert_eq!(program.statements.len(), 2);
    match &program.statements[1] {
        Stmt::Expression(Expr::Assign { name, value }) => { assert_eq!(name, "answer"); assert!(matches!(value.as_ref(), Expr::Binary { operator: BinaryOp::Add, .. })); }
        _ => panic!("expected assignment expression"),
    }
}

#[test]
fn reports_lexical_errors() { assert!(parse_source("let x = @;").is_err()); }
