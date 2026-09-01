use std::io::Write;
use std::process::{Command, Output, Stdio};

fn run(program: &str, input: &str) -> Output {
    run_with_args(&[program], input)
}

fn run_with_args(arguments: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cho"))
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn runtime_errors_keep_prior_output_and_exit_nonzero() {
    let output = run("(filter (> $2 20)) (print $1)", "Alice 30\nBob unknown\n");
    assert!(!output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Alice\n");
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "cho: record 2: >: argument 1 expects Number, but \"unknown\" cannot be parsed as a number\n"
    );
}

#[test]
fn default_recovery_finishes_successfully_without_a_diagnostic() {
    let output = run(
        r#"(print (default (dt/fmt $1 "%Y") "invalid"))"#,
        "2026-08-18T00:00:00Z\nnot-a-date\n",
    );
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "2026\ninvalid\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn parse_errors_explain_unbalanced_parentheses() {
    let missing = run("(print $1", "");
    assert!(!missing.status.success());
    assert_eq!(
        String::from_utf8(missing.stderr).unwrap(),
        "cho: invalid program: missing closing parenthesis\n"
    );

    let unexpected = run("(print $1))", "");
    assert!(!unexpected.status.success());
    assert_eq!(
        String::from_utf8(unexpected.stderr).unwrap(),
        "cho: invalid program: unexpected closing parenthesis\n"
    );
}

#[test]
fn parse_errors_preserve_lexer_details() {
    let output = run(r#"(print "unfinished)"#, "");
    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "cho: invalid program: unterminated string literal\n"
    );
}

#[test]
fn parse_errors_explain_function_argument_counts() {
    for (program, message) in [
        (
            "(print (s/count))",
            "cho: invalid program: s/count: expected 1 argument, but got 0\n",
        ),
        (
            "(filter (reg $1 $2 $3))",
            "cho: invalid program: reg: expected 1 or 2 arguments, but got 3\n",
        ),
        (
            "(print (s/join))",
            "cho: invalid program: s/join: expected at least 1 argument, but got 0\n",
        ),
    ] {
        let output = run(program, "");
        assert!(!output.status.success());
        assert_eq!(String::from_utf8(output.stderr).unwrap(), message);
    }
}

#[test]
fn parse_errors_explain_ambiguous_top_level_output() {
    for (program, message) in [
        (
            "$1 $2",
            "cho: invalid program: only one automatic top-level value is allowed\n",
        ),
        (
            "(print $1) $2",
            "cho: invalid program: cannot combine an automatic top-level value with print\n",
        ),
        (
            "$1 (print $2)",
            "cho: invalid program: cannot combine an automatic top-level value with print\n",
        ),
        (
            "$1 (filter true)",
            "cho: invalid program: an automatic top-level value must follow all filters\n",
        ),
    ] {
        let output = run(program, "");
        assert!(!output.status.success());
        assert_eq!(String::from_utf8(output.stderr).unwrap(), message);
    }
}

#[test]
fn parse_errors_reject_non_finite_number_literals() {
    for literal in ["NaN", "inf", "-inf", "1e309", "-1e309"] {
        let output = run_with_args(&["--no-input", &format!("(print {literal})")], "");
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            format!("cho: invalid program: non-finite number literal: {literal}\n")
        );
    }
}

#[test]
fn parse_errors_name_unknown_functions() {
    let output = run_with_args(&["--no-input", r#"(p (ip/v6 "10.0.0.1"))"#], "");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        "cho: invalid program: no such function: ip/v6\n"
    );
}

#[test]
fn help_lists_types_and_signatures() {
    let output = Command::new(env!("CARGO_BIN_EXE_cho"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains('\x1b'));
    assert!(stdout.starts_with("cho — a small, type-aware text processor for the command line\n"));
    assert!(stdout.contains("Common recipes:"));
    assert!(stdout.contains("Types and errors:"));
    assert!(stdout.contains("Number uses IEEE 754 double precision."));
    assert!(stdout.contains("integer identifiers as strings with s/="));
    assert!(stdout.contains("String functions render VALUE arguments"));
    assert!(stdout.contains("s/ prefix therefore explicitly selects string"));
    assert!(stdout.contains("(p VALUE ...)"));
    assert!(stdout.contains("(f BOOLEAN)"));
    assert!(stdout.contains("(s/empty? VALUE)"));
    assert!(stdout.contains("only filters implicitly prints $0"));
    assert!(stdout.contains("An empty program also prints $0"));
    assert!(stdout.contains("true, false"));
    assert!(stdout.contains("(if BOOLEAN VALUE VALUE)"));
    assert!(stdout.contains("--skip-header"));
    assert!(stdout.contains("--no-input"));
    assert!(stdout.contains("-F separator must be a valid regular expression"));
    assert!(stdout.contains("It must not match an\n  empty string"));
    assert!(stdout.contains("field accepts a non-negative whole number"));
    assert!(stdout.contains("fields also requires START to be less than or equal to END"));
    assert!(stdout.contains("dt/unix accepts only whole Unix seconds"));
    assert!(stdout.contains("(s/part VALUE DELIMITER POSITION)"));
    assert!(stdout.contains("(s/slice VALUE START [LENGTH])"));
    assert!(stdout.contains("(s/replace VALUE FROM TO)"));
    assert!(stdout.contains("(s/replace-all VALUE FROM TO)"));
    assert!(stdout.contains("(s/starts-with? VALUE PREFIX)"));
    assert!(stdout.contains("(s/ends-with? VALUE SUFFIX)"));
    assert!(stdout.contains("(s/contains? VALUE NEEDLE)"));
    assert!(stdout.contains("(re/replace VALUE /PATTERN/ REPLACEMENT)"));
    assert!(stdout.contains("(re/replace-all VALUE /PATTERN/ REPLACEMENT)"));
    assert!(stdout.contains("like awk sub(\"\", ...) and gsub(\"\", ...)"));
    assert!(stdout.contains(r#"(re/replace $1 "\\d+" "X")"#));
    assert!(stdout.contains("(dt/fmt DATETIME STRING)"));
    assert!(stdout.contains("(dt/fmt DATETIME STRING TIMEZONE)"));
    assert!(stdout.contains("Asia/Tokyo"));
    assert!(stdout.contains("+09:00"));
    assert!(stdout.contains("(dt/floor-m DATETIME)"));
    assert!(stdout.contains("(dt/floor-d DATETIME TIMEZONE)"));
    assert!(stdout.contains("(du/m NUMBER)"));
    assert!(stdout.contains("(du/ms NUMBER)"));
    assert!(stdout.contains("(du/d NUMBER)"));
    assert!(stdout.contains("(ip/version IPADDR)"));
    assert!(stdout.contains("(ip/v4? IPADDR)"));
    assert!(stdout.contains("(ip/v6? IPADDR)"));
    assert!(stdout.contains("(cidr/network CIDR)"));
    assert!(stdout.contains("(cidr/prefix CIDR)"));
    assert!(stdout.contains("(cidr/first CIDR)"));
    assert!(stdout.contains("(cidr/last CIDR)"));
    assert!(stdout.contains("(cidr/size CIDR)"));
    assert!(stdout.contains("(url/query-get URL STRING)"));
    assert!(stdout.contains("(url/query-has? URL STRING)"));
    assert!(stdout.contains("(semver/major SEMVER)"));
    assert!(stdout.contains("(semver/prerelease SEMVER)"));
    assert!(stdout.contains("fc00::/7"));
    assert!(!stdout.contains("(dur/m NUMBER)"));
    assert!(stdout.contains("(cidr/contains? CIDR IPADDR)"));
}

#[test]
fn no_input_runs_once_with_an_empty_record() {
    let output = run_with_args(
        &["--no-input", r#"(print (s/join "," $0 $1 NR NF))"#],
        "this input must not be evaluated\n",
    );
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), ",,1,0\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn no_input_supports_datetime_values() {
    let output = run_with_args(&["--no-input", "(print (dt/floor-d (dt/now)))"], "");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.ends_with("T00:00:00Z\n"));
    assert!(output.stderr.is_empty());
}

#[test]
fn skip_header_skips_one_logical_csv_record_and_preserves_nr() {
    let output = run_with_args(
        &["--csv", "--skip-header", "(print NR $1 $2)"],
        "\"display\nname\",age\nAlice,20\nBob,30\n",
    );
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "2 Alice 20\n3 Bob 30\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn skip_header_works_in_tsv_mode() {
    let output = run_with_args(
        &["--tsv", "--skip-header", "(print NR $1 $2)"],
        "name\tage\nAlice\t20\n",
    );
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "2 Alice 20\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn skip_header_accepts_empty_and_header_only_input() {
    for (arguments, input) in [
        (vec!["--csv", "--skip-header", "(print $1)"], ""),
        (vec!["--csv", "--skip-header", "(print $1)"], "name,age\n"),
        (vec!["--tsv", "--skip-header", "(print $1)"], "name\tage\n"),
    ] {
        let output = run_with_args(&arguments, input);
        assert!(output.status.success());
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn skip_header_requires_csv_or_tsv_mode() {
    for arguments in [
        vec!["--skip-header", "(print $1)"],
        vec!["-F,", "--skip-header", "(print $1)"],
    ] {
        let output = run_with_args(&arguments, "name,age\nAlice,20\n");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .starts_with("Usage: cho")
        );
    }
}
