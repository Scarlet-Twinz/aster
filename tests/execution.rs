use aster::execute_source;

#[test]
fn executes_arithmetic_and_variables() {
    let output = execute_source(
        r#"
            let answer = 40 + 2;
            print(answer);
        "#,
    )
    .expect("program should execute");

    assert_eq!(output, vec!["42".to_string()]);
}

#[test]
fn executes_conditionals() {
    let output = execute_source(
        r#"
            let answer = 42;
            if answer > 0 {
                print(answer);
            } else {
                print(0);
            }
        "#,
    )
    .expect("program should execute");

    assert_eq!(output, vec!["42".to_string()]);
}

#[test]
fn executes_assignment() {
    let output = execute_source(
        r#"
            let answer = 40;
            answer = answer + 2;
            print(answer);
        "#,
    )
    .expect("program should execute");

    assert_eq!(output, vec!["42".to_string()]);
}
