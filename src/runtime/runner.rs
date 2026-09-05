use std::cell::RefCell;
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
    let ulid_generator = RefCell::new(ulid::Generator::new());
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
        ulid_generator: &ulid_generator,
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
    let mut physical_line = 1;
    let now = current_datetime();
    let ulid_generator = RefCell::new(ulid::Generator::new());

    while read_csv_record(&mut input, &mut raw, number + 1, &mut physical_line)? {
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
            ulid_generator: &ulid_generator,
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
    let ulid_generator = RefCell::new(ulid::Generator::new());

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
            ulid_generator: &ulid_generator,
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

fn read_csv_record<R: BufRead>(
    input: &mut R,
    record: &mut Vec<u8>,
    record_number: usize,
    physical_line: &mut usize,
) -> io::Result<bool> {
    record.clear();
    let start_line = *physical_line;
    let mut validator = CsvRecordValidator::new();
    loop {
        let bytes_read = input.read_until(b'\n', record)?;
        if bytes_read == 0 {
            if record.is_empty() {
                return Ok(false);
            }
            return match validator.validate(record) {
                CsvRecordStatus::Complete => Ok(true),
                CsvRecordStatus::OpenQuote { field } => Err(csv_syntax_error(
                    record_number,
                    start_line + record.iter().filter(|byte| **byte == b'\n').count(),
                    field,
                    "quoted field is not closed before end of input",
                )),
                CsvRecordStatus::Invalid {
                    offset,
                    field,
                    message,
                } => Err(csv_syntax_error(
                    record_number,
                    start_line
                        + record[..offset]
                            .iter()
                            .filter(|byte| **byte == b'\n')
                            .count(),
                    field,
                    message,
                )),
            };
        }
        *physical_line += 1;
        match validator.validate(record) {
            CsvRecordStatus::OpenQuote { .. } => continue,
            CsvRecordStatus::Invalid {
                offset,
                field,
                message,
            } => {
                return Err(csv_syntax_error(
                    record_number,
                    start_line
                        + record[..offset]
                            .iter()
                            .filter(|byte| **byte == b'\n')
                            .count(),
                    field,
                    message,
                ));
            }
            CsvRecordStatus::Complete => {
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CsvFieldState {
    Start,
    Unquoted,
    Quoted,
    AfterQuote,
}

enum CsvRecordStatus {
    Complete,
    OpenQuote {
        field: usize,
    },
    Invalid {
        offset: usize,
        field: usize,
        message: &'static str,
    },
}

struct CsvRecordValidator {
    state: CsvFieldState,
    field: usize,
    scanned: usize,
}

impl CsvRecordValidator {
    fn new() -> Self {
        Self {
            state: CsvFieldState::Start,
            field: 1,
            scanned: 0,
        }
    }

    fn validate(&mut self, record: &[u8]) -> CsvRecordStatus {
        while self.scanned < record.len() {
            let byte = record[self.scanned];
            let record_end = byte == b'\n' && self.state != CsvFieldState::Quoted;
            if record_end {
                return CsvRecordStatus::Complete;
            }
            if byte == b'\r'
                && self.state != CsvFieldState::Quoted
                && record.get(self.scanned + 1) != Some(&b'\n')
            {
                return CsvRecordStatus::Invalid {
                    offset: self.scanned,
                    field: self.field,
                    message: "bare carriage return is not allowed outside a quoted field",
                };
            }
            match self.state {
                CsvFieldState::Start => match byte {
                    b',' => self.field += 1,
                    b'"' => self.state = CsvFieldState::Quoted,
                    _ => self.state = CsvFieldState::Unquoted,
                },
                CsvFieldState::Unquoted => match byte {
                    b',' => {
                        self.field += 1;
                        self.state = CsvFieldState::Start;
                    }
                    b'"' => {
                        return CsvRecordStatus::Invalid {
                            offset: self.scanned,
                            field: self.field,
                            message: "quote is only allowed at the start of a field",
                        };
                    }
                    _ => {}
                },
                CsvFieldState::Quoted => {
                    if byte == b'"' {
                        self.state = CsvFieldState::AfterQuote;
                    }
                }
                CsvFieldState::AfterQuote => match byte {
                    b'"' => self.state = CsvFieldState::Quoted,
                    b',' => {
                        self.field += 1;
                        self.state = CsvFieldState::Start;
                    }
                    b'\r' if record.get(self.scanned + 1) == Some(&b'\n') => {}
                    _ => {
                        return CsvRecordStatus::Invalid {
                            offset: self.scanned,
                            field: self.field,
                            message: "expected a comma or end of record after closing quote",
                        };
                    }
                },
            }
            self.scanned += 1;
        }
        if self.state == CsvFieldState::Quoted {
            CsvRecordStatus::OpenQuote { field: self.field }
        } else {
            CsvRecordStatus::Complete
        }
    }
}

fn csv_syntax_error(record: usize, line: usize, field: usize, message: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("CSV record {record}, line {line}, field {field}: {message}"),
    )
}

fn parse_csv_fields(record: &[u8]) -> io::Result<Vec<String>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(record);
    let mut records = reader.records();
    let fields = records
        .next()
        .transpose()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?
        .unwrap_or_default();
    if records
        .next()
        .transpose()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?
        .is_some()
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "CSV decoder unexpectedly produced multiple records",
        ));
    }
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

#[cfg(test)]
mod tests {
    use super::{CsvRecordStatus, CsvRecordValidator};

    #[test]
    fn csv_validator_resumes_at_the_first_unscanned_byte() {
        let mut validator = CsvRecordValidator::new();
        let mut record = br#""first line
"#
        .to_vec();

        assert!(matches!(
            validator.validate(&record),
            CsvRecordStatus::OpenQuote { field: 1 }
        ));
        assert_eq!(validator.scanned, record.len());

        let previously_scanned = validator.scanned;
        record.extend_from_slice(b"second line\n");
        assert!(matches!(
            validator.validate(&record),
            CsvRecordStatus::OpenQuote { field: 1 }
        ));
        assert_eq!(validator.scanned, record.len());
        assert!(validator.scanned > previously_scanned);

        record.extend_from_slice(b"\",last\n");
        assert!(matches!(
            validator.validate(&record),
            CsvRecordStatus::Complete
        ));
    }
}
