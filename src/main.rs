use std::env;
use std::io;
use std::process::ExitCode;

const USAGE: &str = "Usage: cho [-F SEPARATOR | --csv | --tsv] [--skip-header] 'PROGRAM'";
const HELP: &str = r#"cho — filter and format text with Lisp-like expressions

Usage: cho [-F SEPARATOR | --csv | --tsv] [--skip-header] 'PROGRAM'

Options:
  -F SEPARATOR       split input fields using this regular expression
  -FSEPARATOR        short form of -F SEPARATOR
  --csv              parse CSV records and fields, including quoted values
  --tsv              split input fields on tabs
  --skip-header      skip the first CSV or TSV record
  -h, --help         print help
  -V, --version      print version

Input:
  cho runs PROGRAM once for every input line. By default, whitespace splits fields.
  $0 is the complete line; $1, $2, ... are fields; NR is the line number; NF is
  the number of fields. A missing field is an empty string.
  In CSV or TSV mode, --skip-header skips the first logical record. NR keeps its
  input position, so the first data record is NR 2.

Types:
  String      input fields and quoted literals
  Number      numeric literals, NR, NF, and s/count results
  DateTime    RFC 3339 timestamps, normalized to UTC when rendered
  Duration    signed seconds with nanosecond precision
  IpAddr      IPv4 or IPv6 addresses
  Cidr        IPv4 or IPv6 networks

  A String is converted when an expression requires another type. A failed
  conversion stops processing with the record, expression, and argument number.
  Wrap a value expression in default to recover from an expected runtime error.

Expressions:
  (print VALUE ...)          print values separated by spaces
  (p VALUE ...)              short form of print
  (filter PREDICATE)         continue only when PREDICATE is true
  (f PREDICATE)              short form of filter

Values:
  $0, $1, ...                input line or field
  NR, NF                     line number or field count
  "text", 12, 3.5            string or number
  (str VALUE ...)                  -> String
  (s/join VALUE VALUE ...)         -> String
  (s/part DELIMITER POSITION VALUE) -> String
  (s/count VALUE)                  -> Number
  (s/escape VALUE)                 -> String
  (if PREDICATE VALUE VALUE)       -> Value
  (s/lower VALUE)                  -> String
  (s/upper VALUE)                  -> String
  (default VALUE VALUE)            -> Value
    default uses its fallback when VALUE is empty or raises a runtime error.
    s/part splits its last value by its first value as a literal delimiter and
    returns the 1-based part. The delimiter must not be empty, and requesting a
    part that does not exist is a runtime error. Empty parts are preserved; if
    the delimiter is absent, position 1 returns the complete value.

Date and duration values:
  (dt/unix NUMBER)                 -> DateTime
  (dt/fmt STRING DATETIME)         -> String
  (dt/now)                         -> DateTime (current UTC time, second precision)
  (dt/floor-s DATETIME)            -> DateTime
  (dt/floor-m DATETIME)            -> DateTime
  (dt/floor-h DATETIME)            -> DateTime
  (dt/floor-d DATETIME)            -> DateTime
  (dt/add DATETIME DURATION)       -> DateTime
  (dt/sub DATETIME DURATION)       -> DateTime
  (dt/diff DATETIME DATETIME)      -> Duration (left minus right)
  (du/s NUMBER)                   -> Duration
  (du/m NUMBER)                   -> Duration
  (du/h NUMBER)                   -> Duration

  DateTime input must be RFC 3339 and include an offset or Z. dt/unix accepts
  whole seconds, including negative values. dt/floor-* floors to a UTC second,
  minute, hour, or day boundary. Duration renders as seconds.

Predicates:
  (> NUMBER NUMBER)  (>= NUMBER NUMBER)    numeric comparisons
  (< NUMBER NUMBER)  (<= NUMBER NUMBER)
  (= NUMBER NUMBER)  (!= NUMBER NUMBER)
  (s/> STRING STRING)  (s/>= STRING STRING) string comparisons
  (s/< STRING STRING)  (s/<= STRING STRING)
  (s/= STRING STRING)  (s/!= STRING STRING)
  (dt/> DATETIME DATETIME)  (dt/>= DATETIME DATETIME)
  (dt/< DATETIME DATETIME)  (dt/<= DATETIME DATETIME)
  (dt/= DATETIME DATETIME)  (dt/!= DATETIME DATETIME)
  (ip/= IPADDR IPADDR)  (ip/!= IPADDR IPADDR)
  (ip/private? IPADDR)       true only for private IPv4 addresses
  (cidr/contains? CIDR IPADDR)
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

Threading values:
  (-> VALUE (FORM ...)) inserts VALUE as each form's first argument.
  (->> VALUE (FORM ...)) inserts VALUE as each form's last argument.

    (-> $1 (dt/add (du/s 10)))
      is (dt/add $1 (du/s 10))

    (->> $1 (dt/fmt "%Y/%m/%d") (str "date: "))
      is (str "date: " (dt/fmt "%Y/%m/%d" $1))

Examples:
  Print the first field of rows whose second field is greater than 20:
    cho '(filter (> $2 20)) (print $1)'

  Join transformed fields, using a fallback when the third field is missing:
    cho '(print (s/join ":" (s/upper $1) (default $3 "UNKNOWN")))'

  Read CSV with a header and print its first and third fields:
    cho --csv --skip-header '(print $1 $3)'"#;

#[derive(Debug, PartialEq)]
struct Options {
    field_separator: Option<String>,
    csv: bool,
    tsv: bool,
    skip_header: bool,
    program: String,
}

fn parse_args(arguments: impl IntoIterator<Item = String>) -> Result<Options, ()> {
    let mut arguments = arguments.into_iter();
    let mut field_separator = None;
    let mut csv = false;
    let mut tsv = false;
    let mut skip_header = false;
    let mut program = None;

    while let Some(argument) = arguments.next() {
        if argument == "--csv" {
            csv = true;
        } else if argument == "--tsv" {
            tsv = true;
        } else if argument == "--skip-header" {
            skip_header = true;
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

    if (csv && tsv) || ((csv || tsv) && field_separator.is_some()) || (skip_header && !(csv || tsv))
    {
        return Err(());
    }
    Ok(Options {
        field_separator,
        csv,
        tsv,
        skip_header,
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

    let program = if options.skip_header {
        format!("(f (!= NR 1)) {}", options.program)
    } else {
        options.program
    };
    let stdin = io::stdin();
    let stdout = io::stdout();
    let result = if options.csv {
        cho::run_csv(&program, stdin.lock(), stdout.lock())
    } else {
        let field_separator = options
            .field_separator
            .as_deref()
            .or(options.tsv.then_some("\\t"));
        cho::run_with_field_separator(&program, field_separator, stdin.lock(), stdout.lock())
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
                skip_header: false,
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
        assert!(parse_args(args(&["--skip-header", "(print $1)"])).is_err());
        assert!(parse_args(args(&["-F,", "--skip-header", "(print $1)"])).is_err());
    }

    #[test]
    fn parses_csv_mode() {
        assert!(parse_args(args(&["--csv", "(print $1)"])).unwrap().csv);
        assert!(
            parse_args(args(&["--csv", "--skip-header", "(print $1)"]))
                .unwrap()
                .skip_header
        );
    }

    #[test]
    fn parses_tsv_mode() {
        assert!(parse_args(args(&["--tsv", "(print $1)"])).unwrap().tsv);
        assert!(
            parse_args(args(&["--tsv", "--skip-header", "(print $1)"]))
                .unwrap()
                .skip_header
        );
    }
}
