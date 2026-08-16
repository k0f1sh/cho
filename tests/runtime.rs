use std::io::{self, Cursor};

fn output(program: &str, input: &str) -> String {
    let mut result = Vec::new();
    cho::run(program, Cursor::new(input), &mut result).unwrap();
    String::from_utf8(result).unwrap()
}

fn output_with_separator(program: &str, separator: &str, input: &str) -> String {
    let mut result = Vec::new();
    cho::run_with_field_separator(program, Some(separator), Cursor::new(input), &mut result)
        .unwrap();
    String::from_utf8(result).unwrap()
}

#[test]
fn prints_fields_strings_and_concatenated_values() {
    assert_eq!(
        output(
            r#"(print $1 "score:" $2) (print (str $1 ":" $2))"#,
            "Alice 20\nBob 30\n"
        ),
        "Alice score: 20\nAlice:20\nBob score: 30\nBob:30\n"
    );
}

#[test]
fn joins_values_with_a_separator() {
    assert_eq!(
        output(
            r#"(print (join "," $1 $2 $3)) (print (join "\t" $1 $3))"#,
            "Alice 20 Tokyo\n"
        ),
        "Alice,20,Tokyo\nAlice\tTokyo\n"
    );
    assert_eq!(output(r#"(print (join ","))"#, "Alice\n"), "\n");
}

#[test]
fn count_counts_unicode_characters() {
    assert_eq!(
        output("(print (count $1))", "Alice\n東京\n🦀\n"),
        "5\n2\n1\n"
    );
}

#[test]
fn count_can_be_used_in_filters() {
    assert_eq!(
        output(
            "(filter (> (count $1) 3)) (print $1)",
            "Al\nAlice\n東京\nCarol\n"
        ),
        "Alice\nCarol\n"
    );
}

#[test]
fn escape_makes_tabs_and_backslashes_visible() {
    assert_eq!(
        output(r#"(print (escape $0))"#, "first\tsecond\\third\r\n"),
        r#"first\tsecond\\third"#.to_owned() + "\n"
    );
}

#[test]
fn field_zero_preserves_the_line_and_missing_fields_are_empty() {
    assert_eq!(
        output("(print $0 $3)", "  Alice   20  \n"),
        "  Alice   20   \n"
    );
}

#[test]
fn nr_and_nf_describe_the_record() {
    assert_eq!(
        output("(print NR NF)", "Alice 20\nBob\t30 Osaka\n\n"),
        "1 2\n2 3\n3 0\n"
    );
}

#[test]
fn an_empty_print_prints_an_empty_line() {
    assert_eq!(output("(print)", "Alice\nBob\n"), "\n\n");
}

#[test]
fn filter_skips_remaining_expressions_for_a_record() {
    assert_eq!(
        output(
            r#"(print "checking" $1) (filter (> $2 20)) (print "passed" $1)"#,
            "Alice 18\nBob 30\n"
        ),
        "checking Alice\nchecking Bob\npassed Bob\n"
    );
}

#[test]
fn multiple_filters_form_an_and_condition() {
    assert_eq!(
        output(
            "(filter (> $2 20)) (filter (< $2 40)) (print $1)",
            "Alice 18\nBob 30\nCarol 45\n"
        ),
        "Bob\n"
    );
}

#[test]
fn not_inverts_a_predicate() {
    assert_eq!(
        output(
            r#"(filter (not (reg "debug"))) (print $0)"#,
            "info ready\ndebug details\nerror failed\n"
        ),
        "info ready\nerror failed\n"
    );
}

#[test]
fn and_requires_every_predicate_to_match() {
    assert_eq!(
        output(
            "(filter (and (> $2 20) (< $2 40))) (print $1)",
            "Alice 18\nBob 30\nCarol 45\n"
        ),
        "Bob\n"
    );
}

#[test]
fn or_requires_any_predicate_to_match() {
    assert_eq!(
        output(
            r#"(filter (or (= $1 "Alice") (= $1 "Bob"))) (print $1)"#,
            "Alice 20\nBob 30\nCarol 40\n"
        ),
        "Alice\nBob\n"
    );
}

#[test]
fn boolean_predicates_can_be_nested_with_regexes() {
    assert_eq!(
        output(
            r#"(filter (and (not (reg "debug")) (or (reg "error") (>= $2 40)))) (print $0)"#,
            "info 10\ndebug error 50\nerror 20\ninfo 40\n"
        ),
        "error 20\ninfo 40\n"
    );
}

#[test]
fn supports_all_comparison_operators() {
    let input = "low 10\nequal 20\nhigh 30\n";
    assert_eq!(output("(filter (> $2 20)) (print $1)", input), "high\n");
    assert_eq!(
        output("(filter (>= $2 20)) (print $1)", input),
        "equal\nhigh\n"
    );
    assert_eq!(output("(filter (< $2 20)) (print $1)", input), "low\n");
    assert_eq!(
        output("(filter (<= $2 20)) (print $1)", input),
        "low\nequal\n"
    );
    assert_eq!(output("(filter (= $2 20)) (print $1)", input), "equal\n");
    assert_eq!(
        output("(filter (!= $2 20)) (print $1)", input),
        "low\nhigh\n"
    );
}

#[test]
fn equality_compares_numbers_or_falls_back_to_strings() {
    assert_eq!(
        output("(filter (= $1 20)) (print $0)", "020\n20.0\n21\n"),
        "020\n20.0\n"
    );
    assert_eq!(
        output(
            r#"(filter (= $1 "Alice")) (print $2)"#,
            "Alice 20\nBob 30\n"
        ),
        "20\n"
    );
}

#[test]
fn non_numeric_values_do_not_match_ordered_comparisons() {
    assert_eq!(
        output("(filter (> $2 20)) (print $1)", "Alice unknown\n"),
        ""
    );
}

#[test]
fn regex_filters_match_lines_or_specific_fields() {
    assert_eq!(
        output(
            r#"(filter (reg "^error:")) (print NR $0)"#,
            "info: ready\nerror: failed\n"
        ),
        "2 error: failed\n"
    );
    assert_eq!(
        output(
            r#"(filter (reg $1 "^[A-Z][a-z]+$")) (print $1)"#,
            "Alice 20\nbob 30\nCAROL 40\n"
        ),
        "Alice\n"
    );
}

#[test]
fn regex_literals_preserve_escapes_and_reg_has_a_tilde_alias() {
    assert_eq!(
        output(r#"(filter (~ $1 /^\d+$/)) (print $1)"#, "123\n12a\n456\n"),
        "123\n456\n"
    );
    assert_eq!(
        output(
            r#"(filter (reg /^foo\/bar$/)) (print $0)"#,
            "foo/bar\nfoo\n"
        ),
        "foo/bar\n"
    );
}

#[test]
fn invalid_regexes_fail_before_processing() {
    let error = cho::run(
        r#"(filter (reg "[")) (print $0)"#,
        Cursor::new("input\n"),
        Vec::new(),
    )
    .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
}

#[test]
fn custom_separators_control_fields_and_nf() {
    assert_eq!(
        output_with_separator("(print NF $1 $3 $0)", ",", "Alice,20,Tokyo\nBob,30,Osaka\n"),
        "3 Alice Tokyo Alice,20,Tokyo\n3 Bob Osaka Bob,30,Osaka\n"
    );
}

#[test]
fn field_separators_can_be_regexes_and_preserve_empty_fields() {
    assert_eq!(
        output_with_separator("(print NF $1 $2 $3)", "[,;]", ",Alice;\n"),
        "3  Alice \n"
    );
}

#[test]
fn empty_records_have_no_fields() {
    assert_eq!(output_with_separator("(print NF)", ",", "\n"), "0\n");
}

#[test]
fn separators_that_match_empty_strings_are_rejected() {
    let error =
        cho::run_with_field_separator("(print $1)", Some(".*"), Cursor::new("Alice\n"), Vec::new())
            .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
}

#[test]
fn csv_mode_decodes_quoted_fields() {
    let mut result = Vec::new();
    cho::run_csv(
        "(print NF $1 $2 $3)",
        Cursor::new("Alice,\"Bob, Jr.\",\"said \"\"hello\"\"\"\n"),
        &mut result,
    )
    .unwrap();
    assert_eq!(
        String::from_utf8(result).unwrap(),
        "3 Alice Bob, Jr. said \"hello\"\n"
    );
}

#[test]
fn csv_mode_streams_logical_records_with_embedded_newlines() {
    let mut result = Vec::new();
    cho::run_csv(
        "(print NR $2 $0)",
        Cursor::new("Alice,\"Tokyo\nJapan\"\r\nBob,Osaka\r\n"),
        &mut result,
    )
    .unwrap();
    assert_eq!(
        String::from_utf8(result).unwrap(),
        "1 Tokyo\nJapan Alice,\"Tokyo\nJapan\"\n2 Osaka Bob,Osaka\n"
    );
}

#[test]
fn escape_keeps_multiline_csv_fields_on_one_output_line() {
    let mut result = Vec::new();
    cho::run_csv(
        "(print NF (escape $2))",
        Cursor::new("Tokyo,\"rain\r\nthen\tsun\\later\"\n"),
        &mut result,
    )
    .unwrap();
    assert_eq!(
        String::from_utf8(result).unwrap(),
        "2 rain\\r\\nthen\\tsun\\\\later\n"
    );
}

#[test]
fn csv_mode_preserves_a_final_record_without_a_newline() {
    let mut result = Vec::new();
    cho::run_csv("(print $0 $2)", Cursor::new("Alice,Tokyo"), &mut result).unwrap();
    assert_eq!(String::from_utf8(result).unwrap(), "Alice,Tokyo Tokyo\n");
}
