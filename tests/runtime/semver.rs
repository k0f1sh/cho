use super::support::output;
use std::io::Cursor;

#[test]
fn semver_numeric_extractors_reject_unsafe_integers_without_rounding() {
    assert_eq!(
        output("(print (semver/major $1))", "9007199254740991.0.0\n"),
        "9007199254740991\n"
    );
    let error = cho::run(
        "(print (semver/major $1))",
        Cursor::new("9007199254740992.0.0\n"),
        Vec::new(),
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .starts_with("record 1: semver/major: argument 1 expects"),
        "{error}"
    );
    assert_eq!(
        output(
            "(print (default (semver/patch $1) \"invalid\"))",
            "1.2.9007199254740992\ninvalid\n"
        ),
        "invalid\ninvalid\n"
    );
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
fn semver_extractors_return_components_and_preserve_build_support() {
    assert_eq!(
        output(
            concat!(
                "(print (semver/major $1) (semver/minor $1) ",
                "(semver/patch $1) (semver/prerelease $1) (semver/build $1))",
            ),
            "1.2.3-alpha.1+build.9\n2.0.0+metadata\n",
        ),
        "1 2 3 alpha.1 build.9\n2 0 0  metadata\n"
    );
    assert_eq!(
        output(
            "(print (+ (semver/major $1) (semver/minor $1)))",
            "10.2.3\n"
        ),
        "12\n"
    );
    assert_eq!(
        output(
            "(print (s/upper (semver/build $1)))",
            "1.2.3+linux.x86-64\n1.2.3\n"
        ),
        "LINUX.X86-64\n\n"
    );
}

#[test]
fn semver_extractors_report_invalid_versions() {
    for extractor in [
        "semver/major",
        "semver/minor",
        "semver/patch",
        "semver/prerelease",
        "semver/build",
    ] {
        let program = format!("(print ({extractor} $1))");
        let error = cho::run(&program, Cursor::new("1.2\n"), Vec::new()).unwrap_err();
        assert!(
            error
                .to_string()
                .starts_with(&format!("record 1: {extractor}: argument 1 expects SemVer")),
            "{error}"
        );
    }
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
