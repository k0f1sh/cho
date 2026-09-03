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
