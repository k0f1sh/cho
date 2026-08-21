use std::env;
use std::io;
use std::process::ExitCode;

const USAGE: &str = "Usage: cho [-F SEPARATOR | --csv | --tsv] [--skip-header] 'PROGRAM'";
const HELP: &str = r#"cho — a tiny semantic awk with Lisp-like expressions

Usage: cho [-F SEPARATOR | --csv | --tsv] [--skip-header] 'PROGRAM'

Common recipes:
  Pick fields             cho '(p $1 $3)'
  Filter numbers          cho '(f (> $2 20)) (p $0)'
  Match a field           cho '(f (~ $1 /^api-/)) (p $0)'
  Match the whole record  cho '(f (~ /ERROR|WARN/)) (p $0)'
  Join values             cho '(p (s/join "," $1 $2))'
  Use a fallback          cho '(p (default $3 "unknown"))'
  Read CSV with a header  cho --csv --skip-header '(p $1 $3)'
  Read TSV                cho --tsv '(p $1 $2)'

Input and options:
  -F SEPARATOR       split fields using this regular expression
  -FSEPARATOR        short form of -F SEPARATOR
  --csv              parse CSV records, including quoted values
  --tsv              split fields on tabs
  --skip-header      skip the first CSV or TSV record
  -h, --help         print help
  -V, --version      print version

  PROGRAM runs once per input record. Whitespace splits fields by default.
  $0 is the complete record; $1, $2, ... are fields. NR is the record number;
  NF is the field count. Missing fields are empty strings. With --skip-header,
  the first data record keeps its input position and therefore has NR 2.

Program expressions:
  (print VALUE ...)              print values separated by spaces
  (p VALUE ...)                  short form of print
  (filter BOOLEAN)               continue only when true
  (f BOOLEAN)                    short form of filter

Literals and fields:
  $0, $1, ...                    complete record or field
  NR, NF                         record number or field count
  "text", 12, 3.5                String or Number
  true, false                    Boolean

Numbers:
  (+ NUMBER NUMBER)              add
  (- NUMBER NUMBER)              subtract
  (* NUMBER NUMBER)              multiply
  (/ NUMBER NUMBER)              divide
  (n/fixed DIGITS NUMBER)        -> String with 0 to 100 fractional digits
  (> NUMBER NUMBER)  (>= NUMBER NUMBER)
  (< NUMBER NUMBER)  (<= NUMBER NUMBER)
  (= NUMBER NUMBER)  (!= NUMBER NUMBER)

Strings:
  (str VALUE ...)                concatenate values
  (s/join SEPARATOR VALUE ...)   join values
  (s/part DELIMITER POSITION VALUE) take a 1-based literal-delimited part
  (s/count VALUE)                count Unicode characters
  (s/escape VALUE)               escape tabs, newlines, and backslashes
  (s/lower VALUE)                lowercase
  (s/upper VALUE)                uppercase
  (s/> STRING STRING)  (s/>= STRING STRING)
  (s/< STRING STRING)  (s/<= STRING STRING)
  (s/= STRING STRING)  (s/!= STRING STRING)

  s/part preserves empty parts. If no delimiter is found, position 1 returns
  the complete value; a missing position returns an empty string. DELIMITER
  must not be empty.

Selection and recovery:
  (if BOOLEAN VALUE VALUE)       select one value lazily
  (default VALUE FALLBACK)       use FALLBACK when VALUE is empty or errors
  (not BOOLEAN)                  negate
  (and BOOLEAN ...)              true when every value is true
  (or BOOLEAN ...)               true when any value is true

Regular expressions:
  (reg /PATTERN/)                match $0
  (reg VALUE /PATTERN/)          match VALUE
  (~ /PATTERN/)                  short form of reg
  (~ VALUE /PATTERN/)

  Regex literals preserve backslashes; escape only a literal / as \/:
    (~ $1 /^\d+$/)     (~ $1 /^foo\/bar$/)
  String patterns require doubled backslashes: (reg $1 "^\\d+$")
  The -F pattern is passed directly: cho -F '\s+' '(p $1)'

DateTime and Duration:
  (dt/unix NUMBER)               Unix seconds -> DateTime
  (dt/fmt STRING DATETIME)       format in UTC -> String
  (dt/fmt STRING TIMEZONE DATETIME) -> String
  (dt/now)                       current UTC time, second precision
  (dt/floor-s DATETIME)          floor to UTC second
  (dt/floor-m DATETIME)          floor to UTC minute
  (dt/floor-h DATETIME)          floor to UTC hour
  (dt/floor-d DATETIME)          floor to UTC day
  (dt/add DATETIME DURATION)     -> DateTime
  (dt/sub DATETIME DURATION)     -> DateTime
  (dt/diff DATETIME DATETIME)    left minus right -> Duration
  (du/s NUMBER)                  seconds -> Duration
  (du/ms NUMBER)                 milliseconds -> Duration
  (du/m NUMBER)                  minutes -> Duration
  (du/h NUMBER)                  hours -> Duration
  (du/d NUMBER)                  fixed 24-hour days -> Duration
  (dt/> DATETIME DATETIME)  (dt/>= DATETIME DATETIME)
  (dt/< DATETIME DATETIME)  (dt/<= DATETIME DATETIME)
  (dt/= DATETIME DATETIME)  (dt/!= DATETIME DATETIME)

  DateTime input must be RFC 3339 with an offset or Z. DateTime renders in UTC;
  Duration renders as seconds. TIMEZONE is an IANA name such as Asia/Tokyo or a
  fixed offset such as +09:00. dt/floor-* always uses UTC boundaries.

IP and CIDR:
  (ip/version IPADDR)            -> Number (4 or 6)
  (ip/= IPADDR IPADDR)           -> Boolean
  (ip/!= IPADDR IPADDR)          -> Boolean
  (ip/private? IPADDR)           RFC 1918 IPv4 or fc00::/7 IPv6 ULA
  (ip/loopback? IPADDR)          -> Boolean
  (ip/link-local? IPADDR)        -> Boolean
  (ip/multicast? IPADDR)         -> Boolean
  (cidr/contains? CIDR IPADDR)   -> Boolean
  (cidr/network CIDR)            -> IpAddr
  (cidr/prefix CIDR)             -> Number
  (cidr/first CIDR)              -> IpAddr (lowest address)
  (cidr/last CIDR)               -> IpAddr (highest address)
  (cidr/size CIDR)               -> Number (must be at most 2^53 - 1)

URLs:
  (url/scheme URL)               -> String
  (url/host URL)                 -> String
  (url/port URL)                 -> String
  (url/path URL)                 -> String
  (url/query URL)                -> String
  (url/fragment URL)             -> String
  (url/query-get STRING URL)     first decoded value or empty -> String
  (url/query-has? STRING URL)    -> Boolean
  (url/encode STRING)            RFC 3986 component encoding -> String
  (url/decode STRING)            percent decoding -> String

  Extracted components preserve percent encoding. Query operations use form
  semantics (+ is space), decode names and values, and use the first duplicate.
  url/encode uses uppercase escapes. url/decode decodes only %XX and leaves +
  unchanged. Invalid URLs, escapes, or UTF-8 are errors.

Semantic versions:
  (semver/> SEMVER SEMVER)  (semver/>= SEMVER SEMVER)
  (semver/< SEMVER SEMVER)  (semver/<= SEMVER SEMVER)
  (semver/= SEMVER SEMVER)  (semver/!= SEMVER SEMVER)
  (semver/major SEMVER)           -> Number
  (semver/minor SEMVER)           -> Number
  (semver/patch SEMVER)           -> Number
  (semver/prerelease SEMVER)      -> String (empty when absent)

Types and errors:
  String      fields and quoted literals    DateTime  RFC 3339 timestamps
  Number      numeric literals, NR, NF       Duration  signed seconds
  Boolean     true, false, predicates        IpAddr    IPv4 or IPv6
  Cidr        IPv4 or IPv6 networks          Url       absolute URLs
  SemVer      MAJOR.MINOR.PATCH versions

  Expressions convert Strings when they require another type. Failed conversion
  reports the record, expression, and argument number. Boolean values never
  convert implicitly from strings or numbers. Numeric arithmetic is binary;
  division by zero and non-finite results are errors.

Composition:
  Values nest anywhere a VALUE is accepted. Multiple program expressions run
  left to right; a failed filter skips the rest of that record. Multiple filters
  therefore act like AND.

  (-> VALUE (FORM ...))   insert VALUE as each form's first argument
  (->> VALUE (FORM ...))  insert VALUE as each form's last argument

    (-> $1 (dt/add (du/s 10)))
      is (dt/add $1 (du/s 10))

    (->> $1 (dt/fmt "%Y/%m/%d") (str "date: "))
      is (str "date: " (dt/fmt "%Y/%m/%d" $1))

More examples:
  Format an RFC 3339 timestamp in Tokyo time:
    cho '(p (dt/fmt "%Y-%m-%d %H:%M" "Asia/Tokyo" $1))'

  Join transformed fields, with a fallback for a missing third field:
    cho '(p (s/join ":" (s/upper $1) (default $3 "UNKNOWN")))'"#;

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
