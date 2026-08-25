use super::support::output;
use std::io::Cursor;

#[test]
fn datetime_values_normalize_format_and_compare() {
    assert_eq!(
        output(
            r#"(print (dt/fmt $1 "%Y/%m/%d %H:%M:%S") (dt/unix -1))"#,
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
fn datetime_format_reports_timezone_and_datetime_arguments() {
    let error = cho::run(
        r#"(print (dt/fmt $2 "%Y" $1))"#,
        Cursor::new("Unknown/Zone 2026-08-18T00:00:00Z\n"),
        Vec::new(),
    )
    .unwrap_err();
    assert_eq!(
        error.to_string(),
        r#"record 1: dt/fmt: argument 3 expects String (IANA time zone or UTC offset ±HH:MM), but "Unknown/Zone" is not a recognized time zone"#
    );

    let error = cho::run(
        r#"(print (dt/fmt $1 "%Y" "Asia/Tokyo"))"#,
        Cursor::new("invalid\n"),
        Vec::new(),
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .starts_with("record 1: dt/fmt: argument 1")
    );

    assert!(
        cho::run(
            r#"(print (dt/fmt $1 "%Y" ""))"#,
            Cursor::new("2026-08-18T00:00:00Z\n"),
            Vec::new(),
        )
        .is_err()
    );
    assert!(
        cho::run(
            r#"(print (dt/fmt $1 "%Y" 9))"#,
            Cursor::new("2026-08-18T00:00:00Z\n"),
            Vec::new(),
        )
        .is_err()
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
fn datetime_values_floor_to_local_boundaries() {
    assert_eq!(
        output(
            r#"(print (dt/floor-h $1 "+05:45") (dt/floor-d $1 "Asia/Tokyo"))"#,
            "2026-08-22T03:30:45Z\n"
        ),
        "2026-08-22T03:15:00Z 2026-08-21T15:00:00Z\n"
    );
}

#[test]
fn datetime_floor_handles_dst_boundaries() {
    assert_eq!(
        output(
            r#"(print (dt/floor-h $1 "America/New_York"))"#,
            "2026-11-01T05:30:00Z\n2026-11-01T06:30:00Z\n"
        ),
        "2026-11-01T05:00:00Z\n2026-11-01T06:00:00Z\n"
    );
    assert_eq!(
        output(
            r#"(print (dt/floor-d $1 "America/Sao_Paulo"))"#,
            "2018-11-04T03:30:00Z\n"
        ),
        "2018-11-04T03:00:00Z\n"
    );
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

    let error = cho::run(
        r#"(print (dt/floor-d $1 "Unknown/Zone"))"#,
        Cursor::new("2026-08-18T00:00:00Z\n"),
        Vec::new(),
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .starts_with("record 1: dt/floor-d: argument 2")
    );
}

#[test]
fn datetime_errors_identify_the_argument() {
    let error = cho::run(
        r#"(print (dt/fmt $1 "%Y"))"#,
        Cursor::new("2026-08-18\n"),
        Vec::new(),
    )
    .unwrap_err();
    assert_eq!(
        error.to_string(),
        r#"record 1: dt/fmt: argument 1 expects DateTime, but "2026-08-18" is not valid RFC 3339"#
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
fn duration_milliseconds_and_days_follow_existing_duration_rules() {
    assert_eq!(
        output(
            "(print (du/ms 250) (du/ms -1.5) (du/d 1) (du/d -0.5))",
            "x\n"
        ),
        "0.25 -0.0015 86400 -43200\n"
    );
    assert_eq!(
        output(
            "(print (dt/add $1 (du/ms 1)) (dt/sub $1 (du/d 1)))",
            "2026-08-21T00:00:00Z\n"
        ),
        "2026-08-21T00:00:00.001Z 2026-08-20T00:00:00Z\n"
    );
}

#[test]
fn datetime_format_accepts_iana_timezones_and_fixed_offsets() {
    assert_eq!(
        output(
            r#"(print (dt/fmt $1 "%Y-%m-%d %H:%M %z" "America/New_York"))"#,
            "2026-01-15T12:00:00Z\n2026-07-15T12:00:00Z\n"
        ),
        "2026-01-15 07:00 -0500\n2026-07-15 08:00 -0400\n"
    );
    assert_eq!(
        output(
            r#"(print (dt/fmt $1 "%Y-%m-%d %H:%M %z" "+05:45"))"#,
            "2026-08-18T23:30:00Z\n"
        ),
        "2026-08-19 05:15 +0545\n"
    );
    assert_eq!(
        output(
            r#"(print (dt/fmt $1 "%H:%M %z" $2))"#,
            "2026-08-18T00:00:00Z Asia/Tokyo\n"
        ),
        "09:00 +0900\n"
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
fn durations_convert_to_numbers_in_explicit_units() {
    assert_eq!(
        output(
            "(print (du/to-ms (du/ms 250)) (du/to-s (du/s 0)) (du/to-m (du/m 3)) (du/to-h (du/h 2.5)) (du/to-d (du/d 1.5)))",
            "x\n"
        ),
        "250 0 3 2.5 1.5\n"
    );
    assert_eq!(
        output(
            "(print (/ (du/to-s (dt/diff $1 $2)) 3600) (du/to-h (dt/diff $1 $2)))",
            "2026-08-18T02:30:45Z 2026-08-18T00:00:00Z\n"
        ),
        "2.5125 2.5125\n"
    );
}

#[test]
fn duration_number_conversions_reject_non_durations_and_empty_values() {
    for program in ["(print (du/to-s 1))", "(print (du/to-m $2))"] {
        assert!(cho::run(program, Cursor::new("x\n"), Vec::new()).is_err());
    }
}
