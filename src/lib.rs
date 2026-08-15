use std::io::{self, BufRead, Write};

#[derive(Debug, PartialEq)]
pub enum ParseError {
    InvalidSyntax,
    InvalidField,
    UnterminatedString,
}

#[derive(Debug, PartialEq)]
pub struct Program {
    pub expressions: Vec<Expr>,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Print(Vec<Value>),
    Filter(Predicate),
}

#[derive(Debug, PartialEq)]
pub enum Predicate {
    Compare {
        operator: ComparisonOperator,
        left: Value,
        right: Value,
    },
}

#[derive(Debug, PartialEq)]
pub enum ComparisonOperator {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Field(usize),
    RecordNumber,
    FieldCount,
    String(String),
    Number(f64),
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

    fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut expressions = Vec::new();
        while self.position < self.tokens.len() {
            expressions.push(self.parse_expression()?);
        }

        Ok(Program { expressions })
    }

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        if self.next() != Some(Token::LeftParen) {
            return Err(ParseError::InvalidSyntax);
        }

        match self.next() {
            Some(Token::Atom(operator)) if operator == "print" => {
                Ok(Expr::Print(self.parse_values_until_right_paren()?))
            }
            Some(Token::Atom(operator)) if operator == "filter" => {
                let predicate = self.parse_predicate()?;
                if self.next() != Some(Token::RightParen) {
                    return Err(ParseError::InvalidSyntax);
                }
                Ok(Expr::Filter(predicate))
            }
            _ => Err(ParseError::InvalidSyntax),
        }
    }

    fn parse_predicate(&mut self) -> Result<Predicate, ParseError> {
        if self.next() != Some(Token::LeftParen) {
            return Err(ParseError::InvalidSyntax);
        }

        let operator = match self.next() {
            Some(Token::Atom(operator)) if operator == ">" => ComparisonOperator::GreaterThan,
            Some(Token::Atom(operator)) if operator == ">=" => {
                ComparisonOperator::GreaterThanOrEqual
            }
            Some(Token::Atom(operator)) if operator == "<" => ComparisonOperator::LessThan,
            Some(Token::Atom(operator)) if operator == "<=" => ComparisonOperator::LessThanOrEqual,
            Some(Token::Atom(operator)) if operator == "=" => ComparisonOperator::Equal,
            Some(Token::Atom(operator)) if operator == "!=" => ComparisonOperator::NotEqual,
            _ => return Err(ParseError::InvalidSyntax),
        };

        let left = self.parse_value()?;
        let right = self.parse_value()?;
        if self.next() != Some(Token::RightParen) {
            return Err(ParseError::InvalidSyntax);
        }

        Ok(Predicate::Compare {
            operator,
            left,
            right,
        })
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
                if field == "NR" {
                    return Ok(Value::RecordNumber);
                }
                if field == "NF" {
                    return Ok(Value::FieldCount);
                }
                if let Ok(number) = field.parse::<f64>() {
                    return Ok(Value::Number(number));
                }

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

pub fn parse(program: &str) -> Result<Program, ParseError> {
    Parser::new(tokenize(program)?).parse_program()
}

struct Record<'a> {
    line: &'a str,
    number: usize,
}

fn evaluate(value: &Value, record: &Record<'_>) -> String {
    match value {
        Value::Field(0) => record.line.to_owned(),
        Value::Field(number) => record
            .line
            .split_whitespace()
            .nth(number - 1)
            .unwrap_or("")
            .to_owned(),
        Value::RecordNumber => record.number.to_string(),
        Value::FieldCount => record.line.split_whitespace().count().to_string(),
        Value::String(value) => value.clone(),
        Value::Number(number) => number.to_string(),
        Value::Format(values) => values.iter().map(|value| evaluate(value, record)).collect(),
    }
}

fn matches(predicate: &Predicate, record: &Record<'_>) -> bool {
    match predicate {
        Predicate::Compare {
            operator,
            left,
            right,
        } => {
            let left = evaluate(left, record);
            let right = evaluate(right, record);
            let numbers = || Some((left.parse::<f64>().ok()?, right.parse::<f64>().ok()?));

            match operator {
                ComparisonOperator::GreaterThan => numbers().is_some_and(|(a, b)| a > b),
                ComparisonOperator::GreaterThanOrEqual => numbers().is_some_and(|(a, b)| a >= b),
                ComparisonOperator::LessThan => numbers().is_some_and(|(a, b)| a < b),
                ComparisonOperator::LessThanOrEqual => numbers().is_some_and(|(a, b)| a <= b),
                ComparisonOperator::Equal | ComparisonOperator::NotEqual => {
                    let equal = match numbers() {
                        Some((a, b)) => a == b,
                        None => left == right,
                    };
                    equal == matches!(operator, ComparisonOperator::Equal)
                }
            }
        }
    }
}

pub fn run<R: BufRead, W: Write>(program: &str, input: R, mut output: W) -> io::Result<()> {
    let program = parse(program).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid program (expected a print expression)",
        )
    })?;

    for (index, line) in input.lines().enumerate() {
        let line = line?;
        let record = Record {
            line: &line,
            number: index + 1,
        };
        for expression in &program.expressions {
            match expression {
                Expr::Print(values) => {
                    let rendered = values
                        .iter()
                        .map(|value| evaluate(value, &record))
                        .collect::<Vec<_>>()
                        .join(" ");
                    writeln!(output, "{rendered}")?;
                }
                Expr::Filter(predicate) if !matches(predicate, &record) => break,
                Expr::Filter(_) => {}
            }
        }
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
            Ok(Program {
                expressions: vec![Expr::Print(vec![Value::Field(1), Value::Field(2)])]
            })
        );
    }

    #[test]
    fn parses_record_number_and_field_count() {
        assert_eq!(
            parse("(print NR NF)"),
            Ok(Program {
                expressions: vec![Expr::Print(vec![Value::RecordNumber, Value::FieldCount,])]
            })
        );
    }

    #[test]
    fn parses_strings_and_nested_formats() {
        assert_eq!(
            parse(r#"(print (fmt $1 ":" $2) "points")"#),
            Ok(Program {
                expressions: vec![Expr::Print(vec![
                    Value::Format(vec![
                        Value::Field(1),
                        Value::String(":".to_owned()),
                        Value::Field(2),
                    ]),
                    Value::String("points".to_owned()),
                ])]
            })
        );
    }

    #[test]
    fn parses_multiple_top_level_expressions() {
        assert_eq!(
            parse("(print $1) (print $2)"),
            Ok(Program {
                expressions: vec![
                    Expr::Print(vec![Value::Field(1)]),
                    Expr::Print(vec![Value::Field(2)]),
                ]
            })
        );
    }

    #[test]
    fn parses_a_greater_than_filter() {
        assert_eq!(
            parse("(filter (> $2 20)) (print $0)"),
            Ok(Program {
                expressions: vec![
                    Expr::Filter(Predicate::Compare {
                        operator: ComparisonOperator::GreaterThan,
                        left: Value::Field(2),
                        right: Value::Number(20.0),
                    }),
                    Expr::Print(vec![Value::Field(0)]),
                ]
            })
        );
    }

    #[test]
    fn rejects_invalid_programs() {
        assert_eq!(parse("(print $x)"), Err(ParseError::InvalidField));
        assert_eq!(parse("print $1"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(unknown $1)"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(filter $1)"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(filter (> $1))"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(filter (> $1 2 3))"), Err(ParseError::InvalidSyntax));
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
    fn runs_top_level_expressions_in_order_for_each_line() {
        assert_eq!(
            output_for("(print $1) (print $2)", "Alice 20\nBob 30\n"),
            "Alice\n20\nBob\n30\n"
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

    #[test]
    fn nr_is_the_one_based_record_number() {
        assert_eq!(
            output_for("(print NR $1)", "Alice 20\nBob 30\n"),
            "1 Alice\n2 Bob\n"
        );
    }

    #[test]
    fn nf_is_the_number_of_fields() {
        assert_eq!(
            output_for("(print NF)", "Alice 20\nBob\t30 Osaka\n\n"),
            "2\n3\n0\n"
        );
    }

    #[test]
    fn nr_and_nf_can_be_used_inside_fmt() {
        assert_eq!(
            output_for(r#"(print (fmt NR ":" NF))"#, "Alice 20\nBob\n"),
            "1:2\n2:1\n"
        );
    }

    #[test]
    fn filter_skips_the_rest_of_a_non_matching_record() {
        assert_eq!(
            output_for(
                "(filter (> $2 20)) (print $1 $2)",
                "Alice 18\nBob 30\nCarol 25\n"
            ),
            "Bob 30\nCarol 25\n"
        );
    }

    #[test]
    fn expressions_before_a_filter_are_still_run() {
        assert_eq!(
            output_for(
                r#"(print "checking:" $1) (filter (> $2 20)) (print "passed:" $1)"#,
                "Alice 18\nBob 30\n"
            ),
            "checking: Alice\nchecking: Bob\npassed: Bob\n"
        );
    }

    #[test]
    fn multiple_filters_work_as_an_and_condition() {
        assert_eq!(
            output_for(
                "(filter (> $2 20)) (filter (> 40 $2)) (print $1)",
                "Alice 18\nBob 30\nCarol 45\n"
            ),
            "Bob\n"
        );
    }

    #[test]
    fn a_non_numeric_value_does_not_match() {
        assert_eq!(
            output_for("(filter (> $2 20)) (print $1)", "Alice unknown\n"),
            ""
        );
    }

    #[test]
    fn supports_all_numeric_comparison_operators() {
        let input = "low 10\nequal 20\nhigh 30\n";

        assert_eq!(
            output_for("(filter (>= $2 20)) (print $1)", input),
            "equal\nhigh\n"
        );
        assert_eq!(output_for("(filter (< $2 20)) (print $1)", input), "low\n");
        assert_eq!(
            output_for("(filter (<= $2 20)) (print $1)", input),
            "low\nequal\n"
        );
        assert_eq!(
            output_for("(filter (= $2 20)) (print $1)", input),
            "equal\n"
        );
        assert_eq!(
            output_for("(filter (!= $2 20)) (print $1)", input),
            "low\nhigh\n"
        );
    }

    #[test]
    fn equality_falls_back_to_string_comparison() {
        let input = "Alice 20\nBob 30\n";

        assert_eq!(
            output_for(r#"(filter (= $1 "Alice")) (print $2)"#, input),
            "20\n"
        );
        assert_eq!(
            output_for(r#"(filter (!= $1 "Alice")) (print $1)"#, input),
            "Bob\n"
        );
    }

    #[test]
    fn numeric_equality_compares_numeric_values() {
        assert_eq!(
            output_for("(filter (= $1 20)) (print $0)", "020\n20.0\n21\n"),
            "020\n20.0\n"
        );
    }
}
