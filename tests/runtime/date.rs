use super::support::output;
use std::io::Cursor;

#[test]
fn date_values_validate_render_and_extract_components() {
    assert_eq!(
        output(
            "(print (date $1) (d/year $1) (d/month $1) (d/day $1) (d/weekday $1))",
            "2024-02-29\n",
        ),
        "2024-02-29 2024 2 29 4\n"
    );
    assert_eq!(
        output(
            "(print (d/weekday $1) (d/weekday $2))",
            "2024-03-04 2024-03-10\n",
        ),
        "1 7\n"
    );
}

#[test]
fn date_values_support_all_comparison_operators() {
    for predicate in [
        r#"(d/< "2024-01-01" "2024-01-02")"#,
        r#"(d/<= "2024-01-01" "2024-01-01")"#,
        r#"(d/> "2024-01-02" "2024-01-01")"#,
        r#"(d/>= "2024-01-01" "2024-01-01")"#,
        r#"(d/= "2024-02-29" "2024-02-29")"#,
        r#"(d/!= "2024-02-29" "2024-03-01")"#,
    ] {
        assert_eq!(
            output(&format!(r#"(print (if {predicate} "yes" "no"))"#), "x\n"),
            "yes\n"
        );
    }
}

#[test]
fn date_values_add_subtract_and_diff_calendar_days() {
    assert_eq!(
        output(
            "(print (d/add $1 1) (d/add $1 2) (d/sub $1 -1) (d/diff $2 $1) (d/diff $1 $2))",
            "2024-02-28 2024-03-01\n",
        ),
        "2024-02-29 2024-03-01 2024-02-29 2 -2\n"
    );
    assert_eq!(
        output("(print (d/add $1 $2) (d/sub $1 0))", "2023-12-31 1\n",),
        "2024-01-01 2023-12-31\n"
    );
}

#[test]
fn date_values_compose_and_thread_without_becoming_strings() {
    assert_eq!(
        output(
            r#"(print (d/year (d/add (date $1) 366)) (-> $1 (d/add 1) d/weekday) (str "date=" (d/sub $1 1)))"#,
            "2024-01-01\n",
        ),
        "2025 2 date=2023-12-31\n"
    );
    assert_eq!(
        output(r#"(print (default (date $1) "invalid"))"#, "not-a-date\n",),
        "invalid\n"
    );
}

#[test]
fn date_input_is_strict_fixed_width_gregorian() {
    for invalid in [
        "2024-2-29",
        "2024-02-9",
        "20240229",
        " 2024-02-29",
        "2024-02-29 ",
        "2023-02-29",
        "2024-13-01",
        "2024-01-00",
        "2024-01-01T00:00:00Z",
    ] {
        let error =
            cho::run("(date $0)", Cursor::new(format!("{invalid}\n")), Vec::new()).unwrap_err();
        assert!(
            error
                .to_string()
                .starts_with("record 1: date: argument 1 expects Date"),
            "unexpected error for {invalid:?}: {error}"
        );
    }

    let error = cho::run("(date $2)", Cursor::new("only-one-field\n"), Vec::new()).unwrap_err();
    assert!(error.to_string().starts_with("record 1: date: argument 1"));

    assert_eq!(output("(date $1)", "0000-01-01\n"), "0000-01-01\n");
    assert_eq!(output("(date $1)", "9999-12-31\n"), "9999-12-31\n");
}

#[test]
fn date_errors_identify_function_and_argument() {
    for function in ["d/year", "d/month", "d/day", "d/weekday"] {
        let error = cho::run(
            &format!("({function} $1)"),
            Cursor::new("invalid\n"),
            Vec::new(),
        )
        .unwrap_err();
        assert!(
            error
                .to_string()
                .starts_with(&format!("record 1: {function}: argument 1 expects Date"))
        );
    }

    let error = cho::run(
        r#"(d/> "2024-01-01" $1)"#,
        Cursor::new("invalid\n"),
        Vec::new(),
    )
    .unwrap_err();
    assert_eq!(
        error.to_string(),
        r#"record 1: d/>: argument 2 expects Date, but "invalid" is not in YYYY-MM-DD format"#
    );

    let error = cho::run("(date (dt/unix 0))", Cursor::new("x\n"), Vec::new()).unwrap_err();
    assert!(error.to_string().ends_with("has type DateTime"));

    for (program, expected) in [
        (r#"(d/add "invalid" 1)"#, "d/add: argument 1"),
        (r#"(d/sub "invalid" 1)"#, "d/sub: argument 1"),
        (r#"(d/diff "invalid" "2024-01-01")"#, "d/diff: argument 1"),
        (r#"(d/diff "2024-01-01" "invalid")"#, "d/diff: argument 2"),
    ] {
        let error = cho::run(program, Cursor::new("x\n"), Vec::new()).unwrap_err();
        assert!(error.to_string().contains(expected));
    }
}

#[test]
fn date_arithmetic_rejects_fractional_invalid_and_overflowing_days() {
    for program in [
        r#"(d/add "2024-01-01" 1.5)"#,
        r#"(d/add "2024-01-01" "unknown")"#,
        r#"(d/add "9999-12-31" 1)"#,
        r#"(d/sub "0000-01-01" 1)"#,
    ] {
        let error = cho::run(program, Cursor::new("x\n"), Vec::new()).unwrap_err();
        assert!(
            error.to_string().starts_with("record 1: d/"),
            "unexpected error for {program}: {error}"
        );
        assert!(error.to_string().contains("argument 2"));
    }
}

#[test]
fn date_conversion_failure_preserves_prior_output() {
    let mut stdout = Vec::new();
    let error = cho::run(
        "(date $1)",
        Cursor::new("2024-01-01\ninvalid\n"),
        &mut stdout,
    )
    .unwrap_err();
    assert_eq!(String::from_utf8(stdout).unwrap(), "2024-01-01\n");
    assert!(error.to_string().starts_with("record 2: date: argument 1"));
}
