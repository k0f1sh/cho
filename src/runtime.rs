use std::io::{self, BufRead, Write};

use regex::Regex;

use crate::ast::{ComparisonOperator, Expr, Predicate, Value};
use crate::parser::parse;

struct Record<'line, 'separator> {
    line: &'line str,
    number: usize,
    field_separator: Option<&'separator Regex>,
}

impl Record<'_, '_> {
    fn field(&self, number: usize) -> Option<&str> {
        match self.field_separator {
            Some(separator) => separator.split(self.line).nth(number - 1),
            None => self.line.split_whitespace().nth(number - 1),
        }
    }

    fn field_count(&self) -> usize {
        if self.line.is_empty() {
            return 0;
        }
        match self.field_separator {
            Some(separator) => separator.split(self.line).count(),
            None => self.line.split_whitespace().count(),
        }
    }
}

fn evaluate(value: &Value, record: &Record<'_, '_>) -> String {
    match value {
        Value::Field(0) => record.line.to_owned(),
        Value::Field(number) => record.field(*number).unwrap_or("").to_owned(),
        Value::RecordNumber => record.number.to_string(),
        Value::FieldCount => record.field_count().to_string(),
        Value::String(value) => value.clone(),
        Value::Number(number) => number.to_string(),
        Value::Concat(values) => values.iter().map(|value| evaluate(value, record)).collect(),
        Value::Count(value) => evaluate(value, record).chars().count().to_string(),
    }
}

fn matches(predicate: &Predicate, regex: Option<&Regex>, record: &Record<'_, '_>) -> bool {
    match predicate {
        Predicate::Compare {
            operator,
            left,
            right,
        } => compare(operator, &evaluate(left, record), &evaluate(right, record)),
        Predicate::Regex { target, .. } => regex
            .expect("a regex predicate must have a compiled regex")
            .is_match(&evaluate(target, record)),
    }
}

fn compare(operator: &ComparisonOperator, left: &str, right: &str) -> bool {
    let numbers = || Some((left.parse::<f64>().ok()?, right.parse::<f64>().ok()?));
    match operator {
        ComparisonOperator::GreaterThan => numbers().is_some_and(|(a, b)| a > b),
        ComparisonOperator::GreaterThanOrEqual => numbers().is_some_and(|(a, b)| a >= b),
        ComparisonOperator::LessThan => numbers().is_some_and(|(a, b)| a < b),
        ComparisonOperator::LessThanOrEqual => numbers().is_some_and(|(a, b)| a <= b),
        ComparisonOperator::Equal | ComparisonOperator::NotEqual => {
            let equal = numbers().map_or_else(|| left == right, |(a, b)| a == b);
            equal == matches!(operator, ComparisonOperator::Equal)
        }
    }
}

pub fn run<R: BufRead, W: Write>(program: &str, input: R, mut output: W) -> io::Result<()> {
    run_with_field_separator(program, None, input, &mut output)
}

pub fn run_with_field_separator<R: BufRead, W: Write>(
    program: &str,
    field_separator: Option<&str>,
    input: R,
    mut output: W,
) -> io::Result<()> {
    let program = parse(program)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid program"))?;
    let field_separator = compile_field_separator(field_separator)?;
    let regexes = compile_predicate_regexes(&program.expressions)?;

    for (index, line) in input.lines().enumerate() {
        let line = line?;
        let record = Record {
            line: &line,
            number: index + 1,
            field_separator: field_separator.as_ref(),
        };
        for (expression, regex) in program.expressions.iter().zip(&regexes) {
            match expression {
                Expr::Print(values) => {
                    let rendered = values
                        .iter()
                        .map(|value| evaluate(value, &record))
                        .collect::<Vec<_>>()
                        .join(" ");
                    writeln!(output, "{rendered}")?;
                }
                Expr::Filter(predicate) if !matches(predicate, regex.as_ref(), &record) => break,
                Expr::Filter(_) => {}
            }
        }
    }
    Ok(())
}

fn compile_field_separator(pattern: Option<&str>) -> io::Result<Option<Regex>> {
    let separator = pattern
        .map(Regex::new)
        .transpose()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    if separator
        .as_ref()
        .is_some_and(|separator| separator.is_match(""))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "field separator must not match an empty string",
        ));
    }
    Ok(separator)
}

fn compile_predicate_regexes(expressions: &[Expr]) -> io::Result<Vec<Option<Regex>>> {
    expressions
        .iter()
        .map(|expression| match expression {
            Expr::Filter(Predicate::Regex { pattern, .. }) => Regex::new(pattern).map(Some),
            _ => Ok(None),
        })
        .collect::<Result<_, _>>()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))
}
