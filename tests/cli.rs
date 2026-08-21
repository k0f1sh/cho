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
        r#"(print (default (dt/fmt "%Y" $1) "invalid"))"#,
        "2026-08-18T00:00:00Z\nnot-a-date\n",
    );
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "2026\ninvalid\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn help_lists_types_and_signatures() {
    let output = Command::new(env!("CARGO_BIN_EXE_cho"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Types:"));
    assert!(stdout.contains("(p VALUE ...)"));
    assert!(stdout.contains("(f PREDICATE)"));
    assert!(stdout.contains("--skip-header"));
    assert!(stdout.contains("(s/part DELIMITER POSITION VALUE)"));
    assert!(stdout.contains("(dt/fmt STRING DATETIME)"));
    assert!(stdout.contains("(dt/floor-m DATETIME)"));
    assert!(stdout.contains("(du/m NUMBER)"));
    assert!(stdout.contains("(du/ms NUMBER)"));
    assert!(stdout.contains("(du/d NUMBER)"));
    assert!(stdout.contains("(ip/version IPADDR)"));
    assert!(stdout.contains("(cidr/network CIDR)"));
    assert!(stdout.contains("(cidr/prefix CIDR)"));
    assert!(stdout.contains("(cidr/first CIDR)"));
    assert!(stdout.contains("(cidr/last CIDR)"));
    assert!(stdout.contains("(cidr/size CIDR)"));
    assert!(stdout.contains("(url/query-get STRING URL)"));
    assert!(stdout.contains("(url/query-has? STRING URL)"));
    assert!(stdout.contains("(semver/major SEMVER)"));
    assert!(stdout.contains("(semver/prerelease SEMVER)"));
    assert!(stdout.contains("fc00::/7"));
    assert!(!stdout.contains("(dur/m NUMBER)"));
    assert!(stdout.contains("(cidr/contains? CIDR IPADDR)"));
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
