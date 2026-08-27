use std::env;
use std::io;
use std::process::ExitCode;

const USAGE: &str =
    "Usage: cho [--no-input | -F SEPARATOR | --csv | --tsv] [--skip-header] 'PROGRAM'";
const HELP: &str = include_str!("help.txt");

#[derive(Debug, PartialEq)]
struct Options {
    field_separator: Option<String>,
    csv: bool,
    tsv: bool,
    skip_header: bool,
    no_input: bool,
    program: String,
}

fn parse_args(arguments: impl IntoIterator<Item = String>) -> Result<Options, ()> {
    let mut arguments = arguments.into_iter();
    let mut field_separator = None;
    let mut csv = false;
    let mut tsv = false;
    let mut skip_header = false;
    let mut no_input = false;
    let mut program = None;

    while let Some(argument) = arguments.next() {
        if argument == "--csv" {
            csv = true;
        } else if argument == "--tsv" {
            tsv = true;
        } else if argument == "--skip-header" {
            skip_header = true;
        } else if argument == "--no-input" {
            no_input = true;
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

    if (csv && tsv)
        || ((csv || tsv) && field_separator.is_some())
        || (skip_header && !(csv || tsv))
        || (no_input && (csv || tsv || field_separator.is_some() || skip_header))
    {
        return Err(());
    }
    Ok(Options {
        field_separator,
        csv,
        tsv,
        skip_header,
        no_input,
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
    let stdout = io::stdout();
    let result = if options.no_input {
        cho::run_no_input(&program, stdout.lock())
    } else if options.csv {
        let stdin = io::stdin();
        cho::run_csv(&program, stdin.lock(), stdout.lock())
    } else {
        let stdin = io::stdin();
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
                no_input: false,
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

    #[test]
    fn parses_no_input_mode_and_rejects_input_options() {
        assert!(
            parse_args(args(&["--no-input", "(print NR NF)"]))
                .unwrap()
                .no_input
        );
        for arguments in [
            vec!["--no-input", "--csv", "(print NR)"],
            vec!["--no-input", "--tsv", "(print NR)"],
            vec!["--no-input", "-F,", "(print NR)"],
            vec!["--no-input", "--skip-header", "(print NR)"],
        ] {
            assert!(parse_args(args(&arguments)).is_err());
        }
    }
}
