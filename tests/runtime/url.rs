use super::support::output;
use std::io::Cursor;

#[test]
fn url_component_extraction_preserves_encoding_and_composes() {
    assert_eq!(
        output(
            concat!(
                r#"(print (url/scheme $1) (url/host $1) (url/port $1) "#,
                r#"(url/path $1) (url/query $1) (url/fragment $1)) "#,
                r#"(print (str "host=" (s/upper (url/host $1))) (-> $1 (url/path)))"#,
            ),
            "https://example.com:8443/a%20b?q=hello%20world#top\n",
        ),
        concat!(
            "https example.com 8443 /a%20b q=hello%20world top\n",
            "host=EXAMPLE.COM /a%20b\n",
        )
    );
}

#[test]
fn url_component_extraction_returns_empty_for_missing_optional_parts() {
    assert_eq!(
        output(
            concat!(
                r#"(print (s/join "|" (url/host $1) (url/port $1) "#,
                r#"(url/path $1) (url/query $1) (url/fragment $1)))"#,
            ),
            "https://example.com\nhttps://[::1]/\nmailto:alice@example.com\n",
        ),
        "example.com||/||\n[::1]||/||\n||alice@example.com||\n"
    );
}

#[test]
fn url_component_extraction_rejects_invalid_urls_and_non_strings() {
    for (program, expected) in [
        (
            "(print (url/host $1))",
            "record 1: url/host: argument 1 expects Url (absolute URL)",
        ),
        (
            "(print (url/path 10))",
            "record 1: url/path: argument 1 expects String",
        ),
        (
            r#"(print (url/query ""))"#,
            "record 1: url/query: argument 1 expects Url (absolute URL)",
        ),
    ] {
        let error = cho::run(program, Cursor::new("relative/path\n"), Vec::new()).unwrap_err();
        assert!(error.to_string().starts_with(expected), "{error}");
    }
}

#[test]
fn url_query_values_use_form_urlencoded_semantics() {
    assert_eq!(
        output(
            concat!(
                r#"(print (s/join "|" (url/query-get $1 "lang") "#,
                r#"(url/query-get $1 "missing") (url/query-get $1 "empty") "#,
                r#"(url/query-get $1 "a b") (url/query-get $1 "名前")))"#,
            ),
            concat!(
                "https://example.com/?lang=ja&empty=&a+b=hello+world&",
                "%E5%90%8D%E5%89%8D=%E6%9D%B1%E4%BA%AC\n",
            ),
        ),
        "ja|||hello world|東京\n"
    );
    assert_eq!(
        output(
            r#"(print (url/query-get $1 "tag") (url/query $1))"#,
            "https://example.com/?tag=first&tag=second\n",
        ),
        "first tag=first&tag=second\n"
    );
}

#[test]
fn url_query_presence_is_boolean_and_distinguishes_missing_keys() {
    assert_eq!(
        output(
            concat!(
                r#"(print (url/query-has? $1 "foo") (url/query-has? $1 "bar") "#,
                r#"(if (url/query-has? $1 "empty") "present" "missing"))"#,
            ),
            "https://example.com/?foo&empty=\nhttps://example.com/\n",
        ),
        "true false present\nfalse false missing\n"
    );
    assert_eq!(
        output(
            r#"(filter (url/query-has? $1 "keep")) (print $1)"#,
            "https://example.com/?keep=\nhttps://example.com/?drop=1\n",
        ),
        "https://example.com/?keep=\n"
    );
    assert_eq!(
        output(
            r#"(print (s/join "|" (url/query-get $1 "foo") (url/query-has? $1 "foo")))"#,
            "https://example.com/?foo\nhttps://example.com/\n",
        ),
        "|true\n|false\n"
    );
}

#[test]
fn url_query_operations_report_argument_errors_and_default_can_recover() {
    for (program, expected) in [
        (
            r#"(print (url/query-get $1 "key"))"#,
            "record 1: url/query-get: argument 1 expects Url (absolute URL)",
        ),
        (
            r#"(print (url/query-has? $1 "key"))"#,
            "record 1: url/query-has?: argument 1 expects Url (absolute URL)",
        ),
        (
            r#"(print (url/query-get "https://example.com/" 1))"#,
            "record 1: url/query-get: argument 2 expects String",
        ),
    ] {
        let error = cho::run(program, Cursor::new("not-a-url\n"), Vec::new()).unwrap_err();
        assert!(error.to_string().starts_with(expected), "{error}");
    }
    assert_eq!(
        output(
            r#"(print (default (url/query-get $1 "key") "invalid"))"#,
            "not-a-url\n"
        ),
        "invalid\n"
    );
}

#[test]
fn url_component_encoding_handles_ascii_unicode_and_composition() {
    assert_eq!(
        output(
            concat!(
                r#"(print (url/encode "hello world") (url/decode "hello%20world") "#,
                r#"(str "https://example.com/search?q=" (url/encode $1)) "#,
                r#"(url/encode $2) (-> $2 (url/encode) (url/decode)))"#,
            ),
            "東京 a/b?q=1+2\n",
        ),
        concat!(
            "hello%20world hello world ",
            "https://example.com/search?q=%E6%9D%B1%E4%BA%AC ",
            "a%2Fb%3Fq%3D1%2B2 a/b?q=1+2\n",
        )
    );
    assert_eq!(
        output(
            r#"(print (s/join "|" (url/encode "") (url/decode "") (url/decode "%2b+%2F")))"#,
            "x\n",
        ),
        "||++/\n"
    );
}

#[test]
fn url_component_decoding_rejects_invalid_escapes_utf8_and_non_strings() {
    for (program, expected) in [
        (
            r#"(print (url/decode "%"))"#,
            "record 1: url/decode: argument 1 expects String (URL component)",
        ),
        (
            r#"(print (url/decode "%2"))"#,
            "record 1: url/decode: argument 1 expects String (URL component)",
        ),
        (
            r#"(print (url/decode "%GG"))"#,
            "record 1: url/decode: argument 1 expects String (URL component)",
        ),
        (
            r#"(print (url/decode "%FF"))"#,
            "record 1: url/decode: argument 1 expects String (URL component)",
        ),
        (
            "(print (url/encode 10))",
            "record 1: url/encode: argument 1 expects String",
        ),
    ] {
        let error = cho::run(program, Cursor::new("x\n"), Vec::new()).unwrap_err();
        assert!(error.to_string().starts_with(expected), "{error}");
    }
}
