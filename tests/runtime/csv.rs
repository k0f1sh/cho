use super::support::output;
use std::io::Cursor;

#[test]
fn csv_join_quotes_special_fields_and_preserves_record_data() {
    assert_eq!(
        output(
            "(csv/join $1 (str $2 \", Japan\") \"say \\\"hi\\\"\" \"line1\\nline2\" \"a\rb\")",
            "Alice Tokyo\n",
        ),
        "Alice,\"Tokyo, Japan\",\"say \"\"hi\"\"\",\"line1\nline2\",\"a\rb\"\n"
    );
}

#[test]
fn csv_join_round_trips_decoded_csv_fields() {
    let input = "Alice,\"Tokyo, Japan\",\"line1\nline2\"\nBob,Osaka,plain\n";
    let mut result = Vec::new();
    cho::run_csv("(csv/join $1 $2 $3)", Cursor::new(input), &mut result).unwrap();
    assert_eq!(String::from_utf8(result).unwrap(), input);
}

#[test]
fn csv_join_handles_empty_and_multiple_values_and_composes() {
    assert_eq!(
        output(
            concat!(
                r#"(print (csv/join)) (print (csv/join "")) "#,
                r#"(print (csv/join "" "x" "")) "#,
                r#"(print (str "row=" (csv/join (+ 1 1) true)))"#,
            ),
            "ignored\n",
        ),
        "\n\"\"\n,x,\nrow=2,true\n"
    );
}

#[test]
fn csv_header_fields_are_resolved_and_the_header_is_skipped() {
    let input = "name,age,display name\nAlice,20,Alice A.\nBob,30,Bob B.\n";
    let mut result = Vec::new();
    cho::run_csv(
        r#"(f (> %age 20)) (print NR %name %"display name" $2 NF)"#,
        Cursor::new(input),
        &mut result,
    )
    .unwrap();
    assert_eq!(String::from_utf8(result).unwrap(), "3 Bob Bob B. 30 3\n");
}

#[test]
fn csv_header_fields_support_decoded_names_and_missing_row_values() {
    let input = "\"full,name\",\"line\nname\",\"\"\nAlice,first\n";
    let mut result = Vec::new();
    cho::run_csv(
        r#"(print %"full,name" %"line\nname" %"")"#,
        Cursor::new(input),
        &mut result,
    )
    .unwrap();
    assert_eq!(String::from_utf8(result).unwrap(), "Alice first \n");
}

#[test]
fn csv_header_field_errors_precede_record_output() {
    for (program, input, message) in [
        (
            "(print %missing)",
            "name,age\nAlice,20\n",
            "CSV header has no field named \"missing\"",
        ),
        (
            "(print %name)",
            "name,name\nAlice,Alicia\n",
            "CSV header has multiple fields named \"name\"",
        ),
        (
            "(print %name)",
            "",
            "CSV header is required for header field references",
        ),
    ] {
        let mut result = Vec::new();
        let error = cho::run_csv(program, Cursor::new(input), &mut result).unwrap_err();
        assert_eq!(error.to_string(), message);
        assert!(result.is_empty());
    }
}

#[test]
fn csv_header_only_input_and_unreferenced_duplicate_names_are_valid() {
    let mut result = Vec::new();
    cho::run_csv(
        "(print %name)",
        Cursor::new("name,unused,unused\n"),
        &mut result,
    )
    .unwrap();
    assert!(result.is_empty());
}

#[test]
fn header_field_references_require_csv_input() {
    let error = cho::run("(print %name)", Cursor::new("Alice\n"), Vec::new()).unwrap_err();
    assert_eq!(error.to_string(), "header field references require --csv");

    let error = cho::run_no_input("(print %name)", Vec::new()).unwrap_err();
    assert_eq!(error.to_string(), "header field references require --csv");
}
