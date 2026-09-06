use aster::disassemble_source;

#[test]
fn disassembles_main_program() {
    let output = disassemble_source(
        r#"
            let answer = 40 + 2;
            print(answer);
        "#,
    )
    .expect("source should compile");

    assert!(output.contains("== ASTER main =="));
    assert!(output.contains("CONSTANT"));
    assert!(output.contains("ADD"));
    assert!(output.contains("STORE_GLOBAL"));
    assert!(output.contains("PRINT"));
    assert!(output.contains("HALT"));
}

#[test]
fn disassembles_functions() {
    let output = disassemble_source(
        r#"
            fn add(a, b) {
                return a + b;
            }
            print(add(20, 22));
        "#,
    )
    .expect("function source should compile");

    assert!(output.contains("== ASTER function add(2) =="));
    assert!(output.contains("LOAD_LOCAL 0"));
    assert!(output.contains("LOAD_LOCAL 1"));
    assert!(output.contains("RETURN"));
    assert!(output.contains("CALL function#0"));
}
