use super::support::{output, output_with_separator};
use std::io::{self, Cursor};

#[test]
fn field_zero_preserves_the_line_and_missing_fields_are_empty() {
    assert_eq!(
        output("(print $0 $3)", "  Alice   20  \n"),
        "  Alice   20   \n"
    );
}

#[test]
fn field_ranges_preserve_original_whitespace_and_compose_as_values() {
    assert_eq!(
        output(
            "(print $..2) (print $3..) (print (s/upper $2..4))",
            "  one\t two   three four  five  \n",
        ),
        "  one\t two\nthree four  five  \nTWO   THREE FOUR\n"
    );
}

#[test]
fn open_field_ranges_extend_to_record_edges() {
    assert_eq!(
        output(
            "(print (dq $..2) (dq $2..) (dq $1..))",
            "  one  two  three  \n"
        ),
        "\"  one  two\" \"two  three  \" \"one  two  three  \"\n"
    );
}

#[test]
fn field_range_ends_are_clamped_and_missing_starts_are_empty() {
    assert_eq!(
        output("(print $3..) (print $..3) (print $2..4)", "one two\n\n"),
        "\none two\ntwo\n\n\n\n"
    );
    assert_eq!(
        output(
            "(print (dq $..4000) (dq $1..30) (dq $30..40))",
            "  a b   c ddd    eeee  \n"
        ),
        "\"  a b   c ddd    eeee\" \"a b   c ddd    eeee\" \"\"\n"
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
fn computed_field_numbers_compose_with_nf_and_other_values() {
    assert_eq!(
        output(
            "(print (field NF) (s/upper (field (- NF 1))) (field (+ NF 1)))",
            "one two three four\nsolo\n",
        ),
        "four THREE \nsolo SOLO \n"
    );
}

#[test]
fn computed_field_zero_is_the_complete_record() {
    assert_eq!(
        output("(print (dq (field 0)))", "  one two  \n"),
        "\"  one two  \"\n"
    );
}

#[test]
fn computed_field_numbers_must_be_non_negative_whole_numbers() {
    for number in ["-1", "1.5"] {
        let error = cho::run(
            &format!("(print (field {number}))"),
            Cursor::new("one two\n"),
            Vec::new(),
        )
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("is not a non-negative whole number")
        );
    }
}

#[test]
fn computed_field_ranges_preserve_record_edges_and_compose_as_values() {
    assert_eq!(
        output(
            "(print (dq (fields-to (- NF 2)))) (print (dq (fields-from 3))) (print (s/upper (fields 2 (- NF 1))))",
            "  one\t two   three four  five  \n",
        ),
        "\"  one\\t two   three\"\n\"three four  five  \"\nTWO   THREE FOUR\n"
    );
}

#[test]
fn computed_field_ranges_handle_empty_and_missing_fields() {
    assert_eq!(
        output(
            "(print (dq (fields-to 3)) (dq (fields-from 3)) (dq (fields 2 4)))",
            "one two\n\n",
        ),
        "\"one two\" \"\" \"two\"\n\"\" \"\" \"\"\n"
    );
}

#[test]
fn computed_field_range_bounds_must_be_positive_ordered_whole_numbers() {
    for program in [
        "(print (fields-from 0))",
        "(print (fields-to 1.5))",
        "(print (fields 3 2))",
    ] {
        assert!(cho::run(program, Cursor::new("one two three\n"), Vec::new()).is_err());
    }
}

#[test]
fn computed_fields_are_available_for_decoded_csv_fields() {
    let mut result = Vec::new();
    cho::run_csv(
        "(print (field (- NF 1)))",
        Cursor::new("Alice,\"Tokyo, Japan\",active\n"),
        &mut result,
    )
    .unwrap();
    assert_eq!(String::from_utf8(result).unwrap(), "Tokyo, Japan\n");
}

#[test]
fn computed_field_ranges_are_unavailable_for_decoded_csv_fields() {
    for program in ["(fields 1 2)", "(fields-from 2)", "(fields-to 2)"] {
        let error = cho::run_csv(program, Cursor::new("one,two,three\n"), Vec::new()).unwrap_err();
        assert_eq!(
            error.to_string(),
            "field ranges are not supported with --csv"
        );
    }
}

#[test]
fn an_empty_print_prints_an_empty_line() {
    assert_eq!(output("(print)", "Alice\nBob\n"), "\n\n");
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
fn empty_program_passes_records_through_unchanged() {
    assert_eq!(
        output("", "  Alice 20  \n\nBob\t30\n"),
        "  Alice 20  \n\nBob\t30\n"
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
fn custom_separators_control_fields_and_nf() {
    assert_eq!(
        output_with_separator("(print NF $1 $3 $0)", ",", "Alice,20,Tokyo\nBob,30,Osaka\n"),
        "3 Alice Tokyo Alice,20,Tokyo\n3 Bob Osaka Bob,30,Osaka\n"
    );
}

#[test]
fn empty_records_have_no_fields() {
    assert_eq!(output_with_separator("(print NF)", ",", "\n"), "0\n");
}

#[test]
fn csv_mode_rejects_field_ranges_before_reading_input() {
    let error = cho::run_csv(
        "(print (s/upper $2..))",
        Cursor::new("not,csv,\""),
        Vec::new(),
    )
    .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    assert_eq!(
        error.to_string(),
        "field ranges are not supported with --csv"
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
fn csv_mode_preserves_a_final_record_without_a_newline() {
    let mut result = Vec::new();
    cho::run_csv("(print $0 $2)", Cursor::new("Alice,Tokyo"), &mut result).unwrap();
    assert_eq!(String::from_utf8(result).unwrap(), "Alice,Tokyo Tokyo\n");
}

#[test]
fn csv_mode_rejects_bare_carriage_returns_without_losing_data_silently() {
    let mut output = Vec::new();
    let error = cho::run_csv(
        "(print NR NF $1 $2)",
        Cursor::new("first,valid\nsecond\rthird,value\n"),
        &mut output,
    )
    .unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert_eq!(
        error.to_string(),
        "CSV record 2, line 2, field 1: bare carriage return is not allowed outside a quoted field"
    );
    assert_eq!(String::from_utf8(output).unwrap(), "1 2 first valid\n");
}

#[test]
fn csv_mode_accepts_carriage_returns_inside_quotes() {
    let mut output = Vec::new();
    cho::run_csv(
        "(print NR NF $1 (s/escape $2))",
        Cursor::new("a,\"b\rc\"\r\nd,e"),
        &mut output,
    )
    .unwrap();

    assert_eq!(String::from_utf8(output).unwrap(), "1 2 a b\\rc\n2 2 d e\n");
}

#[test]
fn csv_mode_rejects_invalid_quotes_with_record_line_and_field() {
    for (input, prior_output, message) in [
        (
            "name,value\nfirst,valid\nsecond,\"value\"trailing\n",
            "name value\nfirst valid\n",
            "CSV record 3, line 3, field 2: expected a comma or end of record after closing quote",
        ),
        (
            "name,value\nfirst,un\"quoted\n",
            "name value\n",
            "CSV record 2, line 2, field 2: quote is only allowed at the start of a field",
        ),
        (
            "name,value\nfirst,\"line one\nline two",
            "name value\n",
            "CSV record 2, line 3, field 2: quoted field is not closed before end of input",
        ),
    ] {
        let mut output = Vec::new();
        let error = cho::run_csv("(print $1 $2)", Cursor::new(input), &mut output).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(error.to_string(), message);
        assert_eq!(String::from_utf8(output).unwrap(), prior_output);
    }
}

#[test]
fn csv_mode_accepts_escaped_quotes_and_multiline_fields() {
    let mut output = Vec::new();
    cho::run_csv(
        "(print $1 (s/escape $2) $3)",
        Cursor::new("first,\"line one\nline \"\"two\"\"\",last\r\n"),
        &mut output,
    )
    .unwrap();
    assert_eq!(
        String::from_utf8(output).unwrap(),
        "first line one\\nline \"two\" last\n"
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
