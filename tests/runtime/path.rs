use super::support::output;
use std::io::Cursor;

#[test]
fn extracts_path_components_and_composes_with_values() {
    assert_eq!(
        output(
            concat!(
                r#"(print (path/name $1) (path/stem $1) (path/ext $1) (path/dir $1)) "#,
                r#"(print (s/upper (path/ext $2)) (-> $2 path/name s/upper))"#,
            ),
            "/var/log/app.log archive.tar.gz\n",
        ),
        "app.log app log /var/log\nGZ ARCHIVE.TAR.GZ\n"
    );
}

#[test]
fn handles_lexical_unix_path_boundaries() {
    assert_eq!(
        output(
            r#"(print (s/join "|" (path/name $1) (path/stem $1) (path/ext $1) (path/dir $1)))"#,
            concat!(
                "/\n",
                "/var/log/\n",
                "file.txt\n",
                ".gitignore\n",
                ".config.json\n",
                "file.\n",
                ".\n",
                "..\n",
                "a//b\n",
                r"windows\path.txt",
            ),
        ),
        concat!(
            "|||/\n",
            "log|log||/var\n",
            "file.txt|file|txt|\n",
            ".gitignore|.gitignore||\n",
            ".config.json|.config|json|\n",
            "file.|file.||\n",
            ".|.||\n",
            "..|..||\n",
            "b|b||a\n",
            "windows\\path.txt|windows\\path|txt|\n",
        )
    );
    assert_eq!(
        output(
            r#"(print (s/join "|" (path/name $2) (path/stem $2) (path/ext $2) (path/dir $2)))"#,
            "only-one-field\n",
        ),
        "|||\n"
    );
}

#[test]
fn rejects_non_string_runtime_values() {
    let error = cho::run("(print (path/ext 10))", Cursor::new("x\n"), Vec::new()).unwrap_err();
    assert!(
        error
            .to_string()
            .starts_with("record 1: path/ext: argument 1 expects String"),
        "{error}"
    );
}
