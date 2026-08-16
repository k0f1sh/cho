use std::env;
use std::io;
use std::process::ExitCode;

const USAGE: &str = "Usage: cho [-F SEPARATOR | --csv | --tsv] 'PROGRAM'";
const HELP: &str = r#"cho — filter and format text with Lisp-like expressions

Usage: cho [-F SEPARATOR | --csv | --tsv] 'PROGRAM'

Options:
  -F SEPARATOR       split input fields using this regular expression
  -FSEPARATOR        short form of -F SEPARATOR
  --csv              parse CSV records and fields, including quoted values
  --tsv              split input fields on tabs
  -h, --help         print help
  -V, --version      print version

Input:
  cho runs PROGRAM once for every input line. By default, whitespace splits fields.
  $0 is the complete line; $1, $2, ... are fields; NR is the line number; NF is
  the number of fields. A missing field is an empty string.

Expressions:
  (print VALUE ...)          print values separated by spaces
  (filter PREDICATE)         continue only when PREDICATE is true

Values:
  $0, $1, ...                input line or field
  NR, NF                     line number or field count
  "text", 12, 3.5            string or number
  (str VALUE ...)            join values without a separator
  (join SEP VALUE ...)       join values with SEP
  (count VALUE)              count Unicode characters in a value
  (escape VALUE)             escape \, newline, CR, and tab for one-line output

Predicates:
  (> A B)  (>= A B)          numeric comparisons
  (< A B)  (<= A B)
  (= A B)  (!= A B)          compare as numbers when possible, otherwise strings
  (reg /PATTERN/)            match $0 against a regular expression
  (reg VALUE /PATTERN/)      match VALUE against a regular expression
  (~ /PATTERN/)              short form of reg
  (~ VALUE /PATTERN/)
  (not PREDICATE)            invert a predicate
  (and PREDICATE ...)        true when every predicate is true
  (or PREDICATE ...)         true when any predicate is true

Regex escaping:
  Regex literals preserve backslashes; escape only a literal / as \/:
    (~ $1 /^\d+$/)
    (~ $1 /^foo\/bar$/)
  Patterns may also be strings, where each \ must be written as \\:
    (reg $1 "^\\d+$")
  The -F pattern is passed directly; quote it only for your shell:
    cho -F '\s+' '(print $1)'

Programs:
  Put one or more expressions in PROGRAM. They run from left to right for each
  input line. A failed filter skips the remaining expressions for that line;
  multiple filters therefore act like AND.

Examples:
  cho '(print NR $1 (count $1))'
  cho -F, '(print $1 $3)'
  cho --csv '(print NF (escape $9))'
  cho --tsv '(print $1 $3)'
  cho '(filter (> $2 20)) (print $1)'
  cho '(filter (or (= $1 "Alice") (~ $1 /^B/))) (print $0)'"#;

#[derive(Debug, PartialEq)]
struct Options {
    field_separator: Option<String>,
    csv: bool,
    tsv: bool,
    program: String,
}

fn parse_args(arguments: impl IntoIterator<Item = String>) -> Result<Options, ()> {
    let mut arguments = arguments.into_iter();
    let mut field_separator = None;
    let mut csv = false;
    let mut tsv = false;
    let mut program = None;

    while let Some(argument) = arguments.next() {
        if argument == "--csv" {
            csv = true;
        } else if argument == "--tsv" {
            tsv = true;
        } else if argument == "-F" {
            field_separator = Some(arguments.next().ok_or(())?);
        } else if let Some(separator) = argument.strip_prefix("-F") {
            if separator.is_empty() {
                return Err(());
            }
            field_separator = Some(separator.to_owned());
        } else if program.replace(argument).is_some() {
            return Err(());
        }
    }

    if (csv && tsv) || ((csv || tsv) && field_separator.is_some()) {
        return Err(());
    }
    Ok(Options {
        field_separator,
        csv,
        tsv,
        program: program.ok_or(())?,
    })
}

fn main() -> ExitCode {
    let mut arguments = env::args();
    let _command = arguments.next();
    let arguments = arguments.collect::<Vec<_>>();

    if arguments
        .iter()
        .any(|argument| matches!(argument.as_str(), "-h" | "--help"))
    {
        println!("{HELP}");
        return ExitCode::SUCCESS;
    }
    if arguments
        .iter()
        .any(|argument| matches!(argument.as_str(), "-V" | "--version"))
    {
        println!("cho {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }

    let Ok(options) = parse_args(arguments) else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };

    let stdin = io::stdin();
    let stdout = io::stdout();
    let result = if options.csv {
        cho::run_csv(&options.program, stdin.lock(), stdout.lock())
    } else {
        let field_separator = options
            .field_separator
            .as_deref()
            .or(options.tsv.then_some("\\t"));
        cho::run_with_field_separator(
            &options.program,
            field_separator,
            stdin.lock(),
            stdout.lock(),
        )
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("cho: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn parses_a_program() {
        assert_eq!(
            parse_args(args(&["(print $1)"])),
            Ok(Options {
                field_separator: None,
                csv: false,
                tsv: false,
                program: "(print $1)".into(),
            })
        );
    }

    #[test]
    fn parses_both_field_separator_forms() {
        assert_eq!(
            parse_args(args(&["-F", ",", "(print $1)"]))
                .unwrap()
                .field_separator,
            Some(",".into())
        );
        assert_eq!(
            parse_args(args(&["-F[,;]", "(print $1)"]))
                .unwrap()
                .field_separator,
            Some("[,;]".into())
        );
    }

    #[test]
    fn rejects_missing_or_extra_arguments() {
        assert!(parse_args(args(&[])).is_err());
        assert!(parse_args(args(&["-F"])).is_err());
        assert!(parse_args(args(&["(print $1)", "extra"])).is_err());
        assert!(parse_args(args(&["--csv", "-F,", "(print $1)"])).is_err());
        assert!(parse_args(args(&["--tsv", "-F,", "(print $1)"])).is_err());
        assert!(parse_args(args(&["--csv", "--tsv", "(print $1)"])).is_err());
    }

    #[test]
    fn parses_csv_mode() {
        assert!(parse_args(args(&["--csv", "(print $1)"])).unwrap().csv);
    }

    #[test]
    fn parses_tsv_mode() {
        assert!(parse_args(args(&["--tsv", "(print $1)"])).unwrap().tsv);
    }
}
