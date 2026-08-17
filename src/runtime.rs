use std::io::{self, BufRead, Write};

use regex::Regex;

use crate::ast::{ComparisonOperator, Expr, Predicate, Value};
use crate::parser::parse;

struct Record<'line, 'separator> {
    line: &'line str,
    number: usize,
    field_separator: Option<&'separator Regex>,
    csv_fields: Option<&'line [String]>,
}

enum PreparedPredicate<'a> {
    Compare {
        operator: &'a ComparisonOperator,
        left: &'a Value,
        right: &'a Value,
    },
    Regex {
        target: &'a Value,
        regex: Regex,
    },
    Not(Box<PreparedPredicate<'a>>),
    And(Vec<PreparedPredicate<'a>>),
    Or(Vec<PreparedPredicate<'a>>),
}

enum PreparedExpr<'a> {
    Print(&'a [Value]),
    Filter(PreparedPredicate<'a>),
}

impl Record<'_, '_> {
    fn field(&self, number: usize) -> Option<&str> {
        if let Some(fields) = self.csv_fields {
            return fields.get(number - 1).map(String::as_str);
        }
        match self.field_separator {
            Some(separator) => separator.split(self.line).nth(number - 1),
            None => self.line.split_whitespace().nth(number - 1),
        }
    }

    fn field_count(&self) -> usize {
        if let Some(fields) = self.csv_fields {
            return fields.len();
        }
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
        Value::Join { separator, values } => values
            .iter()
            .map(|value| evaluate(value, record))
            .collect::<Vec<_>>()
            .join(&evaluate(separator, record)),
        Value::Count(value) => evaluate(value, record).chars().count().to_string(),
        Value::Escape(value) => escape(&evaluate(value, record)),
        Value::If {
            predicate,
            then_value,
            else_value,
        } => {
            if matches_unprepared(predicate, record) {
                evaluate(then_value, record)
            } else {
                evaluate(else_value, record)
            }
        }
        Value::Lower(value) => evaluate(value, record).to_lowercase(),
        Value::Upper(value) => evaluate(value, record).to_uppercase(),
        Value::Default { value, fallback } => {
            let value = evaluate(value, record);
            if value.is_empty() {
                evaluate(fallback, record)
            } else {
                value
            }
        }
    }
}

fn matches_unprepared(predicate: &Predicate, record: &Record<'_, '_>) -> bool {
    match predicate {
        Predicate::Compare {
            operator,
            left,
            right,
        } => compare(operator, &evaluate(left, record), &evaluate(right, record)),
        Predicate::Regex { target, pattern } => Regex::new(pattern)
            .expect("regular expressions are prepared before execution")
            .is_match(&evaluate(target, record)),
        Predicate::Not(predicate) => !matches_unprepared(predicate, record),
        Predicate::And(predicates) => predicates
            .iter()
            .all(|predicate| matches_unprepared(predicate, record)),
        Predicate::Or(predicates) => predicates
            .iter()
            .any(|predicate| matches_unprepared(predicate, record)),
    }
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn matches(predicate: &PreparedPredicate<'_>, record: &Record<'_, '_>) -> bool {
    match predicate {
        PreparedPredicate::Compare {
            operator,
            left,
            right,
        } => compare(operator, &evaluate(left, record), &evaluate(right, record)),
        PreparedPredicate::Regex { target, regex } => regex.is_match(&evaluate(target, record)),
        PreparedPredicate::Not(predicate) => !matches(predicate, record),
        PreparedPredicate::And(predicates) => predicates
            .iter()
            .all(|predicate| matches(predicate, record)),
        PreparedPredicate::Or(predicates) => predicates
            .iter()
            .any(|predicate| matches(predicate, record)),
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

pub fn run_csv<R: BufRead, W: Write>(program: &str, input: R, mut output: W) -> io::Result<()> {
    let program = parse(program)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid program"))?;
    let expressions = prepare_expressions(&program.expressions)?;
    let mut input = input;
    let mut raw = Vec::new();
    let mut number = 0;

    while read_csv_record(&mut input, &mut raw)? {
        number += 1;
        let line = std::str::from_utf8(&raw)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let fields = parse_csv_fields(&raw)?;
        let record = Record {
            line,
            number,
            field_separator: None,
            csv_fields: Some(&fields),
        };
        execute(&expressions, &record, &mut output)?;
    }
    Ok(())
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
    let expressions = prepare_expressions(&program.expressions)?;

    for (index, line) in input.lines().enumerate() {
        let line = line?;
        let record = Record {
            line: &line,
            number: index + 1,
            field_separator: field_separator.as_ref(),
            csv_fields: None,
        };
        execute(&expressions, &record, &mut output)?;
    }
    Ok(())
}

fn execute<W: Write>(
    expressions: &[PreparedExpr<'_>],
    record: &Record<'_, '_>,
    output: &mut W,
) -> io::Result<()> {
    for expression in expressions {
        match expression {
            PreparedExpr::Print(values) => {
                let rendered = values
                    .iter()
                    .map(|value| evaluate(value, record))
                    .collect::<Vec<_>>()
                    .join(" ");
                writeln!(output, "{rendered}")?;
            }
            PreparedExpr::Filter(predicate) if !matches(predicate, record) => break,
            PreparedExpr::Filter(_) => {}
        }
    }
    Ok(())
}

fn read_csv_record<R: BufRead>(input: &mut R, record: &mut Vec<u8>) -> io::Result<bool> {
    record.clear();
    loop {
        let bytes_read = input.read_until(b'\n', record)?;
        if bytes_read == 0 {
            return Ok(!record.is_empty());
        }
        if !csv_record_has_open_quote(record) {
            if record.last() == Some(&b'\n') {
                record.pop();
                if record.last() == Some(&b'\r') {
                    record.pop();
                }
            }
            return Ok(true);
        }
    }
}

fn csv_record_has_open_quote(record: &[u8]) -> bool {
    let mut quoted = false;
    let mut index = 0;
    while index < record.len() {
        if record[index] == b'"' {
            if quoted && record.get(index + 1) == Some(&b'"') {
                index += 2;
                continue;
            }
            quoted = !quoted;
        }
        index += 1;
    }
    quoted
}

fn parse_csv_fields(record: &[u8]) -> io::Result<Vec<String>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(record);
    let fields = reader
        .records()
        .next()
        .transpose()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?
        .unwrap_or_default();
    Ok(fields.iter().map(str::to_owned).collect())
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

fn prepare_expressions(expressions: &[Expr]) -> io::Result<Vec<PreparedExpr<'_>>> {
    expressions
        .iter()
        .map(|expression| match expression {
            Expr::Print(values) => {
                for value in values {
                    validate_value(value)?;
                }
                Ok(PreparedExpr::Print(values))
            }
            Expr::Filter(predicate) => prepare_predicate(predicate).map(PreparedExpr::Filter),
        })
        .collect()
}

fn validate_value(value: &Value) -> io::Result<()> {
    match value {
        Value::Concat(values) => values.iter().try_for_each(validate_value),
        Value::Join { separator, values } => {
            validate_value(separator)?;
            values.iter().try_for_each(validate_value)
        }
        Value::Count(value) | Value::Escape(value) | Value::Lower(value) | Value::Upper(value) => {
            validate_value(value)
        }
        Value::If {
            predicate,
            then_value,
            else_value,
        } => {
            validate_predicate(predicate)?;
            validate_value(then_value)?;
            validate_value(else_value)
        }
        Value::Default { value, fallback } => {
            validate_value(value)?;
            validate_value(fallback)
        }
        Value::Field(_)
        | Value::RecordNumber
        | Value::FieldCount
        | Value::String(_)
        | Value::Number(_) => Ok(()),
    }
}

fn validate_predicate(predicate: &Predicate) -> io::Result<()> {
    match predicate {
        Predicate::Compare { left, right, .. } => {
            validate_value(left)?;
            validate_value(right)
        }
        Predicate::Regex { target, pattern } => {
            validate_value(target)?;
            Regex::new(pattern)
                .map(|_| ())
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))
        }
        Predicate::Not(predicate) => validate_predicate(predicate),
        Predicate::And(predicates) | Predicate::Or(predicates) => {
            predicates.iter().try_for_each(validate_predicate)
        }
    }
}

fn prepare_predicate(predicate: &Predicate) -> io::Result<PreparedPredicate<'_>> {
    validate_predicate(predicate)?;
    match predicate {
        Predicate::Compare {
            operator,
            left,
            right,
        } => Ok(PreparedPredicate::Compare {
            operator,
            left,
            right,
        }),
        Predicate::Regex { target, pattern } => Ok(PreparedPredicate::Regex {
            target,
            regex: Regex::new(pattern)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?,
        }),
        Predicate::Not(predicate) => Ok(PreparedPredicate::Not(Box::new(prepare_predicate(
            predicate,
        )?))),
        Predicate::And(predicates) => Ok(PreparedPredicate::And(
            predicates
                .iter()
                .map(prepare_predicate)
                .collect::<io::Result<_>>()?,
        )),
        Predicate::Or(predicates) => Ok(PreparedPredicate::Or(
            predicates
                .iter()
                .map(prepare_predicate)
                .collect::<io::Result<_>>()?,
        )),
    }
}
