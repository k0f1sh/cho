use std::io::Write;
use std::process::{Command, Output, Stdio};

fn run(program: &str, input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cho"))
        .arg(program)
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
    assert!(stdout.contains("(dt/fmt STRING DATETIME)"));
    assert!(stdout.contains("(dt/floor-m DATETIME)"));
    assert!(stdout.contains("(du/m NUMBER)"));
    assert!(!stdout.contains("(dur/m NUMBER)"));
    assert!(stdout.contains("(cidr/contains? CIDR IPADDR)"));
}
