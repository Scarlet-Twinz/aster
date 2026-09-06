use aster::{analyze_source, CompileError};

#[test]
fn accepts_valid_program() {
    let source = r#"
        let answer = 40 + 2;
        if answer > 0 {
            print(answer);
        }
    "#;

    assert!(analyze_source(source).is_ok());
}

#[test]
fn rejects_undefined_variable() {
    let error = analyze_source("print(missing);").expect_err("undefined variable should fail");
    match error {
        CompileError::Semantic(errors) => {
            assert!(errors.iter().any(|error| error.message.contains("undefined variable 'missing'")));
        }
        other => panic!("expected semantic error, got {other:?}"),
    }
}

#[test]
fn rejects_wrong_arithmetic_type() {
    let error = analyze_source("let value = \"aster\" + 1;").expect_err("mixed arithmetic should fail");
    match error {
        CompileError::Semantic(errors) => {
            assert!(errors.iter().any(|error| error.message.contains("arithmetic operators require numbers")));
        }
        other => panic!("expected semantic error, got {other:?}"),
    }
}

#[test]
fn checks_function_arity() {
    let error = analyze_source(
        r#"
            fn greet(name) {
                print(name);
            }
            greet();
        "#,
    )
    .expect_err("wrong argument count should fail");

    match error {
        CompileError::Semantic(errors) => {
            assert!(errors.iter().any(|error| error.message.contains("expects 1 argument(s), got 0")));
        }
        other => panic!("expected semantic error, got {other:?}"),
    }
}

#[test]
fn rejects_non_boolean_if_condition() {
    let error = analyze_source("if 42 { print(42); }").expect_err("numeric if condition should fail");
    match error {
        CompileError::Semantic(errors) => {
            assert!(errors.iter().any(|error| error.message.contains("if condition must be bool")));
        }
        other => panic!("expected semantic error, got {other:?}"),
    }
}
