use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, IsTerminal};
use std::process::ExitCode;

const USAGE: &str = concat!(
    "Usage:\n",
    "  cho [INPUT OPTIONS] 'PROGRAM'\n",
    "  cho [INPUT OPTIONS] --file FILE\n",
    "  cho [INPUT OPTIONS] --call FUNCTION [ARG ...]\n",
    "  cho --help [TOPIC]\n",
    "  cho --apropos [QUERY]\n",
    "  cho --version",
);
const HELP: &str = include_str!("help.txt");
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const CYAN: &str = "\x1b[36m";
const MAGENTA: &str = "\x1b[35m";

#[derive(Debug, PartialEq)]
struct Options {
    field_separator: Option<String>,
    csv: bool,
    tsv: bool,
    skip_header: bool,
    no_input: bool,
    program: ProgramSource,
}

#[derive(Debug, PartialEq)]
enum ProgramSource {
    CommandLine(String),
    File(String),
}

#[derive(Debug, PartialEq)]
enum Command {
    Run(Options),
    Help(Option<String>),
    Version,
    Apropos(Option<String>),
}

#[derive(Debug, PartialEq)]
enum ArgumentError {
    MissingProgram,
    MissingSeparator,
    MissingProgramFile,
    UnexpectedArgument(String),
    ConflictingInputFormats,
    SkipHeaderWithoutDelimitedInput,
    NoInputWithInputOptions,
    CallAfterProgram,
    MissingCallFunction,
    InvalidCallFunction,
    CallExpression,
    ConflictingProgramSources,
    EmptyAproposQuery,
    InformationCommandCannotBeCombined(String),
}

impl std::fmt::Display for ArgumentError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingProgram => formatter.write_str("missing PROGRAM"),
            Self::MissingSeparator => formatter.write_str("-F expects SEPARATOR"),
            Self::MissingProgramFile => formatter.write_str("--file expects FILE"),
            Self::UnexpectedArgument(argument) => {
                write!(formatter, "unexpected argument: {argument}")
            }
            Self::ConflictingInputFormats => {
                formatter.write_str("--csv, --tsv, and -F are mutually exclusive")
            }
            Self::SkipHeaderWithoutDelimitedInput => {
                formatter.write_str("--skip-header requires --csv or --tsv")
            }
            Self::NoInputWithInputOptions => formatter.write_str(
                "--no-input cannot be combined with -F, --csv, --tsv, or --skip-header",
            ),
            Self::CallAfterProgram => formatter.write_str("--call must precede PROGRAM"),
            Self::MissingCallFunction => formatter.write_str("--call expects FUNCTION"),
            Self::InvalidCallFunction => formatter.write_str(
                "--call expects a function name without whitespace, parentheses, or quotes",
            ),
            Self::CallExpression => formatter.write_str(
                "--call expects FUNCTION without parentheses; use -n 'PROGRAM' to evaluate an expression without input",
            ),
            Self::ConflictingProgramSources => {
                formatter.write_str("--file cannot be combined with PROGRAM or --call")
            }
            Self::EmptyAproposQuery => formatter.write_str("--apropos expects a non-empty QUERY"),
            Self::InformationCommandCannotBeCombined(option) => write!(
                formatter,
                "{option} must be used without PROGRAM or input options"
            ),
        }
    }
}

fn help_color_enabled(
    stdout_is_terminal: bool,
    no_color: Option<&OsStr>,
    term: Option<&OsStr>,
) -> bool {
    stdout_is_terminal && no_color.is_none() && term != Some(OsStr::new("dumb"))
}

fn colorize_help(help: &str) -> String {
    let mut output = String::with_capacity(help.len() + 512);
    for (index, line) in help.lines().enumerate() {
        if index > 0 {
            output.push('\n');
        }
        if index == 0 {
            output.push_str(BOLD);
            output.push_str(line);
            output.push_str(RESET);
        } else if !line.starts_with(char::is_whitespace) && line.ends_with(':') {
            output.push_str(BOLD);
            output.push_str(CYAN);
            output.push_str(line);
            output.push_str(RESET);
        } else if let Some(styled) = colorize_signature(line) {
            output.push_str(&styled);
        } else if line.starts_with("    ")
            && matches!(line.trim_start(), command if command.starts_with("cho ") || command.starts_with("printf "))
        {
            output.push_str(DIM);
            output.push_str(line);
            output.push_str(RESET);
        } else if let Some((name, rest)) = split_type_definition(line) {
            output.push_str("  ");
            output.push_str(MAGENTA);
            output.push_str(name);
            output.push_str(RESET);
            output.push_str(rest);
        } else {
            output.push_str(line);
        }
    }
    output
}

fn colorize_signature(line: &str) -> Option<String> {
    let syntax = line.strip_prefix("  ")?;
    if !(syntax.starts_with('(') || syntax.starts_with('-')) {
        return None;
    }
    let description = syntax.find("  ").unwrap_or(syntax.len());
    Some(format!(
        "  {CYAN}{}{RESET}{}",
        &syntax[..description],
        &syntax[description..]
    ))
}

fn split_type_definition(line: &str) -> Option<(&str, &str)> {
    let definition = line.strip_prefix("  ")?;
    let name_end = definition.find(char::is_whitespace)?;
    let name = &definition[..name_end];
    matches!(
        name,
        "String"
            | "Number"
            | "Boolean"
            | "Date"
            | "DateTime"
            | "Duration"
            | "IpAddr"
            | "Cidr"
            | "Url"
            | "SemVer"
    )
    .then_some((name, &definition[name_end..]))
}

fn parse_args(arguments: impl IntoIterator<Item = String>) -> Result<Options, ArgumentError> {
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
        } else if matches!(argument.as_str(), "-s" | "--skip-header") {
            skip_header = true;
        } else if matches!(argument.as_str(), "-n" | "--no-input") {
            no_input = true;
        } else if matches!(argument.as_str(), "-f" | "--file") {
            if program.is_some() {
                return Err(ArgumentError::ConflictingProgramSources);
            }
            let path = arguments.next().ok_or(ArgumentError::MissingProgramFile)?;
            program = Some(ProgramSource::File(path));
        } else if is_call_option(&argument) {
            if program.is_some() {
                return if matches!(program, Some(ProgramSource::File(_))) {
                    Err(ArgumentError::ConflictingProgramSources)
                } else {
                    Err(ArgumentError::CallAfterProgram)
                };
            }
            if matches!(argument.as_str(), "-nc" | "-cn") {
                no_input = true;
            }
            let function = arguments.next().ok_or(ArgumentError::MissingCallFunction)?;
            program = Some(ProgramSource::CommandLine(call_program(
                &function, arguments, !no_input,
            )?));
            break;
        } else if argument == "-F" {
            field_separator = Some(arguments.next().ok_or(ArgumentError::MissingSeparator)?);
        } else if let Some(separator) = argument.strip_prefix("-F") {
            if separator.is_empty() {
                return Err(ArgumentError::MissingSeparator);
            }
            field_separator = Some(separator.to_owned());
        } else if is_information_option(&argument) {
            return Err(ArgumentError::InformationCommandCannotBeCombined(argument));
        } else if program.is_some() {
            return if matches!(program, Some(ProgramSource::File(_))) {
                Err(ArgumentError::ConflictingProgramSources)
            } else {
                Err(ArgumentError::UnexpectedArgument(argument))
            };
        } else {
            program = Some(ProgramSource::CommandLine(argument));
        }
    }

    if no_input && (csv || tsv || field_separator.is_some() || skip_header) {
        return Err(ArgumentError::NoInputWithInputOptions);
    }
    if (csv && tsv) || ((csv || tsv) && field_separator.is_some()) {
        return Err(ArgumentError::ConflictingInputFormats);
    }
    if skip_header && !(csv || tsv) {
        return Err(ArgumentError::SkipHeaderWithoutDelimitedInput);
    }
    Ok(Options {
        field_separator,
        csv,
        tsv,
        skip_header,
        no_input,
        program: program.ok_or(ArgumentError::MissingProgram)?,
    })
}

fn parse_command(arguments: Vec<String>) -> Result<Command, ArgumentError> {
    let mut arguments = arguments.into_iter();
    let Some(first) = arguments.next() else {
        return Err(ArgumentError::MissingProgram);
    };
    match first.as_str() {
        "-h" | "--help" => {
            let topic = arguments.next();
            if topic.as_deref().is_some_and(is_cli_option) {
                return Err(ArgumentError::InformationCommandCannotBeCombined(first));
            }
            if let Some(argument) = arguments.next() {
                return Err(ArgumentError::UnexpectedArgument(argument));
            }
            Ok(Command::Help(topic))
        }
        "-V" | "--version" => {
            if arguments.next().is_some() {
                return Err(ArgumentError::InformationCommandCannotBeCombined(first));
            }
            Ok(Command::Version)
        }
        "-k" | "--apropos" => {
            let query = arguments.next();
            if query.as_deref() == Some("") {
                return Err(ArgumentError::EmptyAproposQuery);
            }
            if let Some(argument) = arguments.next() {
                return Err(ArgumentError::UnexpectedArgument(argument));
            }
            Ok(Command::Apropos(query))
        }
        _ => parse_args(std::iter::once(first).chain(arguments)).map(Command::Run),
    }
}

fn is_information_option(argument: &str) -> bool {
    matches!(
        argument,
        "-h" | "--help" | "-V" | "--version" | "-k" | "--apropos"
    )
}

fn is_cli_option(argument: &str) -> bool {
    is_information_option(argument)
        || is_call_option(argument)
        || matches!(
            argument,
            "--csv"
                | "--tsv"
                | "-s"
                | "--skip-header"
                | "-n"
                | "--no-input"
                | "-F"
                | "-f"
                | "--file"
        )
        || argument.starts_with("-F")
}

fn is_call_option(argument: &str) -> bool {
    matches!(argument, "-c" | "--call" | "-nc" | "-cn")
}

fn call_program(
    function: &str,
    arguments: impl IntoIterator<Item = String>,
    include_record: bool,
) -> Result<String, ArgumentError> {
    if function.starts_with('(') || function.ends_with(')') {
        return Err(ArgumentError::CallExpression);
    }
    if function.is_empty()
        || function
            .chars()
            .any(|character| character.is_whitespace() || matches!(character, '(' | ')' | '"'))
    {
        return Err(ArgumentError::InvalidCallFunction);
    }

    let mut program = format!("({function}");
    if include_record {
        program.push_str(" $0");
    }
    for argument in arguments {
        program.push(' ');
        if is_call_reference(&argument) {
            program.push_str(&argument);
        } else {
            program.push('"');
            for character in argument.chars() {
                match character {
                    '\\' => program.push_str("\\\\"),
                    '"' => program.push_str("\\\""),
                    '\n' => program.push_str("\\n"),
                    '\t' => program.push_str("\\t"),
                    other => program.push(other),
                }
            }
            program.push('"');
        }
    }
    program.push(')');
    Ok(program)
}

fn is_call_reference(argument: &str) -> bool {
    argument.strip_prefix('$').is_some_and(|index| {
        !index.is_empty() && index.chars().all(|character| character.is_ascii_digit())
    })
}

fn main() -> ExitCode {
    let mut arguments = env::args();
    let _command = arguments.next();
    let command = match parse_command(arguments.collect()) {
        Ok(command) => command,
        Err(error) => {
            eprintln!("cho: {error}");
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };

    if let Command::Help(topic) = &command {
        let topic_help = match topic {
            Some(topic) => match cho::help(topic) {
                Some(help) => Some(help),
                None => {
                    eprintln!("cho: no such help topic: {topic}");
                    eprintln!("Try 'cho --help' for all functions and forms.");
                    return ExitCode::from(2);
                }
            },
            None => None,
        };
        let help = topic_help.as_deref().unwrap_or(HELP);
        let stdout = io::stdout();
        if help_color_enabled(
            stdout.is_terminal(),
            env::var_os("NO_COLOR").as_deref(),
            env::var_os("TERM").as_deref(),
        ) {
            println!("{}", colorize_help(help));
        } else {
            println!("{help}");
        }
        return ExitCode::SUCCESS;
    }
    if command == Command::Version {
        println!("cho {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    if let Command::Apropos(query) = &command {
        let output = match query {
            Some(query) => {
                let Some(output) = cho::apropos(query) else {
                    eprintln!("cho: no functions or forms match: {query}");
                    return ExitCode::FAILURE;
                };
                output
            }
            None => cho::catalog(),
        };
        let stdout = io::stdout();
        if help_color_enabled(
            stdout.is_terminal(),
            env::var_os("NO_COLOR").as_deref(),
            env::var_os("TERM").as_deref(),
        ) {
            println!("{}", colorize_apropos(&output));
        } else {
            println!("{output}");
        }
        return ExitCode::SUCCESS;
    }
    let Command::Run(options) = command else {
        unreachable!("information commands returned above");
    };

    let program = match options.program {
        ProgramSource::CommandLine(program) => program,
        ProgramSource::File(path) => match fs::read_to_string(&path) {
            Ok(program) => program,
            Err(error) => {
                eprintln!("cho: cannot read program file {path:?}: {error}");
                return ExitCode::FAILURE;
            }
        },
    };
    let program = if options.skip_header {
        format!("(f (!= NR 1)) {program}")
    } else {
        program
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
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("cho: {error}");
            ExitCode::FAILURE
        }
    }
}

fn colorize_apropos(output: &str) -> String {
    output
        .lines()
        .map(|line| {
            let name_end = line.find("  ").unwrap_or(line.len());
            format!("{CYAN}{}{RESET}{}", &line[..name_end], &line[name_end..])
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn help_color_requires_a_supported_terminal() {
        assert!(help_color_enabled(
            true,
            None,
            Some(OsStr::new("xterm-256color"))
        ));
        assert!(!help_color_enabled(
            false,
            None,
            Some(OsStr::new("xterm-256color"))
        ));
        assert!(!help_color_enabled(true, Some(OsStr::new("")), None));
        assert!(!help_color_enabled(true, None, Some(OsStr::new("dumb"))));
    }

    #[test]
    fn colored_help_styles_structure_without_changing_text() {
        let plain = concat!(
            "cho — description\n\n",
            "Strings:\n",
            "  (s/count VALUE) -> NUMBER    count characters\n",
            "  String      a string value\n",
            "    cho '(p $1)'\n",
            "  ordinary explanation"
        );
        let colored = colorize_help(plain);
        assert!(colored.contains("\x1b[1mcho — description\x1b[0m"));
        assert!(colored.contains("\x1b[1m\x1b[36mStrings:\x1b[0m"));
        assert!(colored.contains("\x1b[36m(s/count VALUE) -> NUMBER\x1b[0m"));
        assert!(colored.contains("\x1b[35mString\x1b[0m"));
        assert!(colored.contains("\x1b[2m    cho '(p $1)'\x1b[0m"));
        assert_eq!(strip_ansi(&colored), plain);
        assert_eq!(strip_ansi(&colorize_help(HELP)), HELP);
    }

    #[test]
    fn colored_apropos_styles_names_without_changing_text() {
        let plain = "s/dquote (dq)  stringify and quote\ns/unquote      remove quotes";
        let colored = colorize_apropos(plain);
        assert!(colored.contains("\x1b[36ms/dquote (dq)\x1b[0m"));
        assert!(colored.contains("\x1b[36ms/unquote\x1b[0m"));
        assert_eq!(strip_ansi(&colored), plain);
    }

    fn strip_ansi(value: &str) -> String {
        [BOLD, DIM, CYAN, MAGENTA, RESET]
            .iter()
            .fold(value.to_owned(), |value, code| value.replace(code, ""))
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
                program: ProgramSource::CommandLine("(print $1)".into()),
            })
        );
    }

    #[test]
    fn parses_standalone_information_commands() {
        assert_eq!(parse_command(args(&["--help"])), Ok(Command::Help(None)));
        assert_eq!(
            parse_command(args(&["-h", "s/trim"])),
            Ok(Command::Help(Some("s/trim".into())))
        );
        assert_eq!(parse_command(args(&["--version"])), Ok(Command::Version));
        assert_eq!(
            parse_command(args(&["-k", "trim"])),
            Ok(Command::Apropos(Some("trim".into())))
        );
        assert_eq!(parse_command(args(&["-k"])), Ok(Command::Apropos(None)));
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
    fn parses_a_program_file() {
        for option in ["-f", "--file"] {
            assert_eq!(
                parse_args(args(&[option, "program.cho"])).unwrap().program,
                ProgramSource::File("program.cho".into())
            );
        }
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
        assert!(parse_args(args(&["-s", "(print $1)"])).is_err());
        assert!(parse_args(args(&["-F,", "--skip-header", "(print $1)"])).is_err());
    }

    #[test]
    fn parses_csv_mode() {
        assert!(parse_args(args(&["--csv", "(print $1)"])).unwrap().csv);
        for option in ["-s", "--skip-header"] {
            assert!(
                parse_args(args(&["--csv", option, "(print $1)"]))
                    .unwrap()
                    .skip_header
            );
        }
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
        for option in ["-n", "--no-input"] {
            assert!(
                parse_args(args(&[option, "(print NR NF)"]))
                    .unwrap()
                    .no_input
            );
        }
        for arguments in [
            vec!["--no-input", "--csv", "(print NR)"],
            vec!["-n", "--tsv", "(print NR)"],
            vec!["--no-input", "-F,", "(print NR)"],
            vec!["-n", "--skip-header", "(print NR)"],
            vec!["-n", "-s", "(print NR)"],
        ] {
            assert!(parse_args(args(&arguments)).is_err());
        }
    }

    #[test]
    fn call_mode_builds_one_function_call() {
        assert_eq!(call_program("s/upper", [], true).unwrap(), "(s/upper $0)");
        assert_eq!(
            call_program("str", args(&["$1", "$2", "NR", "NF"]), true).unwrap(),
            r#"(str $0 $1 $2 "NR" "NF")"#
        );
        assert_eq!(
            call_program("str", args(&["a b", "a\\b", "a\"b", "a\nb", "a\tb"]), true,).unwrap(),
            r#"(str $0 "a b" "a\\b" "a\"b" "a\nb" "a\tb")"#
        );
        assert_eq!(
            parse_args(args(&["-n", "--call", "s/upper", "hoge"]))
                .unwrap()
                .program,
            ProgramSource::CommandLine(r#"(s/upper "hoge")"#.into())
        );
        for option in ["-nc", "-cn"] {
            let options = parse_args(args(&[option, "s/upper", "hoge"])).unwrap();
            assert!(options.no_input);
            assert_eq!(
                options.program,
                ProgramSource::CommandLine(r#"(s/upper "hoge")"#.into())
            );
        }
    }

    #[test]
    fn call_mode_rejects_a_missing_or_invalid_function_name() {
        assert!(parse_args(args(&["-c"])).is_err());
        assert!(parse_args(args(&["-c", "s/upper $1"])).is_err());
        assert!(parse_args(args(&["(p $0)", "-c", "s/upper"])).is_err());
        assert!(parse_args(args(&["-s", "s/upper"])).is_err());
        assert_eq!(
            parse_args(args(&["-nc", "(ulid/new)"])),
            Err(ArgumentError::CallExpression)
        );
    }
}
