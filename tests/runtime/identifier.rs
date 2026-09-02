use std::collections::HashSet;
use std::io::Cursor;

use ulid::Ulid;
use uuid::Uuid;

use super::support::output;

#[test]
fn uuid_and_ulid_normalize_common_text_forms() {
    assert_eq!(
        output(
            "(print (uuid $1) (uuid $2) (ulid $3))",
            concat!(
                "A1A2A3A4B1B2C1C2D1D2D3D4D5D6D7D8 ",
                "urn:uuid:A1A2A3A4-B1B2-C1C2-D1D2-D3D4D5D6D7D8 ",
                "01d39zy06fgsctvn4t2v9pkhfz\n",
            ),
        ),
        concat!(
            "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8 ",
            "a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8 ",
            "01D39ZY06FGSCTVN4T2V9PKHFZ\n",
        )
    );
    assert_eq!(
        output(
            r#"(print (s/upper (uuid $1)) (s/lower (ulid $2)))"#,
            "550e8400-e29b-41d4-a716-446655440000 01D39ZY06FGSCTVN4T2V9PKHFZ\n",
        ),
        "550E8400-E29B-41D4-A716-446655440000 01d39zy06fgsctvn4t2v9pkhfz\n"
    );
}

#[test]
fn generated_identifiers_have_the_requested_versions_and_increase() {
    let input = (1..=100)
        .map(|number| format!("{number}\n"))
        .collect::<String>();
    let v4_output = output("(uuid/v4)", &input);
    let v4 = v4_output
        .lines()
        .map(|value| Uuid::try_parse(value).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(v4.len(), 100);
    assert!(v4.iter().all(|uuid| uuid.get_version_num() == 4));
    assert_eq!(v4.iter().collect::<HashSet<_>>().len(), 100);

    let v7_output = output("(uuid/v7)", &input);
    let v7 = v7_output
        .lines()
        .map(|value| Uuid::try_parse(value).unwrap())
        .collect::<Vec<_>>();
    assert!(v7.iter().all(|uuid| uuid.get_version_num() == 7));
    assert!(v7.windows(2).all(|pair| pair[0] < pair[1]));

    let ulid_output = output("(ulid/new)", &input);
    let ulids = ulid_output
        .lines()
        .map(|value| value.parse::<Ulid>().unwrap())
        .collect::<Vec<_>>();
    assert!(ulids.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn identifier_versions_and_times_are_typed_values() {
    assert_eq!(
        output(
            concat!(
                "(print (uuid/version $1) (uuid/version $2) ",
                "(uuid/time $2) (ulid/time $3))",
            ),
            concat!(
                "550e8400-e29b-41d4-a716-446655440000 ",
                "00000000-0000-7000-8000-000000000000 ",
                "00000000000000000000000001\n",
            ),
        ),
        "4 7 1970-01-01T00:00:00Z 1970-01-01T00:00:00Z\n"
    );
    assert_eq!(
        output(
            "(print (uuid/time $1) (uuid/time $2) (uuid/time $3))",
            concat!(
                "f81d4fae-7dec-11d0-a765-00a0c91e6bf6 ",
                "1d07decf-81d4-6fae-a765-00a0c91e6bf6 ",
                "00000000-0000-1000-8000-000000000000\n",
            ),
        ),
        concat!(
            "1997-02-03T17:43:12.216875Z ",
            "1997-02-03T17:43:12.216875Z ",
            "1582-10-15T00:00:00Z\n",
        )
    );
}

#[test]
fn uuid_and_ulid_support_all_comparisons() {
    for prefix in ["uuid", "ulid"] {
        let (low, high) = if prefix == "uuid" {
            (
                "00000000-0000-0000-0000-000000000001",
                "00000000-0000-0000-0000-000000000002",
            )
        } else {
            ("00000000000000000000000001", "00000000000000000000000002")
        };
        for (operator, left, right, expected) in [
            ("<", low, high, "true\n"),
            ("<=", low, low, "true\n"),
            (">", high, low, "true\n"),
            (">=", high, high, "true\n"),
            ("=", low, low, "true\n"),
            ("!=", low, high, "true\n"),
        ] {
            assert_eq!(
                output(
                    &format!("(print ({prefix}/{operator} $1 $2))"),
                    &format!("{left} {right}\n")
                ),
                expected
            );
        }
    }
}

#[test]
fn identifier_errors_name_the_function_argument_and_type() {
    for (program, input, expected) in [
        ("(uuid $1)", "nope\n", "uuid: argument 1 expects UUID"),
        ("(uuid $1)", "\n", "uuid: argument 1 expects UUID"),
        ("(uuid true)", "record\n", "uuid: argument 1 expects UUID"),
        ("(ulid $1)", "nope\n", "ulid: argument 1 expects ULID"),
        ("(ulid $1)", "\n", "ulid: argument 1 expects ULID"),
        ("(ulid true)", "record\n", "ulid: argument 1 expects ULID"),
        (
            "(ulid $1)",
            "80000000000000000000000000\n",
            "ulid: argument 1 expects ULID",
        ),
        (
            "(uuid/time $1)",
            "550e8400-e29b-41d4-a716-446655440000\n",
            "uuid/time: argument 1 expects UUID version 1, 6, or 7",
        ),
        (
            "(uuid/= $1 $2)",
            "550e8400-e29b-41d4-a716-446655440000 nope\n",
            "uuid/=: argument 2 expects UUID",
        ),
    ] {
        let error = cho::run(program, Cursor::new(input), Vec::new()).unwrap_err();
        assert!(error.to_string().contains(expected), "{error}");
    }
    assert_eq!(
        output(
            r#"(print (default (uuid/time $1) "none"))"#,
            "550e8400-e29b-41d4-a716-446655440000\n",
        ),
        "none\n"
    );
}
