use std::io::{self, BufRead, Write};

#[derive(Debug, PartialEq)]
pub enum ParseError {
    InvalidSyntax,
    InvalidField,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Print(Value),
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Field(usize),
}

#[derive(Debug, PartialEq)]
enum Token<'a> {
    LeftParen,
    RightParen,
    Atom(&'a str),
}

fn tokenize(program: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut start = None;

    for (index, character) in program.char_indices() {
        if character.is_whitespace() || matches!(character, '(' | ')') {
            if let Some(start) = start.take() {
                tokens.push(Token::Atom(&program[start..index]));
            }

            match character {
                '(' => tokens.push(Token::LeftParen),
                ')' => tokens.push(Token::RightParen),
                _ => {}
            }
        } else if start.is_none() {
            start = Some(index);
        }
    }

    if let Some(start) = start {
        tokens.push(Token::Atom(&program[start..]));
    }

    tokens
}

pub fn parse(program: &str) -> Result<Expr, ParseError> {
    let tokens = tokenize(program);
    let [
        Token::LeftParen,
        Token::Atom("print"),
        Token::Atom(field),
        Token::RightParen,
    ] = tokens.as_slice()
    else {
        return Err(ParseError::InvalidSyntax);
    };

    let number = field
        .strip_prefix('$')
        .ok_or(ParseError::InvalidField)?
        .parse::<usize>()
        .map_err(|_| ParseError::InvalidField)?;

    Ok(Expr::Print(Value::Field(number)))
}

pub fn run<R: BufRead, W: Write>(program: &str, input: R, mut output: W) -> io::Result<()> {
    let expression = parse(program).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid program (expected `(print $N)`, where N is 0 or greater)",
        )
    })?;

    let Expr::Print(Value::Field(field_number)) = expression;

    for line in input.lines() {
        let line = line?;
        let field = if field_number == 0 {
            line.as_str()
        } else {
            line.split_whitespace().nth(field_number - 1).unwrap_or("")
        };
        writeln!(output, "{field}")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parses_a_print_expression() {
        assert_eq!(parse("(print $2)"), Ok(Expr::Print(Value::Field(2))));
        assert_eq!(parse("(print $0)"), Ok(Expr::Print(Value::Field(0))));
    }

    #[test]
    fn allows_whitespace_around_tokens() {
        assert_eq!(parse(" ( print\t$10 ) "), Ok(Expr::Print(Value::Field(10))));
    }

    #[test]
    fn rejects_invalid_programs() {
        assert_eq!(parse("(print $x)"), Err(ParseError::InvalidField));
        assert_eq!(parse("(print $1 $2)"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("print $1"), Err(ParseError::InvalidSyntax));
    }

    #[test]
    fn prints_the_first_field_of_every_line() {
        let input = Cursor::new("Alice 20\nBob\t30\n\n");
        let mut output = Vec::new();

        run("(print $1)", input, &mut output).unwrap();

        assert_eq!(String::from_utf8(output).unwrap(), "Alice\nBob\n\n");
    }

    #[test]
    fn field_zero_prints_the_whole_line() {
        let input = Cursor::new("  Alice   20  \nBob\t30\n\n");
        let mut output = Vec::new();

        run("(print $0)", input, &mut output).unwrap();

        assert_eq!(
            String::from_utf8(output).unwrap(),
            "  Alice   20  \nBob\t30\n\n"
        );
    }

    #[test]
    fn prints_an_arbitrary_field() {
        let input = Cursor::new("Alice 20 Tokyo\nBob 30\n");
        let mut output = Vec::new();

        run("(print $3)", input, &mut output).unwrap();

        assert_eq!(String::from_utf8(output).unwrap(), "Tokyo\n\n");
    }
}
