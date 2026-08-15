use std::io::{self, BufRead, Write};

#[derive(Debug, PartialEq)]
pub enum ParseError {
    InvalidSyntax,
    InvalidField,
    UnterminatedString,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Print(Vec<Value>),
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Field(usize),
    String(String),
    Format(Vec<Value>),
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    LeftParen,
    RightParen,
    Atom(String),
    String(String),
}

fn tokenize(program: &str) -> Result<Vec<Token>, ParseError> {
    let mut characters = program.chars().peekable();
    let mut tokens = Vec::new();

    while let Some(character) = characters.next() {
        match character {
            character if character.is_whitespace() => {}
            '(' => tokens.push(Token::LeftParen),
            ')' => tokens.push(Token::RightParen),
            '"' => {
                let mut value = String::new();
                let mut terminated = false;

                while let Some(character) = characters.next() {
                    match character {
                        '"' => {
                            terminated = true;
                            break;
                        }
                        '\\' => {
                            let escaped =
                                characters.next().ok_or(ParseError::UnterminatedString)?;
                            value.push(match escaped {
                                'n' => '\n',
                                't' => '\t',
                                other => other,
                            });
                        }
                        other => value.push(other),
                    }
                }

                if !terminated {
                    return Err(ParseError::UnterminatedString);
                }
                tokens.push(Token::String(value));
            }
            first => {
                let mut atom = String::from(first);
                while let Some(character) = characters.peek() {
                    if character.is_whitespace() || matches!(character, '(' | ')' | '"') {
                        break;
                    }
                    atom.push(characters.next().expect("peeked character must exist"));
                }
                tokens.push(Token::Atom(atom));
            }
        }
    }

    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.position).cloned();
        self.position += usize::from(token.is_some());
        token
    }

    fn parse_program(&mut self) -> Result<Expr, ParseError> {
        if self.next() != Some(Token::LeftParen)
            || self.next() != Some(Token::Atom("print".to_owned()))
        {
            return Err(ParseError::InvalidSyntax);
        }

        let values = self.parse_values_until_right_paren()?;
        if self.next().is_some() {
            return Err(ParseError::InvalidSyntax);
        }

        Ok(Expr::Print(values))
    }

    fn parse_values_until_right_paren(&mut self) -> Result<Vec<Value>, ParseError> {
        let mut values = Vec::new();
        loop {
            match self.tokens.get(self.position) {
                Some(Token::RightParen) => {
                    self.position += 1;
                    return Ok(values);
                }
                Some(_) => values.push(self.parse_value()?),
                None => return Err(ParseError::InvalidSyntax),
            }
        }
    }

    fn parse_value(&mut self) -> Result<Value, ParseError> {
        match self.next() {
            Some(Token::Atom(field)) => {
                let number = field
                    .strip_prefix('$')
                    .ok_or(ParseError::InvalidSyntax)?
                    .parse::<usize>()
                    .map_err(|_| ParseError::InvalidField)?;
                Ok(Value::Field(number))
            }
            Some(Token::String(value)) => Ok(Value::String(value)),
            Some(Token::LeftParen) => {
                if self.next() != Some(Token::Atom("fmt".to_owned())) {
                    return Err(ParseError::InvalidSyntax);
                }
                Ok(Value::Format(self.parse_values_until_right_paren()?))
            }
            _ => Err(ParseError::InvalidSyntax),
        }
    }
}

pub fn parse(program: &str) -> Result<Expr, ParseError> {
    Parser::new(tokenize(program)?).parse_program()
}

fn evaluate(value: &Value, line: &str) -> String {
    match value {
        Value::Field(0) => line.to_owned(),
        Value::Field(number) => line
            .split_whitespace()
            .nth(number - 1)
            .unwrap_or("")
            .to_owned(),
        Value::String(value) => value.clone(),
        Value::Format(values) => values.iter().map(|value| evaluate(value, line)).collect(),
    }
}

pub fn run<R: BufRead, W: Write>(program: &str, input: R, mut output: W) -> io::Result<()> {
    let expression = parse(program).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid program (expected a print expression)",
        )
    })?;

    let Expr::Print(values) = expression;
    for line in input.lines() {
        let line = line?;
        let rendered = values
            .iter()
            .map(|value| evaluate(value, &line))
            .collect::<Vec<_>>()
            .join(" ");
        writeln!(output, "{rendered}")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn output_for(program: &str, input: &str) -> String {
        let mut output = Vec::new();
        run(program, Cursor::new(input), &mut output).unwrap();
        String::from_utf8(output).unwrap()
    }

    #[test]
    fn parses_print_with_multiple_values() {
        assert_eq!(
            parse("(print $1 $2)"),
            Ok(Expr::Print(vec![Value::Field(1), Value::Field(2)]))
        );
    }

    #[test]
    fn parses_strings_and_nested_formats() {
        assert_eq!(
            parse(r#"(print (fmt $1 ":" $2) "points")"#),
            Ok(Expr::Print(vec![
                Value::Format(vec![
                    Value::Field(1),
                    Value::String(":".to_owned()),
                    Value::Field(2),
                ]),
                Value::String("points".to_owned()),
            ]))
        );
    }

    #[test]
    fn rejects_invalid_programs() {
        assert_eq!(parse("(print $x)"), Err(ParseError::InvalidField));
        assert_eq!(parse("print $1"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(unknown $1)"), Err(ParseError::InvalidSyntax));
        assert_eq!(
            parse("(print (unknown $1))"),
            Err(ParseError::InvalidSyntax)
        );
        assert_eq!(
            parse("(print \"unfinished)"),
            Err(ParseError::UnterminatedString)
        );
    }

    #[test]
    fn print_separates_values_with_spaces() {
        assert_eq!(
            output_for("(print $1 $2)", "Alice 20\nBob 30\n"),
            "Alice 20\nBob 30\n"
        );
    }

    #[test]
    fn print_with_no_values_prints_an_empty_line() {
        assert_eq!(output_for("(print)", "Alice\nBob\n"), "\n\n");
    }

    #[test]
    fn strings_support_spaces_and_escapes() {
        assert_eq!(
            output_for(r#"(print "score:\t" $2)"#, "Alice 20\n"),
            "score:\t 20\n"
        );
    }

    #[test]
    fn fmt_concatenates_without_separators() {
        assert_eq!(
            output_for(r#"(print (fmt $1 ":" $2))"#, "Alice 20\nBob 30\n"),
            "Alice:20\nBob:30\n"
        );
    }

    #[test]
    fn field_zero_preserves_the_whole_line() {
        assert_eq!(
            output_for("(print $0)", "  Alice   20  \nBob\t30\n"),
            "  Alice   20  \nBob\t30\n"
        );
    }

    #[test]
    fn a_missing_field_is_an_empty_string() {
        assert_eq!(output_for("(print $3)", "Alice 20\n"), "\n");
    }
}
