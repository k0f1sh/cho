use std::env;
use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args();
    let command = args.next().unwrap_or_else(|| "cho".to_owned());
    let mut field_separator = None;
    let mut program = None;

    while let Some(argument) = args.next() {
        if argument == "-F" {
            let Some(separator) = args.next() else {
                eprintln!("usage: {command} [-F separator] 'program'");
                return ExitCode::from(2);
            };
            field_separator = Some(separator);
        } else if let Some(separator) = argument.strip_prefix("-F") {
            if separator.is_empty() {
                eprintln!("usage: {command} [-F separator] 'program'");
                return ExitCode::from(2);
            }
            field_separator = Some(separator.to_owned());
        } else if program.replace(argument).is_some() {
            eprintln!("usage: {command} [-F separator] 'program'");
            return ExitCode::from(2);
        }
    }

    let Some(program) = program else {
        eprintln!("usage: {command} [-F separator] 'program'");
        return ExitCode::from(2);
    };

    let stdin = io::stdin();
    let stdout = io::stdout();

    match cho::run_with_field_separator(
        &program,
        field_separator.as_deref(),
        stdin.lock(),
        stdout.lock(),
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("cho: {error}");
            ExitCode::FAILURE
        }
    }
}
