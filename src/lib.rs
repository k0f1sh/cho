use std::io::{self, BufRead, Write};

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnsupportedProgram,
}

/// For the first prototype, the entire language consists of one program.
pub fn parse(program: &str) -> Result<(), ParseError> {
    match program.trim() {
        "(print $1)" => Ok(()),
        _ => Err(ParseError::UnsupportedProgram),
    }
}

pub fn run<R: BufRead, W: Write>(program: &str, input: R, mut output: W) -> io::Result<()> {
    parse(program).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "unsupported program (currently only `(print $1)` is available)",
        )
    })?;

    for line in input.lines() {
        let line = line?;
        let first_field = line.split_whitespace().next().unwrap_or("");
        writeln!(output, "{first_field}")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn accepts_the_only_supported_program() {
        assert_eq!(parse("(print $1)"), Ok(()));
    }

    #[test]
    fn rejects_other_programs() {
        assert_eq!(parse("(print $2)"), Err(ParseError::UnsupportedProgram));
    }

    #[test]
    fn prints_the_first_field_of_every_line() {
        let input = Cursor::new("Alice 20\nBob\t30\n\n");
        let mut output = Vec::new();

        run("(print $1)", input, &mut output).unwrap();

        assert_eq!(String::from_utf8(output).unwrap(), "Alice\nBob\n\n");
    }
}
