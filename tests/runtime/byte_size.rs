use super::support::output;
use std::io::Cursor;

#[test]
fn byte_sizes_normalize_decimal_and_binary_units_exactly() {
    assert_eq!(
        output(
            "(print (bs $1) (bs $2) (bs $3) (bs $4) (bs $5) (bs $6))",
            "1kB 1KiB 2.11GB 2.352MiB 0.1KiB 0B\n",
        ),
        "1000B 1024B 2110000000B 2466250.752B 102.4B 0B\n"
    );
    assert_eq!(
        output(
            concat!(
                "(print (bs \"1 MB\") (bs \"1\\tGiB\") (bs \"1PB\") ",
                "(bs \"1PiB\") (bs (bs \"1kB\")))",
            ),
            "ignored\n",
        ),
        "1000000B 1073741824B 1000000000000000B 1125899906842624B 1000B\n"
    );
    assert_eq!(
        output(
            concat!(
                "(print (bs \"1B\") (bs \"1kB\") (bs \"1MB\") (bs \"1GB\") ",
                "(bs \"1TB\") (bs \"1PB\") (bs \"1KiB\") (bs \"1MiB\") ",
                "(bs \"1GiB\") (bs \"1TiB\") (bs \"1PiB\"))",
            ),
            "ignored\n",
        ),
        concat!(
            "1B 1000B 1000000B 1000000000B 1000000000000B 1000000000000000B ",
            "1024B 1048576B 1073741824B 1099511627776B 1125899906842624B\n",
        )
    );
}

#[test]
fn byte_size_comparisons_are_exact_across_units_and_compose() {
    assert_eq!(
        output(
            concat!(
                "(print (bs/= \"1kB\" \"1000B\") (bs/!= \"1kB\" \"1KiB\") ",
                "(bs/< \"999MB\" \"1GB\") (bs/<= \"1024MiB\" \"1GiB\") ",
                "(bs/> \"2.11GB\" \"2048MiB\") (bs/>= \"1PiB\" \"1PB\"))",
            ),
            "ignored\n",
        ),
        "true true true true false true\n"
    );
    assert_eq!(
        output(
            r#"(filter (bs/> $2 "500MB")) (print (str $1 "=" (bs $2)))"#,
            "small 470MB\nlarge 2.11GB\n",
        ),
        "large=2110000000B\n"
    );
}

#[test]
fn byte_size_to_bytes_obeys_the_number_safe_integer_boundary() {
    assert_eq!(
        output(
            "(print (bs/to-b \"2.352MiB\") (bs/to-b \"9007199254740991B\"))",
            "ignored\n",
        ),
        "2466250.752 9007199254740991\n"
    );
    let error = cho::run(
        "(print (bs/to-b \"9007199254740992B\"))",
        Cursor::new("ignored\n"),
        Vec::new(),
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("bs/to-b: argument 1 expects ByteSize no greater than 2^53 - 1 B"),
        "{error}"
    );
}

#[test]
fn byte_sizes_reject_invalid_syntax_ranges_and_runtime_types() {
    for input in [
        "",
        "500",
        "500mb",
        "500M",
        "-1GB",
        "+1GB",
        "1e3MB",
        "1,000MB",
        ".5MB",
        "1.MB",
        " 1MB",
        "1MB ",
        "1\nMB",
        "79228162514264337593543950335PB",
        "12345678901234.12345678901234PiB",
    ] {
        let program = format!("(print (bs {:?}))", input);
        let error = cho::run(&program, Cursor::new("ignored\n"), Vec::new()).unwrap_err();
        assert!(
            error
                .to_string()
                .starts_with("record 1: bs: argument 1 expects ByteSize"),
            "input {input:?}: {error}"
        );
    }

    let missing = cho::run("(print (bs $2))", Cursor::new("only\n"), Vec::new()).unwrap_err();
    assert!(
        missing.to_string().contains("expects ByteSize"),
        "{missing}"
    );
    let typed = cho::run("(print (bs 10))", Cursor::new("x\n"), Vec::new()).unwrap_err();
    assert!(
        typed.to_string().contains("but \"10\" has type Number"),
        "{typed}"
    );
    let second = cho::run(
        "(print (bs/> \"1MB\" \"bad\"))",
        Cursor::new("x\n"),
        Vec::new(),
    )
    .unwrap_err();
    assert!(
        second
            .to_string()
            .contains("bs/>: argument 2 expects ByteSize"),
        "{second}"
    );
}

#[test]
fn default_recovers_from_invalid_byte_sizes() {
    assert_eq!(
        output(
            r#"(print (default (bs $1) "invalid"))"#,
            "2.11GB\nunknown\n",
        ),
        "2110000000B\ninvalid\n"
    );
}
