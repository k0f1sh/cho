use super::support::output;
use std::io::Cursor;

#[test]
fn short_top_level_aliases_filter_and_print() {
    assert_eq!(
        output("(f (> $2 20)) (p $1)", "Alice 18\nBob 30\n"),
        "Bob\n"
    );
    assert_eq!(output("(p)", "Alice\nBob\n"), "\n\n");
}

#[test]
fn a_single_top_level_value_is_printed() {
    assert_eq!(output("(s/upper $1)", "Alice 18\nBob 30\n"), "ALICE\nBOB\n");
    assert_eq!(
        output("(f (> $2 20)) (s/upper $1)", "Alice 18\nBob 30\n"),
        "BOB\n"
    );
}

#[test]
fn if_selects_a_value_and_nests_with_other_values() {
    assert_eq!(
        output(
            r#"(print (str $1 ":" (if (>= $2 20) (s/upper "adult") (s/lower "MINOR"))))"#,
            "Alice 18\nBob 30\n"
        ),
        "Alice:minor\nBob:ADULT\n"
    );
}

#[test]
fn filter_skips_remaining_forms_for_a_record() {
    assert_eq!(
        output(
            r#"(print "checking" $1) (filter (> $2 20)) (print "passed" $1)"#,
            "Alice 18\nBob 30\n"
        ),
        "checking Alice\nchecking Bob\npassed Bob\n"
    );
}

#[test]
fn filter_only_programs_implicitly_print_passing_records() {
    assert_eq!(
        output(
            "(filter (> $2 20)) (filter (< $2 40))",
            "Alice 18\nBob 30\nCarol 45\n"
        ),
        "Bob 30\n"
    );
}

#[test]
fn not_inverts_a_predicate() {
    assert_eq!(
        output(
            r#"(filter (not (reg "debug"))) (print $0)"#,
            "info ready\ndebug details\nerror failed\n"
        ),
        "info ready\nerror failed\n"
    );
}

#[test]
fn and_requires_every_predicate_to_match() {
    assert_eq!(
        output(
            "(filter (and (> $2 20) (< $2 40))) (print $1)",
            "Alice 18\nBob 30\nCarol 45\n"
        ),
        "Bob\n"
    );
}

#[test]
fn or_requires_any_predicate_to_match() {
    assert_eq!(
        output(
            r#"(filter (or (s/= $1 "Alice") (s/= $1 "Bob"))) (print $1)"#,
            "Alice 20\nBob 30\nCarol 40\n"
        ),
        "Alice\nBob\n"
    );
}

#[test]
fn boolean_values_compose_in_filter_if_and_logical_functions() {
    assert_eq!(
        output(
            concat!(
                r#"(filter (if (s/= $1 "prod") (ip/private? $2) (ip/loopback? $2))) "#,
                r#"(print true false (not (if (> $3 0) true false)))"#,
            ),
            "prod 10.0.0.1 1\ndev 127.0.0.1 -1\ndev 8.8.8.8 1\n",
        ),
        "true false false\ntrue false true\n"
    );
}

#[test]
fn boolean_consumers_reject_other_types_without_implicit_conversion() {
    for (program, function) in [
        (r#"(filter "true")"#, "filter"),
        (r#"(print (if 1 "yes" "no"))"#, "if"),
        ("(print (not 1))", "not"),
        ("(print (and true 1))", "and"),
        ("(print (or false 1))", "or"),
    ] {
        let error = cho::run(program, Cursor::new("x\n"), Vec::new()).unwrap_err();
        assert!(
            error.to_string().contains(&format!("{function}: argument")),
            "{error}"
        );
        assert!(error.to_string().contains("expects Boolean"), "{error}");
    }
}

#[test]
fn and_and_or_short_circuit_boolean_value_errors() {
    assert_eq!(
        output(
            r#"(print (and false (> "not-a-number" 0)) (or true (> "not-a-number" 0)))"#,
            "x\n",
        ),
        "false true\n"
    );
}

#[test]
fn default_recovers_from_runtime_errors_and_is_lazy() {
    assert_eq!(
        output(
            r#"(print (default (if (> $1 0) "positive" "zero") "invalid"))"#,
            "2\nunknown\n"
        ),
        "positive\ninvalid\n"
    );
    assert_eq!(
        output(r#"(print (default "ok" (dt/fmt "invalid" "%Q")))"#, "x\n"),
        "ok\n"
    );
}

#[test]
fn predicates_render_as_boolean_values_and_compose() {
    assert_eq!(
        output(
            concat!(
                r#"(print (semver/> $1 $2) (> $3 $4) "#,
                r#"(str "loopback=" (ip/loopback? $5)) "#,
                r#"(s/join ":" (s/= $6 "yes") (not (s/= $6 "yes"))) "#,
                r#"(default (semver/= $1 $2) "fallback"))"#,
            ),
            "1.2.3 2.0.0 10 2 127.0.0.1 yes\n2.0.0 1.2.3 1 2 8.8.8.8 no\n",
        ),
        concat!(
            "false true loopback=true true:false false\n",
            "true false loopback=false false:true false\n",
        )
    );
}

#[test]
fn threading_runs_as_the_expanded_function_call() {
    assert_eq!(
        output(
            r#"(print (str "date: " (-> $1 (dt/fmt "%Y/%m/%d"))))"#,
            "2026-08-18T00:00:00Z\n"
        ),
        "date: 2026/08/18\n"
    );
}

#[test]
fn threading_accepts_bare_names_for_unary_steps() {
    assert_eq!(
        output(
            r#"(print (-> $1 s/trim (s/replace-all "-" "_") s/upper))"#,
            "  api-log-prod  \n",
        ),
        "API_LOG_PROD\n"
    );
    assert_eq!(
        output(r#"(print (->> $1 s/upper (str "value=")))"#, "hello\n"),
        "value=HELLO\n"
    );
}
