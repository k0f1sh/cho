use super::support::output;
use std::io::{self, Cursor};

#[test]
fn numeric_and_string_equality_are_explicit() {
    assert_eq!(
        output("(filter (= $1 20)) (print $0)", "020\n20.0\n21\n"),
        "020\n20.0\n"
    );
    assert_eq!(
        output(
            r#"(filter (s/= $1 "Alice")) (print $2)"#,
            "Alice 20\nBob 30\n"
        ),
        "20\n"
    );
}

#[test]
fn numbers_follow_ieee_754_precision() {
    assert_eq!(
        output(
            r#"(print (= $1 $2) (s/= $1 $2))"#,
            "9007199254740992 9007199254740993\n",
        ),
        "true false\n"
    );
    assert_eq!(
        output("(print (= (+ $1 $2) $3))", "0.1 0.2 0.3\n"),
        "false\n"
    );
}

#[test]
fn minimum_maximum_and_clamp_convert_and_compose() {
    assert_eq!(
        output(
            "(print (n/min $1 $2 0) (n/max $1 $2 0) (n/clamp $1 0 100) (+ (n/min $1 $2) 1))",
            "-5 20\n50 120\n",
        ),
        "-5 20 0 -4\n0 120 50 51\n"
    );
}

#[test]
fn number_range_functions_report_argument_errors() {
    for (program, input, expected) in [
        (
            "(print (n/min $1 $2))",
            "1 nope\n",
            r#"record 1: n/min: argument 2 expects Number, but "nope" cannot be parsed as a number"#,
        ),
        (
            "(print (n/clamp $1 $2 $3))",
            "5 10 0\n",
            r#"record 1: n/clamp: argument 3 expects Number greater than or equal to MIN, but "0" is less than MIN"#,
        ),
    ] {
        let error = cho::run(program, Cursor::new(input), Vec::new()).unwrap_err();
        assert_eq!(error.to_string(), expected);
    }
}

#[test]
fn non_numeric_values_are_runtime_errors() {
    let error = cho::run(
        "(filter (> $2 20)) (print $1)",
        Cursor::new("Alice unknown\n"),
        Vec::new(),
    )
    .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert_eq!(
        error.to_string(),
        r#"record 1: >: argument 1 expects Number, but "unknown" cannot be parsed as a number"#
    );
}

#[test]
fn binary_arithmetic_converts_fields_and_composes_as_values() {
    assert_eq!(
        output(
            concat!(
                r#"(print (+ $1 $2) (- $1 $2) (* $1 2.0) (/ $1 $2) "#,
                r#"(+ (* $1 $2) 1) (-> $1 (+ 2) (* $2)))"#,
            ),
            "10 2.5\n-3 2\n",
        ),
        "12.5 7.5 20 4 26 30\n-1 -5 -6 -1.5 -5 -2\n"
    );
}

#[test]
fn remainder_handles_whole_fractional_and_negative_numbers_and_composes() {
    assert_eq!(
        output(
            "(print (% $1 $2) (% $3 $2) (% $4 $2) (+ 10 (% $1 $2)))",
            "7 3 -7 7.5\n-4 2 4 -4.5\n",
        ),
        "1 -1 1.5 11\n0 0 -0.5 10\n"
    );
}

#[test]
fn arithmetic_reports_invalid_numbers_zero_division_and_non_finite_results() {
    for (program, input, expected) in [
        (
            "(print (+ $1 2))",
            "unknown\n",
            r#"record 1: +: argument 1 expects Number, but "unknown" cannot be parsed as a number"#,
        ),
        (
            "(print (/ 1 $1))",
            "0\n",
            r#"record 1: /: argument 2 expects a non-zero Number, but "0" is zero"#,
        ),
        (
            "(print (% 1 $1))",
            "0\n",
            r#"record 1: %: argument 2 expects a non-zero Number, but "0" is zero"#,
        ),
    ] {
        let error = cho::run(program, Cursor::new(input), Vec::new()).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(error.to_string(), expected);
    }

    let error = cho::run("(print (* 1e308 1e308))", Cursor::new("x\n"), Vec::new()).unwrap_err();
    assert!(
        error.to_string().starts_with(
            "record 1: *: argument 2 expects Number producing a finite result with argument 1"
        ),
        "{error}"
    );

    for program in [r#"(print (+ "" 1))"#, "(print (+ $3 1))"] {
        let error = cho::run(program, Cursor::new("10 20\n"), Vec::new()).unwrap_err();
        assert!(
            error
                .to_string()
                .starts_with("record 1: +: argument 1 expects Number"),
            "{error}"
        );
    }
}

#[test]
fn fixed_number_formatting_rounds_keeps_zeroes_and_composes_as_a_string() {
    assert_eq!(
        output(
            concat!(
                r#"(print (n/fixed $1 2) (n/fixed $2 3) (n/fixed $2 0) "#,
                r#"(str "$" (n/fixed (* $1 2) (+ 1 1))))"#,
            ),
            "3 3.14159\n-2.5 1.25\n",
        ),
        "3.00 3.142 3 $6.00\n-2.50 1.250 1 $-5.00\n"
    );
}

#[test]
fn named_number_operations_handle_positive_negative_and_nested_values() {
    assert_eq!(
        output(
            "(print (n/trunc $1) (n/trunc $2) (n/floor $2) (n/ceil $2) (n/round $1) (n/round $2) (n/abs $2) (+ 1 (n/trunc $1)))",
            "2.5125 -2.5125\n"
        ),
        "2 -2 -3 -2 3 -3 2.5125 3\n"
    );
    assert_eq!(
        output(
            "(print (n/trunc $1) (n/floor $1) (n/ceil $1) (n/round $1) (n/abs $1))",
            "-0.25\n"
        ),
        "0 -1 0 0 0.25\n"
    );
}

#[test]
fn named_number_operations_reject_non_numbers_and_empty_values() {
    for function in ["n/trunc", "n/floor", "n/ceil", "n/round", "n/abs"] {
        let program = format!("(print ({function} $2))");
        let error = cho::run(&program, Cursor::new("x\n"), Vec::new()).unwrap_err();
        assert!(
            error
                .to_string()
                .starts_with(&format!("record 1: {function}: argument 1 expects Number")),
            "{error}"
        );
    }
}

#[test]
fn fixed_number_formatting_rejects_invalid_digits_and_values() {
    for (program, expected) in [
        (
            "(print (n/fixed $1 -1))",
            "record 1: n/fixed: argument 2 expects Number (whole digits from 0 to 100)",
        ),
        (
            "(print (n/fixed $1 1.5))",
            "record 1: n/fixed: argument 2 expects Number (whole digits from 0 to 100)",
        ),
        (
            "(print (n/fixed $1 101))",
            "record 1: n/fixed: argument 2 expects Number (whole digits from 0 to 100)",
        ),
        (
            "(print (n/fixed $2 2))",
            "record 1: n/fixed: argument 1 expects Number",
        ),
        (
            r#"(print (n/fixed $1 ""))"#,
            "record 1: n/fixed: argument 2 expects Number",
        ),
    ] {
        let error = cho::run(program, Cursor::new("3\n"), Vec::new()).unwrap_err();
        assert!(error.to_string().starts_with(expected), "{error}");
    }
}

#[test]
fn boolean_values_do_not_convert_to_numbers_or_url_components() {
    for program in ["(print (+ (> 2 1) 1))", r#"(print (url/encode (> 2 1)))"#] {
        let error = cho::run(program, Cursor::new("x\n"), Vec::new()).unwrap_err();
        assert!(error.to_string().contains("has type Boolean"), "{error}");
    }
}
