use super::support::{output, output_with_separator};
use std::io::{self, Cursor};
use std::process::Command;

#[test]
fn prints_fields_strings_and_concatenated_values() {
    assert_eq!(
        output(
            r#"(print $1 "score:" $2) (print (str $1 ":" $2))"#,
            "Alice 20\nBob 30\n"
        ),
        "Alice score: 20\nAlice:20\nBob score: 30\nBob:30\n"
    );
}

#[test]
fn joins_values_with_a_separator() {
    assert_eq!(
        output(
            r#"(print (s/join "," $1 $2 $3)) (print (s/join "\t" $1 $3))"#,
            "Alice 20 Tokyo\n"
        ),
        "Alice,20,Tokyo\nAlice\tTokyo\n"
    );
    assert_eq!(output(r#"(print (s/join ","))"#, "Alice\n"), "\n");
}

#[test]
fn literal_replace_handles_first_all_empty_and_nested_values() {
    assert_eq!(
        output(
            concat!(
                r#"(print (s/replace $1 "a" "x") (s/replace-all $1 "a" "x")) "#,
                r#"(print (s/replace $1 "" "-") (s/replace-all $1 "" "-")) "#,
                r#"(print (s/upper (s/replace-all $1 (str "a") (+ 1 1))))"#,
            ),
            "aba\n\n東京\n",
        ),
        concat!(
            "xba xbx\n-aba -a-b-a-\n2B2\n",
            " \n- -\n\n",
            "東京 東京\n-東京 -東-京-\n東京\n",
        )
    );
    assert_eq!(
        output(r#"(print (s/replace-all $2 "" "-"))"#, "only-one-field\n"),
        "-\n"
    );
}

#[test]
fn regex_replace_handles_captures_zero_width_and_threading() {
    assert_eq!(
        output(
            concat!(
                r#"(print (re/replace $1 /(?P<word>[a-z]+)-(\d+)/ "${word}:$2") "#,
                r#"(re/replace-all $1 /(?P<word>[a-z]+)-(\d+)/ "${word}:$2")) "#,
                r#"(print (re/replace $2 /(a)?b/ "<$1>") (re/replace $3 /a/ "$$")) "#,
                r#"(print (re/replace-all $1 /^|$/ "-") "#,
                r#"(-> $1 (re/replace /[a-z]+/ (str "[" "$0" "]"))))"#,
            ),
            "item-12-other-34 b a\n",
        ),
        concat!(
            "item:12-other-34 item:12-other:34\n",
            "<> $\n",
            "-item-12-other-34- [item]-12-other-34\n",
        )
    );
    assert_eq!(
        output(r#"(print (re/replace-all $1 // "-"))"#, "abc\n\n"),
        "-a-b-c-\n-\n"
    );
    assert_eq!(
        output(r#"(print (re/replace-all $1 "\\d+" "X"))"#, "a12b34\n"),
        "aXbX\n"
    );
}

#[test]
fn regex_part_extracts_parts_and_composes_with_values() {
    assert_eq!(
        output(
            concat!(
                r#"(print (re/part $1 /[,:]+/ 1) (re/part $1 /[,:]+/ 2) "#,
                r#"(s/upper (-> $1 (re/part /[,:]+/ (s/count "x")))))"#,
            ),
            "alpha,:beta,,,gamma\n",
        ),
        "alpha beta ALPHA\n"
    );
    assert_eq!(
        output(
            r#"(print (re/part $0 "\\s*[:;,]\\s*" 2))"#,
            "left ; right\n"
        ),
        "right\n"
    );
}

#[test]
fn regex_part_preserves_empty_parts_and_handles_missing_parts() {
    assert_eq!(
        output(
            concat!(
                r#"(print (s/join "|" "#,
                r#"(re/part ":a::" /:+/ 1) "#,
                r#"(re/part ":a::" /:+/ 2) "#,
                r#"(re/part ":a::" /:+/ 3))) "#,
                r#"(print (re/part "whole" /:+/ 1) (dq (re/part "whole" /:+/ 2)))"#,
            ),
            "x\n",
        ),
        "|a|\nwhole \"\"\n"
    );
}

#[test]
fn regex_part_accepts_empty_and_zero_width_patterns() {
    assert_eq!(
        output(
            r#"(print (s/join "|" (re/part "abc" // 1) (re/part "abc" // 2) (re/part "abc" // 5)))"#,
            "x\n",
        ),
        "|a|\n"
    );
}

#[test]
fn regex_part_rejects_invalid_positions() {
    for (program, expected) in [
        (
            r#"(print (re/part $1 /:/ 0))"#,
            "record 1: re/part: argument 3 expects Number (positive whole part position)",
        ),
        (
            r#"(print (re/part $1 /:/ 1.5))"#,
            "record 1: re/part: argument 3 expects Number (positive whole part position)",
        ),
        (
            r#"(print (re/part $1 /:/ NaN))"#,
            "record 1: re/part: argument 3 expects finite Number",
        ),
        (
            r#"(print (re/part $1 /:/ 1e40))"#,
            "record 1: re/part: argument 3 expects Number (representable part position)",
        ),
    ] {
        let error = cho::run(program, Cursor::new("a:b\n"), Vec::new()).unwrap_err();
        assert!(error.to_string().starts_with(expected), "{error}");
    }
}

#[test]
fn part_extracts_one_literal_delimited_part() {
    assert_eq!(
        output(
            concat!(
                r#"(print (s/part $1 ":" 1) "#,
                r#"(s/part $2 "=" 2) "#,
                r#"(s/part (s/part $3 "[" 2) "]:" 1))"#,
            ),
            "192.168.10.20:39652 SRC=10.0.0.25 [fd00::1]:443\n",
        ),
        "192.168.10.20 10.0.0.25 fd00::1\n"
    );
}

#[test]
fn part_preserves_empty_parts_and_returns_the_whole_unsplit_value() {
    assert_eq!(
        output(
            concat!(
                r#"(print (s/join "|" "#,
                r#"(s/part ":a::" ":" 1) "#,
                r#"(s/part ":a::" ":" 2) "#,
                r#"(s/part ":a::" ":" 3) "#,
                r#"(s/part ":a::" ":" 4))) "#,
                r#"(print (s/part "whole" ":" 1)) "#,
                r#"(print (s/part "左区切右" "区切" 2) "#,
                r#"(default (s/part "" ":" 1) "empty"))"#,
            ),
            "x\n",
        ),
        "|a||\nwhole\n右 empty\n"
    );
}

#[test]
fn part_composes_with_values_threading_and_typed_predicates() {
    assert_eq!(
        output(
            concat!(
                r#"(filter (ip/private? (-> $1 (s/part ":" 1)))) "#,
                r#"(print (str "ip=" (s/upper (s/part $1 (str ":") (s/count "x")))))"#,
            ),
            "10.1.2.3:443\n8.8.8.8:53\n",
        ),
        "ip=10.1.2.3\n"
    );
}

#[test]
fn part_returns_empty_for_a_missing_part_and_composes_with_other_values() {
    assert_eq!(
        output(
            concat!(
                r#"(print (s/join "|" (s/part $1 ":" 3) (s/upper (s/part $1 ":" 3)))) "#,
                r#"(print (default (s/part $1 ":" 3) "missing") "#,
                r#"(default (s/part $2 ":" 2) "empty"))"#,
            ),
            "a:b x:\n",
        ),
        "|\nmissing empty\n"
    );
}

#[test]
fn part_keeps_invalid_delimiters_and_positions_strict() {
    for (program, expected) in [
        (
            r#"(print (s/part $1 "" 1))"#,
            "record 1: s/part: argument 2 expects a non-empty delimiter",
        ),
        (
            r#"(print (s/part $1 ":" 0))"#,
            "record 1: s/part: argument 3 expects Number (positive whole part position)",
        ),
        (
            r#"(print (s/part $1 ":" -1))"#,
            "record 1: s/part: argument 3 expects Number (positive whole part position)",
        ),
        (
            r#"(print (s/part $1 ":" 1.5))"#,
            "record 1: s/part: argument 3 expects Number (positive whole part position)",
        ),
        (
            r#"(print (s/part $1 ":" NaN))"#,
            "record 1: s/part: argument 3 expects finite Number",
        ),
        (
            r#"(print (s/part $1 ":" 1e40))"#,
            "record 1: s/part: argument 3 expects Number (representable part position)",
        ),
    ] {
        let error = cho::run(program, Cursor::new("a:b\n"), Vec::new()).unwrap_err();
        assert!(error.to_string().starts_with(expected), "{error}");
    }

    let error = cho::run(
        r#"(filter (> (s/part $1 ":" 3) 0))"#,
        Cursor::new("a:b\n"),
        Vec::new(),
    )
    .unwrap_err();
    assert!(
        error
            .to_string()
            .starts_with("record 1: >: argument 1 expects Number"),
        "{error}"
    );
}

#[test]
fn slice_extracts_unicode_characters_and_clamps_to_the_end() {
    assert_eq!(
        output(
            r#"(print (s/slice "washington" 5) (s/slice "washington" 5 3)) (print (s/slice "東京駅前" 2 3)) (print (s/slice "short" 20 3) (dq (s/slice "abc" 2 0)))"#,
            "x\n",
        ),
        "ington ing\n京駅前\n \"\"\n"
    );
    assert_eq!(
        output(
            r#"(print (dq (s/slice "" 1)) (dq (s/slice "abc" 1 1e40)) (dq (s/slice "abc" 1e40)))"#,
            "x\n",
        ),
        "\"\" \"abc\" \"\"\n"
    );
}

#[test]
fn slice_composes_with_values_and_threading() {
    assert_eq!(
        output(
            r#"(print (s/upper (s/slice $1 (s/count "x") 3)) (-> $1 (s/slice 2) (dq)))"#,
            "abcdef\n",
        ),
        "ABC \"bcdef\"\n"
    );
}

#[test]
fn slice_rejects_invalid_starts_and_lengths() {
    for (program, argument) in [
        (r#"(print (s/slice "abc" 0))"#, 2),
        (r#"(print (s/slice "abc" -1))"#, 2),
        (r#"(print (s/slice "abc" 1.5))"#, 2),
        (r#"(print (s/slice "abc" 1 -1))"#, 3),
        (r#"(print (s/slice "abc" 1 1.5))"#, 3),
    ] {
        let error = cho::run(program, Cursor::new("x\n"), Vec::new()).unwrap_err();
        assert!(
            error.to_string().starts_with(&format!(
                "record 1: s/slice: argument {argument} expects Number"
            )),
            "{error}"
        );
    }
}

#[test]
fn count_counts_unicode_characters() {
    assert_eq!(
        output("(print (s/count $1))", "Alice\n東京\n🦀\n"),
        "5\n2\n1\n"
    );
}

#[test]
fn count_can_be_used_in_filters() {
    assert_eq!(
        output(
            "(filter (> (s/count $1) 3)) (print $1)",
            "Al\nAlice\n東京\nCarol\n"
        ),
        "Alice\nCarol\n"
    );
}

#[test]
fn escape_makes_tabs_and_backslashes_visible() {
    assert_eq!(
        output(r#"(print (s/escape $0))"#, "first\tsecond\\third\r\n"),
        r#"first\tsecond\\third"#.to_owned() + "\n"
    );
}

#[test]
fn quote_stringifies_values_and_escapes_the_enclosing_quote() {
    assert_eq!(
        output(
            r#"(print (s/dquote $0)) (print (s/squote $0))"#,
            "say \"it's\\ok\"\t2026\n",
        ),
        "\"say \\\"it's\\\\ok\\\"\\t2026\"\n'say \"it\\'s\\\\ok\"\\t2026'\n"
    );
    assert_eq!(
        output(r#"(print (str "value=" (s/dquote (+ $1 1))))"#, "41\n",),
        "value=\"42\"\n"
    );
    assert_eq!(output("(print (s/dquote $2))", "x\n"), "\"\"\n");
    assert_eq!(
        output("(print (dq $1) (sq $1))", "Alice\n"),
        "\"Alice\" 'Alice'\n"
    );
}

#[test]
fn unquote_decodes_quotes_and_composes_as_a_value() {
    assert_eq!(
        output(
            r#"(print (dq (s/unquote $0))) (print (s/upper (s/unquote $0)))"#,
            "\"say \\\"hello\\\"\\nnext\\tpath\\\\end\"\n'it\\'s good'\nplain\n\"\"\n",
        ),
        concat!(
            "\"say \\\"hello\\\"\\nnext\\tpath\\\\end\"\n",
            "SAY \"HELLO\"\nNEXT\tPATH\\END\n",
            "\"it's good\"\nIT'S GOOD\n",
            "\"plain\"\nPLAIN\n",
            "\"\"\n\n",
        )
    );
    assert_eq!(
        output("(print (s/unquote (dq $0)))", "say \"it's\\ok\"\n"),
        "say \"it's\\ok\"\n"
    );
    assert_eq!(
        output("(print (s/unquote (sq $0)))", "say \"it's\\ok\"\n"),
        "say \"it's\\ok\"\n"
    );
    assert_eq!(output("(print (s/unquote 42))", "x\n"), "42\n");
}

#[test]
fn unquote_rejects_mismatched_quotes_and_invalid_escapes() {
    for input in [
        "\"value'\n",
        "\"value\n",
        "\"bad\\q\"\n",
        "\"bad\\\"\n",
        "\"bad\"quote\"\n",
    ] {
        let error = cho::run("(print (s/unquote $0))", Cursor::new(input), Vec::new()).unwrap_err();
        assert!(
            error
                .to_string()
                .starts_with("record 1: s/unquote: argument 1 expects"),
            "{error}"
        );
    }
}

#[test]
fn shq_quotes_one_posix_shell_argument_without_expansion() {
    let cases = [
        (r#"hello"#, "hello"),
        (r#""#, ""),
        (r#"hello world"#, "hello world"),
        (r#"it's good"#, "it's good"),
        (r#"say \"hello\""#, "say \"hello\""),
        (r#"$HOME"#, "$HOME"),
        (r#"$(date)"#, "$(date)"),
        (r#"`date`"#, "`date`"),
        (r#"* ? [abc]"#, "* ? [abc]"),
        (r#"; | && >"#, "; | && >"),
        (r#"a\\b"#, "a\\b"),
        ("line\\nnext", "line\nnext"),
        (r#"日本語"#, "日本語"),
    ];

    for (literal, original) in cases {
        let program = format!(r#"(print (shq "{literal}"))"#);
        let rendered = output(&program, "x\n");
        let quoted = rendered.strip_suffix('\n').unwrap();
        let expected = format!("'{}'", original.replace('\'', "'\\''"));
        assert_eq!(quoted, expected, "failed to quote {original:?}");

        let recovered = Command::new("sh")
            .args([
                "-c",
                r#"eval "set -- $1"; test "$#" -eq 1 && printf %s "$1""#,
                "sh",
                quoted,
            ])
            .output()
            .unwrap();
        assert!(recovered.status.success(), "shell rejected {quoted:?}");
        assert_eq!(recovered.stdout, original.as_bytes());
    }

    assert_eq!(
        output(r#"(print (str "command " (shq (+ 40 2))))"#, "x\n"),
        "command '42'\n"
    );
}

#[test]
fn lower_and_upper_apply_unicode_case_conversion() {
    assert_eq!(
        output(r#"(print (s/lower $1) (s/upper $2))"#, "ÄLICE 東京abc\n"),
        "älice 東京ABC\n"
    );
}

#[test]
fn reverse_reverses_unicode_characters_and_composes_as_a_value() {
    assert_eq!(
        output(
            r#"(print (s/reverse $1) (s/reverse "東京駅")) (print (-> $2 s/reverse s/upper))"#,
            "abc xyz\n",
        ),
        "cba 駅京東\nZYX\n"
    );
    assert_eq!(output(r#"(print (dq (s/reverse "")))"#, "x\n"), "\"\"\n");
}

#[test]
fn trim_operations_remove_unicode_whitespace_and_compose() {
    assert_eq!(
        output(
            r#"(print (dq (s/trim $0))) (print (dq (s/ltrim $0))) (print (dq (s/rtrim $0)))"#,
            "\u{2003}\t Alice \u{3000}\n",
        ),
        "\"Alice\"\n\"Alice 　\"\n\" \\t Alice\"\n"
    );
    assert_eq!(
        output(
            r#"(print (s/upper (s/trim $0))) (print (dq (s/trim $0)))"#,
            "  alice  \n\u{3000}\t \n\n",
        ),
        "ALICE\n\"alice\"\n\n\"\"\n\n\"\"\n"
    );
    assert_eq!(output("(print (s/trim 42))", "x\n"), "42\n");
}

#[test]
fn string_tests_use_subject_first_and_compose_as_boolean_values() {
    assert_eq!(
        output(
            concat!(
                r#"(print (s/starts-with? $1 "api-") (s/ends-with? $1 ".log") "#,
                r#"(s/contains? $1 "error")) "#,
                r#"(filter (and (-> $1 (s/starts-with? "api-")) "#,
                r#"(s/contains? $1 "error"))) (print $1)"#,
            ),
            "api-error.log\napi-info.log\nworker-error.log\n",
        ),
        concat!(
            "true true true\napi-error.log\n",
            "true true false\n",
            "false true true\n",
        )
    );
    assert_eq!(
        output(
            r#"(print (s/starts-with? $1 "") (s/ends-with? $1 "") (s/contains? $1 ""))"#,
            "東京\n\n",
        ),
        "true true true\ntrue true true\n"
    );
}

#[test]
fn string_tests_require_string_arguments() {
    for (program, argument) in [
        (r#"(print (s/starts-with? 1 "1"))"#, 1),
        (r#"(print (s/ends-with? $1 1))"#, 2),
        (r#"(print (s/contains? true "true"))"#, 1),
    ] {
        let error = cho::run(program, Cursor::new("value\n"), Vec::new()).unwrap_err();
        assert!(
            error
                .to_string()
                .contains(&format!("argument {argument} expects String")),
            "{error}"
        );
    }
}

#[test]
fn default_replaces_only_empty_values_and_can_be_nested() {
    assert_eq!(
        output(
            r#"(print (default $2 (s/upper "unknown")))"#,
            "Alice Tokyo\nBob\nCarol \n"
        ),
        "Tokyo\nUNKNOWN\nUNKNOWN\n"
    );
    assert_eq!(
        output(r#"(print (default "" "fallback"))"#, "x\n"),
        "fallback\n"
    );
}

#[test]
fn field_ranges_preserve_regex_separators_and_empty_fields() {
    assert_eq!(
        output_with_separator(
            "(print $-2) (print $2-) (print $2-3)",
            "[,;]",
            ",Alice;Tokyo;"
        ),
        ",Alice\nAlice;Tokyo;\nAlice;Tokyo\n"
    );
}

#[test]
fn boolean_predicates_can_be_nested_with_regexes() {
    assert_eq!(
        output(
            r#"(filter (and (not (reg "debug")) (or (reg "error") (>= $2 40)))) (print $0)"#,
            "info 10\ndebug error 50\nerror 20\ninfo 40\n"
        ),
        "error 20\ninfo 40\n"
    );
}

#[test]
fn regex_predicates_share_the_same_execution_path_as_boolean_values() {
    assert_eq!(
        output(
            r#"(print (~ $1 /^a/) (if (~ $2 /z$/) "yes" "no"))"#,
            "apple jazz\npear music\n"
        ),
        "true yes\nfalse no\n"
    );
}

#[test]
fn supports_all_string_comparison_operators() {
    assert_eq!(
        output(r#"(print (if (s/< "10" "2") "yes" "no"))"#, "x\n"),
        "yes\n"
    );
    assert_eq!(
        output(r#"(print (if (s/<= "a" "a") "yes" "no"))"#, "x\n"),
        "yes\n"
    );
    assert_eq!(
        output(r#"(print (if (s/> "b" "a") "yes" "no"))"#, "x\n"),
        "yes\n"
    );
    assert_eq!(
        output(r#"(print (if (s/>= "b" "b") "yes" "no"))"#, "x\n"),
        "yes\n"
    );
    assert_eq!(
        output(r#"(print (if (s/!= "a" "b") "yes" "no"))"#, "x\n"),
        "yes\n"
    );
}

#[test]
fn regex_filters_match_lines_or_specific_fields() {
    assert_eq!(
        output(
            r#"(filter (reg "^error:")) (print NR $0)"#,
            "info: ready\nerror: failed\n"
        ),
        "2 error: failed\n"
    );
    assert_eq!(
        output(
            r#"(filter (reg $1 "^[A-Z][a-z]+$")) (print $1)"#,
            "Alice 20\nbob 30\nCAROL 40\n"
        ),
        "Alice\n"
    );
}

#[test]
fn regex_literals_preserve_escapes_and_reg_has_a_tilde_alias() {
    assert_eq!(
        output(r#"(filter (~ $1 /^\d+$/)) (print $1)"#, "123\n12a\n456\n"),
        "123\n456\n"
    );
    assert_eq!(
        output(
            r#"(filter (reg /^foo\/bar$/)) (print $0)"#,
            "foo/bar\nfoo\n"
        ),
        "foo/bar\n"
    );
}

#[test]
fn invalid_regexes_fail_before_processing() {
    let error = cho::run(
        r#"(filter (reg "[")) (print $0)"#,
        Cursor::new("input\n"),
        Vec::new(),
    )
    .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
}

#[test]
fn invalid_regexes_in_if_fail_before_processing() {
    let error = cho::run(
        r#"(print (if (reg "[") "yes" "no"))"#,
        Cursor::new("input\n"),
        Vec::new(),
    )
    .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
}

#[test]
fn field_separators_can_be_regexes_and_preserve_empty_fields() {
    assert_eq!(
        output_with_separator("(print NF $1 $2 $3)", "[,;]", ",Alice;\n"),
        "3  Alice \n"
    );
}

#[test]
fn separators_that_match_empty_strings_are_rejected() {
    let error =
        cho::run_with_field_separator("(print $1)", Some(".*"), Cursor::new("Alice\n"), Vec::new())
            .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
}

#[test]
fn csv_mode_decodes_quoted_fields() {
    let mut result = Vec::new();
    cho::run_csv(
        "(print NF $1 $2 $3)",
        Cursor::new("Alice,\"Bob, Jr.\",\"said \"\"hello\"\"\"\n"),
        &mut result,
    )
    .unwrap();
    assert_eq!(
        String::from_utf8(result).unwrap(),
        "3 Alice Bob, Jr. said \"hello\"\n"
    );
}

#[test]
fn escape_keeps_multiline_csv_fields_on_one_output_line() {
    let mut result = Vec::new();
    cho::run_csv(
        "(print NF (s/escape $2))",
        Cursor::new("Tokyo,\"rain\r\nthen\tsun\\later\"\n"),
        &mut result,
    )
    .unwrap();
    assert_eq!(
        String::from_utf8(result).unwrap(),
        "2 rain\\r\\nthen\\tsun\\\\later\n"
    );
}

#[test]
fn typed_values_render_inside_string_operations() {
    assert_eq!(
        output(
            r#"(print (str "at=" (dt/unix 0)) (s/join "," (dt/unix 0) (du/s 1.5)) (s/count (dt/unix 0)))"#,
            "x\n"
        ),
        "at=1970-01-01T00:00:00Z 1970-01-01T00:00:00Z,1.5 20\n"
    );
}
