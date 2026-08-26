use std::io::{self, BufRead, Write};
use std::time::SystemTime;

use chrono::{DateTime, Timelike, Utc};
use regex::Regex;

use crate::ast::Form;

use super::compile::compile_program;
use super::eval::{EvalContext, Record, evaluate};
use super::value::{EvalResult, expect_boolean};

pub fn run<R: BufRead, W: Write>(program: &str, input: R, mut output: W) -> io::Result<()> {
    run_with_field_separator(program, None, input, &mut output)
}

pub fn run_no_input<W: Write>(program: &str, mut output: W) -> io::Result<()> {
    let program = compile_program(program)?;
    let record = Record {
        line: "",
        number: 1,
        field_spans: Vec::new(),
        csv_fields: None,
        now: current_datetime(),
    };
    let context = EvalContext {
        record: &record,
        regexes: &program.regexes,
    };
    execute(&program.program.forms, &context, &mut output)
}

pub fn run_csv<R: BufRead, W: Write>(program: &str, input: R, mut output: W) -> io::Result<()> {
    let program = compile_program(program)?;
    if program.program.contains_field_range {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "field ranges are not supported with --csv",
        ));
    }
    let mut input = input;
    let mut raw = Vec::new();
    let mut number = 0;
    let now = current_datetime();

    while read_csv_record(&mut input, &mut raw)? {
        number += 1;
        let line = std::str::from_utf8(&raw)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let fields = parse_csv_fields(&raw)?;
        let record = Record {
            line,
            number,
            field_spans: Vec::new(),
            csv_fields: Some(&fields),
            now,
        };
        let context = EvalContext {
            record: &record,
            regexes: &program.regexes,
        };
        execute(&program.program.forms, &context, &mut output)?;
    }
    Ok(())
}

pub fn run_with_field_separator<R: BufRead, W: Write>(
    program: &str,
    field_separator: Option<&str>,
    input: R,
    mut output: W,
) -> io::Result<()> {
    let program = compile_program(program)?;
    let field_separator = compile_field_separator(field_separator)?;
    let now = current_datetime();

    for (index, line) in input.lines().enumerate() {
        let line = line?;
        let field_spans = split_field_spans(&line, field_separator.as_ref());
        let record = Record {
            line: &line,
            number: index + 1,
            field_spans,
            csv_fields: None,
            now,
        };
        let context = EvalContext {
            record: &record,
            regexes: &program.regexes,
        };
        execute(&program.program.forms, &context, &mut output)?;
    }
    Ok(())
}

fn split_field_spans(line: &str, separator: Option<&Regex>) -> Vec<(usize, usize)> {
    if line.is_empty() {
        return Vec::new();
    }
    if let Some(separator) = separator {
        let mut spans = Vec::new();
        let mut start = 0;
        for delimiter in separator.find_iter(line) {
            spans.push((start, delimiter.start()));
            start = delimiter.end();
        }
        spans.push((start, line.len()));
        return spans;
    }

    let mut spans = Vec::new();
    let mut start = None;
    for (index, character) in line.char_indices() {
        if character.is_whitespace() {
            if let Some(start) = start.take() {
                spans.push((start, index));
            }
        } else if start.is_none() {
            start = Some(index);
        }
    }
    if let Some(start) = start {
        spans.push((start, line.len()));
    }
    spans
}

fn current_datetime() -> DateTime<Utc> {
    DateTime::<Utc>::from(SystemTime::now())
        .with_nanosecond(0)
        .expect("zero nanoseconds is always valid")
}

fn execute<W: Write>(
    forms: &[Form],
    record: &EvalContext<'_, '_, '_>,
    output: &mut W,
) -> io::Result<()> {
    for form in forms {
        let result = match form {
            Form::Print(values) => {
                let rendered = values
                    .iter()
                    .map(|value| evaluate(value, record).map(|value| value.render()))
                    .collect::<EvalResult<Vec<_>>>()
                    .map(|values| values.join(" "));
                match rendered {
                    Ok(rendered) => {
                        writeln!(output, "{rendered}")?;
                        Ok(true)
                    }
                    Err(error) => Err(error),
                }
            }
            Form::Filter(condition) => {
                evaluate(condition, record).and_then(|value| expect_boolean(value, "filter", 1))
            }
        };
        match result {
            Ok(true) => {}
            Ok(false) => break,
            Err(error) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("record {}: {error}", record.number),
                ));
            }
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
