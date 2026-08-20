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
fn short_top_level_aliases_filter_and_print() {
    assert_eq!(
        output("(f (> $2 20)) (p $1)", "Alice 18\nBob 30\n"),
        "Bob\n"
    );
    assert_eq!(output("(p)", "Alice\nBob\n"), "\n\n");
}

#[test]
fn joins_values_with_a_separator() {
    assert_eq!(
        output(
            r#"(print (s/join "," $1 $2 $3)) (print (s/join "\t" $1 $3))"#,
            "Alice 20 Tokyo\n"
        ),
        "Alice,20,Tokyo\nAlice\tTokyo\n"
    );
    assert_eq!(output(r#"(print (s/join ","))"#, "Alice\n"), "\n");
}

#[test]
fn part_extracts_one_literal_delimited_part() {
    assert_eq!(
        output(
            concat!(
                r#"(print (s/part ":" 1 $1) "#,
                r#"(s/part "=" 2 $2) "#,
                r#"(s/part "]:" 1 (s/part "[" 2 $3)))"#,
            ),
            "192.168.10.20:39652 SRC=10.0.0.25 [fd00::1]:443\n",
        ),
        "192.168.10.20 10.0.0.25 fd00::1\n"
    );
}

#[test]
fn part_preserves_empty_parts_and_returns_the_whole_unsplit_value() {
    assert_eq!(
        output(
            concat!(
                r#"(print (s/join "|" "#,
                r#"(s/part ":" 1 ":a::") "#,
                r#"(s/part ":" 2 ":a::") "#,
                r#"(s/part ":" 3 ":a::") "#,
                r#"(s/part ":" 4 ":a::"))) "#,
                r#"(print (s/part ":" 1 "whole")) "#,
                r#"(print (s/part "区切" 2 "左区切右") "#,
                r#"(default (s/part ":" 1 "") "empty"))"#,
            ),
            "x\n",
        ),
        "|a||\nwhole\n右 empty\n"
    );
}

#[test]
fn part_composes_with_values_threading_and_typed_predicates() {
    assert_eq!(
        output(
            concat!(
                r#"(filter (ip/private? (->> $1 (s/part ":" 1)))) "#,
                r#"(print (str "ip=" (s/upper (s/part (str ":") (s/count "x") $1))))"#,
            ),
            "10.1.2.3:443\n8.8.8.8:53\n",
        ),
        "ip=10.1.2.3\n"
    );
}

#[test]
fn part_returns_empty_for_a_missing_part_and_composes_with_other_values() {
    assert_eq!(
        output(
            concat!(
                r#"(print (s/join "|" (s/part ":" 3 $1) (s/upper (s/part ":" 3 $1)))) "#,
                r#"(print (default (s/part ":" 3 $1) "missing") "#,
                r#"(default (s/part ":" 2 $2) "empty"))"#,
            ),
            "a:b x:\n",
        ),
        "|\nmissing empty\n"
    );
}

#[test]
fn part_keeps_invalid_delimiters_and_positions_strict() {
    for (program, expected) in [
        (
            r#"(print (s/part "" 1 $1))"#,
            "record 1: s/part: argument 1 expects a non-empty delimiter",
        ),
        (
            r#"(print (s/part ":" 0 $1))"#,
            "record 1: s/part: argument 2 expects Number (positive whole part position)",
        ),
        (
            r#"(print (s/part ":" -1 $1))"#,
            "record 1: s/part: argument 2 expects Number (positive whole part position)",
        ),
        (
            r#"(print (s/part ":" 1.5 $1))"#,
            "record 1: s/part: argument 2 expects Number (positive whole part position)",
        ),
        (
            r#"(print (s/part ":" NaN $1))"#,
            "record 1: s/part: argument 2 expects finite Number",
        ),
        (
            r#"(print (s/part ":" 1e40 $1))"#,
            "record 1: s/part: argument 2 expects Number (representable part position)",
        ),
    ] {
        let error = cho::run(program, Cursor::new("a:b\n"), Vec::new()).unwrap_err();
        assert!(error.to_string().starts_with(expected), "{error}");
    }

    let error = cho::run(
        r#"(filter (> (s/part ":" 3 $1) 0))"#,
        Cursor::new("a:b\n"),
        Vec::new(),
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .starts_with("record 1: >: argument 1 expects Number"),
        "{error}"
    );
}

#[test]
fn count_counts_unicode_characters() {
    assert_eq!(
        output("(print (s/count $1))", "Alice\n東京\n🦀\n"),
        "5\n2\n1\n"
    );
}

#[test]
fn count_can_be_used_in_filters() {
    assert_eq!(
        output(
            "(filter (> (s/count $1) 3)) (print $1)",
            "Al\nAlice\n東京\nCarol\n"
        ),
        "Alice\nCarol\n"
    );
}

#[test]
fn escape_makes_tabs_and_backslashes_visible() {
    assert_eq!(
        output(r#"(print (s/escape $0))"#, "first\tsecond\\third\r\n"),
        r#"first\tsecond\\third"#.to_owned() + "\n"
    );
}

#[test]
fn if_selects_a_value_and_nests_with_other_values() {
    assert_eq!(
        output(
            r#"(print (str $1 ":" (if (>= $2 20) (s/upper "adult") (s/lower "MINOR"))))"#,
            "Alice 18\nBob 30\n"
        ),
        "Alice:minor\nBob:ADULT\n"
    );
}

#[test]
fn lower_and_upper_apply_unicode_case_conversion() {
    assert_eq!(
        output(r#"(print (s/lower $1) (s/upper $2))"#, "ÄLICE 東京abc\n"),
        "älice 東京ABC\n"
    );
}

#[test]
fn default_replaces_only_empty_values_and_can_be_nested() {
    assert_eq!(
        output(
            r#"(print (default $2 (s/upper "unknown")))"#,
            "Alice Tokyo\nBob\nCarol \n"
        ),
        "Tokyo\nUNKNOWN\nUNKNOWN\n"
    );
    assert_eq!(
        output(r#"(print (default "" "fallback"))"#, "x\n"),
        "fallback\n"
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
            r#"(filter (or (s/= $1 "Alice") (s/= $1 "Bob"))) (print $1)"#,
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
fn supports_all_string_comparison_operators() {
    assert_eq!(
        output(r#"(print (if (s/< "10" "2") "yes" "no"))"#, "x\n"),
        "yes\n"
    );
    assert_eq!(
        output(r#"(print (if (s/<= "a" "a") "yes" "no"))"#, "x\n"),
        "yes\n"
    );
    assert_eq!(
        output(r#"(print (if (s/> "b" "a") "yes" "no"))"#, "x\n"),
        "yes\n"
    );
    assert_eq!(
        output(r#"(print (if (s/>= "b" "b") "yes" "no"))"#, "x\n"),
        "yes\n"
    );
    assert_eq!(
        output(r#"(print (if (s/!= "a" "b") "yes" "no"))"#, "x\n"),
        "yes\n"
    );
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
                r#"(print (n/fixed 2 $1) (n/fixed 3 $2) (n/fixed 0 $2) "#,
                r#"(str "$" (n/fixed (+ 1 1) (* $1 2))))"#,
            ),
            "3 3.14159\n-2.5 1.25\n",
        ),
        "3.00 3.142 3 $6.00\n-2.50 1.250 1 $-5.00\n"
    );
}

#[test]
fn fixed_number_formatting_rejects_invalid_digits_and_values() {
    for (program, expected) in [
        (
            "(print (n/fixed -1 $1))",
            "record 1: n/fixed: argument 1 expects Number (whole digits from 0 to 100)",
        ),
        (
            "(print (n/fixed 1.5 $1))",
            "record 1: n/fixed: argument 1 expects Number (whole digits from 0 to 100)",
        ),
        (
            "(print (n/fixed 101 $1))",
            "record 1: n/fixed: argument 1 expects Number (whole digits from 0 to 100)",
        ),
        (
            "(print (n/fixed 2 $2))",
            "record 1: n/fixed: argument 2 expects Number",
        ),
        (
            r#"(print (n/fixed "" $1))"#,
            "record 1: n/fixed: argument 1 expects Number",
        ),
    ] {
        let error = cho::run(program, Cursor::new("3\n"), Vec::new()).unwrap_err();
        assert!(error.to_string().starts_with(expected), "{error}");
    }
}

#[test]
fn url_component_extraction_preserves_encoding_and_composes() {
    assert_eq!(
        output(
            concat!(
                r#"(print (url/scheme $1) (url/host $1) (url/port $1) "#,
                r#"(url/path $1) (url/query $1) (url/fragment $1)) "#,
                r#"(print (str "host=" (s/upper (url/host $1))) (-> $1 (url/path)))"#,
            ),
            "https://example.com:8443/a%20b?q=hello%20world#top\n",
        ),
        concat!(
            "https example.com 8443 /a%20b q=hello%20world top\n",
            "host=EXAMPLE.COM /a%20b\n",
        )
    );
}

#[test]
fn url_component_extraction_returns_empty_for_missing_optional_parts() {
    assert_eq!(
        output(
            concat!(
                r#"(print (s/join "|" (url/host $1) (url/port $1) "#,
                r#"(url/path $1) (url/query $1) (url/fragment $1)))"#,
            ),
            "https://example.com\nhttps://[::1]/\nmailto:alice@example.com\n",
        ),
        "example.com||/||\n[::1]||/||\n||alice@example.com||\n"
    );
}

#[test]
fn url_component_extraction_rejects_invalid_urls_and_non_strings() {
    for (program, expected) in [
        (
            "(print (url/host $1))",
            "record 1: url/host: argument 1 expects Url (absolute URL)",
        ),
        (
            "(print (url/path 10))",
            "record 1: url/path: argument 1 expects String",
        ),
        (
            r#"(print (url/query ""))"#,
            "record 1: url/query: argument 1 expects Url (absolute URL)",
        ),
    ] {
        let error = cho::run(program, Cursor::new("relative/path\n"), Vec::new()).unwrap_err();
        assert!(error.to_string().starts_with(expected), "{error}");
    }
}

#[test]
fn url_component_encoding_handles_ascii_unicode_and_composition() {
    assert_eq!(
        output(
            concat!(
                r#"(print (url/encode "hello world") (url/decode "hello%20world") "#,
                r#"(str "https://example.com/search?q=" (url/encode $1)) "#,
                r#"(url/encode $2) (-> $2 (url/encode) (url/decode)))"#,
            ),
            "東京 a/b?q=1+2\n",
        ),
        concat!(
            "hello%20world hello world ",
            "https://example.com/search?q=%E6%9D%B1%E4%BA%AC ",
            "a%2Fb%3Fq%3D1%2B2 a/b?q=1+2\n",
        )
    );
    assert_eq!(
        output(
            r#"(print (s/join "|" (url/encode "") (url/decode "") (url/decode "%2b+%2F")))"#,
            "x\n",
        ),
        "||++/\n"
    );
}

#[test]
fn url_component_decoding_rejects_invalid_escapes_utf8_and_non_strings() {
    for (program, expected) in [
        (
            r#"(print (url/decode "%"))"#,
            "record 1: url/decode: argument 1 expects String (URL component)",
        ),
        (
            r#"(print (url/decode "%2"))"#,
            "record 1: url/decode: argument 1 expects String (URL component)",
        ),
        (
            r#"(print (url/decode "%GG"))"#,
            "record 1: url/decode: argument 1 expects String (URL component)",
        ),
        (
            r#"(print (url/decode "%FF"))"#,
            "record 1: url/decode: argument 1 expects String (URL component)",
        ),
        (
            "(print (url/encode 10))",
            "record 1: url/encode: argument 1 expects String",
        ),
    ] {
        let error = cho::run(program, Cursor::new("x\n"), Vec::new()).unwrap_err();
        assert!(error.to_string().starts_with(expected), "{error}");
    }
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
fn invalid_regexes_in_if_fail_before_processing() {
    let error = cho::run(
        r#"(print (if (reg "[") "yes" "no"))"#,
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
        "(print NF (s/escape $2))",
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

#[test]
fn default_recovers_from_runtime_errors_and_is_lazy() {
    assert_eq!(
        output(
            r#"(print (default (if (> $1 0) "positive" "zero") "invalid"))"#,
            "2\nunknown\n"
        ),
        "positive\ninvalid\n"
    );
    assert_eq!(
        output(r#"(print (default "ok" (dt/fmt "%Q" "invalid")))"#, "x\n"),
        "ok\n"
    );
}

#[test]
fn output_before_a_runtime_error_is_preserved() {
    let mut result = Vec::new();
    let error = cho::run(
        "(filter (> $2 20)) (print $1)",
        Cursor::new("Alice 30\nBob unknown\n"),
        &mut result,
    )
    .unwrap_err();
    assert_eq!(String::from_utf8(result).unwrap(), "Alice\n");
    assert!(error.to_string().starts_with("record 2: >: argument 1"));
}

#[test]
fn datetime_values_normalize_format_and_compare() {
    assert_eq!(
        output(
            r#"(print (dt/fmt "%Y/%m/%d %H:%M:%S" $1) (dt/unix -1))"#,
            "2026-08-18T12:34:56.120+09:00\n"
        ),
        "2026/08/18 03:34:56 1969-12-31T23:59:59Z\n"
    );
    assert_eq!(
        output(
            r#"(filter (dt/= $1 "2026-08-18T03:00:00Z")) (print $1)"#,
            "2026-08-18T12:00:00+09:00\n2026-08-18T04:00:00Z\n"
        ),
        "2026-08-18T12:00:00+09:00\n"
    );
}

#[test]
fn typed_values_render_inside_string_operations() {
    assert_eq!(
        output(
            r#"(print (str "at=" (dt/unix 0)) (s/join "," (dt/unix 0) (du/s 1.5)) (s/count (dt/unix 0)))"#,
            "x\n"
        ),
        "at=1970-01-01T00:00:00Z 1970-01-01T00:00:00Z,1.5 20\n"
    );
}

#[test]
fn supports_all_datetime_comparison_operators() {
    let early = "2026-08-18T00:00:00Z";
    let late = "2026-08-18T00:00:01Z";
    for predicate in [
        format!(r#"(dt/< "{early}" "{late}")"#),
        format!(r#"(dt/<= "{early}" "{early}")"#),
        format!(r#"(dt/> "{late}" "{early}")"#),
        format!(r#"(dt/>= "{late}" "{late}")"#),
        format!(r#"(dt/!= "{early}" "{late}")"#),
    ] {
        assert_eq!(
            output(&format!(r#"(print (if {predicate} "yes" "no"))"#), "x\n"),
            "yes\n"
        );
    }
}

#[test]
fn datetime_values_floor_to_utc_boundaries() {
    assert_eq!(
        output(
            r#"(print (dt/floor-s $1) (dt/floor-m $1) (dt/floor-h $1) (dt/floor-d $1))"#,
            "2026-08-18T12:34:56.789+09:00\n"
        ),
        concat!(
            "2026-08-18T03:34:56Z ",
            "2026-08-18T03:34:00Z ",
            "2026-08-18T03:00:00Z ",
            "2026-08-18T00:00:00Z\n"
        )
    );
    let now = output(r#"(print (dt/now))"#, "x\n");
    assert!(now.ends_with("Z\n"));
    assert!(!now.contains('.'));
}

#[test]
fn datetime_floor_reports_its_own_argument_error() {
    let error = cho::run(
        "(print (dt/floor-m $1))",
        Cursor::new("invalid\n"),
        Vec::new(),
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .starts_with("record 1: dt/floor-m: argument 1")
    );
}

#[test]
fn datetime_errors_identify_the_argument() {
    let error = cho::run(
        r#"(print (dt/fmt "%Y" $1))"#,
        Cursor::new("2026-08-18\n"),
        Vec::new(),
    )
    .unwrap_err();
    assert_eq!(
        error.to_string(),
        r#"record 1: dt/fmt: argument 2 expects DateTime, but "2026-08-18" is not valid RFC 3339"#
    );
    assert!(cho::run("(print (dt/unix 1.5))", Cursor::new("x\n"), Vec::new()).is_err());
    assert!(
        cho::run(
            r#"(print (dt/diff "9999-12-31T23:59:59Z" "0000-01-01T00:00:00Z"))"#,
            Cursor::new("x\n"),
            Vec::new()
        )
        .is_err()
    );
}

#[test]
fn duration_values_support_fractional_arithmetic_and_differences() {
    assert_eq!(output("(print (du/h 1) (du/s 0.25))", "x\n"), "3600 0.25\n");
    assert_eq!(
        output(
            r#"(print (dt/add $1 (du/s 10.5)) (dt/sub $1 $2))"#,
            "2026-08-18T00:00:00Z 60\n"
        ),
        "2026-08-18T00:00:10.500Z 2026-08-17T23:59:00Z\n"
    );
    assert_eq!(
        output(
            r#"(print (dt/diff $1 $2))"#,
            "2026-08-18T00:00:00.500Z 2026-08-18T00:00:01Z\n"
        ),
        "-0.5\n"
    );
    assert_eq!(
        output(
            "(print (if (dt/= (dt/now) (dt/now)) \"same\" \"different\"))",
            "x\n"
        ),
        "same\n"
    );
}

#[test]
fn ip_and_cidr_predicates_are_typed() {
    assert_eq!(
        output(
            "(filter (ip/private? $1)) (print $1)",
            "10.1.2.3\n8.8.8.8\nfc00::1\n"
        ),
        "10.1.2.3\n"
    );
    assert_eq!(
        output(
            r#"(filter (cidr/contains? "10.0.0.0/8" $1)) (print $1)"#,
            "10.2.3.4\n11.0.0.1\n2001:db8::1\n"
        ),
        "10.2.3.4\n"
    );
    assert_eq!(
        output(
            r#"(filter (ip/= $1 "2001:db8::1")) (print $1)"#,
            "2001:0db8:0:0:0:0:0:1\n2001:db8::2\n"
        ),
        "2001:0db8:0:0:0:0:0:1\n"
    );
}

#[test]
fn ip_classification_predicates_cover_ipv4_and_ipv6_boundaries() {
    assert_eq!(
        output(
            "(filter (ip/loopback? $1)) (print $1)",
            "127.0.0.1\n127.255.255.255\n126.255.255.255\n::1\n::2\n",
        ),
        "127.0.0.1\n127.255.255.255\n::1\n"
    );
    assert_eq!(
        output(
            "(filter (ip/link-local? $1)) (print $1)",
            concat!(
                "169.254.0.1\n169.254.255.255\n169.253.255.255\n",
                "fe80::1\nfebf::1\nfec0::1\n",
            ),
        ),
        "169.254.0.1\n169.254.255.255\nfe80::1\nfebf::1\n"
    );
    assert_eq!(
        output(
            "(filter (ip/multicast? $1)) (print $1)",
            concat!(
                "224.0.0.1\n239.255.255.255\n223.255.255.255\n240.0.0.1\n",
                "ff02::1\nfeff::1\n",
            ),
        ),
        "224.0.0.1\n239.255.255.255\nff02::1\n"
    );
}

#[test]
fn ip_classification_predicates_report_their_own_conversion_errors() {
    for predicate in ["ip/loopback?", "ip/link-local?", "ip/multicast?"] {
        let program = format!("(filter ({predicate} $1))");
        let error = cho::run(&program, Cursor::new("not-an-ip\n"), Vec::new()).unwrap_err();
        assert!(
            error
                .to_string()
                .starts_with(&format!("record 1: {predicate}: argument 1 expects IpAddr")),
            "{error}"
        );
    }
}

#[test]
fn semver_comparisons_follow_precedence_including_prereleases() {
    assert_eq!(
        output(
            r#"(filter (semver/> $1 $2)) (print $1 $2)"#,
            concat!(
                "1.10.0 1.9.0\n",
                "1.0.0 1.0.0-rc.1\n",
                "1.0.0-alpha.10 1.0.0-alpha.2\n",
                "1.0.0-alpha 1.0.0\n",
            ),
        ),
        "1.10.0 1.9.0\n1.0.0 1.0.0-rc.1\n1.0.0-alpha.10 1.0.0-alpha.2\n"
    );
    assert_eq!(
        output(
            concat!(
                r#"(print (if (semver/= "1.0.0+linux" "1.0.0+darwin") "same" "different") "#,
                r#"(if (s/= "1.0.0+linux" "1.0.0+darwin") "same" "different"))"#,
            ),
            "x\n",
        ),
        "same different\n"
    );
}

#[test]
fn semver_supports_all_comparison_operators() {
    for (operator, left, right, expected) in [
        ("<", "1.0.0-alpha", "1.0.0", "yes\n"),
        ("<=", "1.0.0", "1.0.0", "yes\n"),
        (">", "2.0.0", "1.9.9", "yes\n"),
        (">=", "2.0.0", "2.0.0", "yes\n"),
        ("=", "1.0.0+one", "1.0.0+two", "yes\n"),
        ("!=", "1.0.1", "1.0.0", "yes\n"),
    ] {
        let program = format!(r#"(print (if (semver/{operator} "{left}" "{right}") "yes" "no"))"#);
        assert_eq!(output(&program, "x\n"), expected);
    }
}

#[test]
fn semver_comparisons_reject_non_strict_or_invalid_versions() {
    for value in ["1.2", "v1.2.3", "1.2.3.4", "1.0.0-01", ""] {
        let error = cho::run(
            r#"(filter (semver/>= $1 "1.0.0"))"#,
            Cursor::new(format!("{value}\n")),
            Vec::new(),
        )
        .unwrap_err();
        assert!(
            error
                .to_string()
                .starts_with("record 1: semver/>=: argument 1 expects SemVer"),
            "{error}"
        );
    }
}

#[test]
fn predicates_render_as_boolean_values_and_compose() {
    assert_eq!(
        output(
            concat!(
                r#"(print (semver/> $1 $2) (> $3 $4) "#,
                r#"(str "loopback=" (ip/loopback? $5)) "#,
                r#"(s/join ":" (s/= $6 "yes") (not (s/= $6 "yes"))) "#,
                r#"(default (semver/= $1 $2) "fallback"))"#,
            ),
            "1.2.3 2.0.0 10 2 127.0.0.1 yes\n2.0.0 1.2.3 1 2 8.8.8.8 no\n",
        ),
        concat!(
            "false true loopback=true true:false false\n",
            "true false loopback=false false:true false\n",
        )
    );
}

#[test]
fn boolean_values_do_not_convert_to_numbers_or_strings() {
    for program in ["(print (+ (> 2 1) 1))", r#"(print (url/encode (> 2 1)))"#] {
        let error = cho::run(program, Cursor::new("x\n"), Vec::new()).unwrap_err();
        assert!(error.to_string().contains("has type Boolean"), "{error}");
    }
}

#[test]
fn default_can_recover_from_an_invalid_ip() {
    assert_eq!(
        output(
            r#"(print (default (if (ip/private? $1) "private" "public") "invalid"))"#,
            "10.0.0.1\nnot-an-ip\n2001:db8::1\n"
        ),
        "private\ninvalid\npublic\n"
    );
}

#[test]
fn threading_runs_as_the_expanded_value_expression() {
    assert_eq!(
        output(
            r#"(print (->> $1 (dt/fmt "%Y/%m/%d") (str "date: ")))"#,
            "2026-08-18T00:00:00Z\n"
        ),
        "date: 2026/08/18\n"
    );
}
