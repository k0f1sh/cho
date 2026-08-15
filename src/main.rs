use std::env;
use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args();
    let command = args.next().unwrap_or_else(|| "lispawk".to_owned());

    let Some(program) = args.next() else {
        eprintln!("usage: {command} '(print $1)'");
        return ExitCode::from(2);
    };

    if args.next().is_some() {
        eprintln!("usage: {command} '(print $1)'");
        return ExitCode::from(2);
    }

    let stdin = io::stdin();
    let stdout = io::stdout();

    match lispawk::run(&program, stdin.lock(), stdout.lock()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("lispawk: {error}");
            ExitCode::FAILURE
        }
    }
}
