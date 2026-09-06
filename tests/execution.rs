use aster::execute_source;

#[test]
fn executes_basic_addition() {
    let output = execute_source(
        r#"
            print(5 + 2);
        "#,
    )
    .expect("basic addition should execute");

    assert_eq!(output, vec!["7".to_string()]);
}

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

#[test]
fn executes_function_calls() {
    let output = execute_source(
        r#"
            fn add(a, b) {
                return a + b;
            }

            print(add(20, 22));
        "#,
    )
    .expect("function call should execute");

    assert_eq!(output, vec!["42".to_string()]);
}

#[test]
fn executes_recursive_functions() {
    let output = execute_source(
        r#"
            fn factorial(n) {
                if n <= 1 {
                    return 1;
                }
                return n * factorial(n - 1);
            }

            print(factorial(5));
        "#,
    )
    .expect("recursive function should execute");

    assert_eq!(output, vec!["120".to_string()]);
}
