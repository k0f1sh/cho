use std::io;

use regex::Regex;

use crate::ast::{Form, Predicate, Program, Value};
use crate::parser::parse;

pub(super) struct CompiledProgram {
    pub(super) program: Program,
    pub(super) regexes: Vec<Regex>,
}

pub(super) fn compile_program(source: &str) -> io::Result<CompiledProgram> {
    let mut program = parse(source)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid program"))?;
    if program
        .forms
        .iter()
        .all(|form| matches!(form, Form::Filter(_)))
    {
        program.forms.push(Form::Print(vec![Value::Field(0)]));
    }
    for form in &program.forms {
        match form {
            Form::Print(values) => values.iter().try_for_each(validate_value)?,
            Form::Filter(condition) => validate_value(condition)?,
        }
    }
    let regexes = program
        .regex_patterns
        .iter()
        .map(|pattern| Regex::new(pattern))
        .collect::<Result<_, _>>()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    Ok(CompiledProgram { program, regexes })
}

fn validate_value(value: &Value) -> io::Result<()> {
    match value {
        Value::Concat(values) => values.iter().try_for_each(validate_value),
        Value::Join { separator, values } => {
            validate_value(separator)?;
            values.iter().try_for_each(validate_value)
        }
        Value::Replace {
            from, to, value, ..
        } => {
            validate_value(value)?;
            validate_value(from)?;
            validate_value(to)
        }
        Value::RegexReplace {
            replacement, value, ..
        } => {
            validate_value(value)?;
            validate_value(replacement)
        }
        Value::Part {
            delimiter,
            position,
            value,
        } => {
            validate_value(value)?;
            validate_value(delimiter)?;
            validate_value(position)
        }
        Value::Slice {
            start,
            length,
            value,
        } => {
            validate_value(value)?;
            validate_value(start)?;
            if let Some(length) = length {
                validate_value(length)?;
            }
            Ok(())
        }
        Value::Count(value)
        | Value::Escape(value)
        | Value::Quote { value, .. }
        | Value::Lower(value)
        | Value::Upper(value)
        | Value::Trim { value, .. }
        | Value::IpVersion(value)
        | Value::CidrPart { value, .. }
        | Value::SemVerPart { value, .. }
        | Value::DateTimeFromUnix(value)
        | Value::DurationSeconds(value)
        | Value::DurationMilliseconds(value)
        | Value::DurationMinutes(value)
        | Value::DurationHours(value)
        | Value::DurationDays(value)
        | Value::DurationToMilliseconds(value)
        | Value::DurationToSeconds(value)
        | Value::DurationToMinutes(value)
        | Value::DurationToHours(value)
        | Value::DurationToDays(value) => validate_value(value),
        Value::FloorDateTime {
            timezone, value, ..
        } => {
            validate_value(value)?;
            if let Some(timezone) = timezone {
                validate_value(timezone)?;
            }
            Ok(())
        }
        Value::FormatDateTime {
            format,
            timezone,
            value,
        } => {
            validate_value(value)?;
            validate_value(format)?;
            if let Some(timezone) = timezone {
                validate_value(timezone)?;
            }
            Ok(())
        }
        Value::AddDateTime { datetime, duration }
        | Value::SubtractDateTime { datetime, duration } => {
            validate_value(datetime)?;
            validate_value(duration)
        }
        Value::DifferenceDateTime { left, right } => {
            validate_value(left)?;
            validate_value(right)
        }
        Value::Arithmetic { left, right, .. } => {
            validate_value(left)?;
            validate_value(right)
        }
        Value::NumberOperation { value, .. } => validate_value(value),
        Value::FormatNumberFixed { digits, value } => {
            validate_value(value)?;
            validate_value(digits)
        }
        Value::UrlPart { value, .. } => validate_value(value),
        Value::UrlEncoding { value, .. } => validate_value(value),
        Value::UrlQueryGet { name, url } => {
            validate_value(url)?;
            validate_value(name)
        }
        Value::Predicate(predicate) => validate_predicate(predicate),
        Value::Not(value) => validate_value(value),
        Value::And(values) | Value::Or(values) => values.iter().try_for_each(validate_value),
        Value::If {
            condition,
            then_value,
            else_value,
        } => {
            validate_value(condition)?;
            validate_value(then_value)?;
            validate_value(else_value)
        }
        Value::Default { value, fallback } => {
            validate_value(value)?;
            validate_value(fallback)
        }
        Value::Field(_)
        | Value::FieldRange { .. }
        | Value::RecordNumber
        | Value::FieldCount
        | Value::String(_)
        | Value::Number(_)
        | Value::Boolean(_)
        | Value::DateTimeNow => Ok(()),
    }
}

fn validate_predicate(predicate: &Predicate) -> io::Result<()> {
    match predicate {
        Predicate::Compare { left, right, .. } => {
            validate_value(left)?;
            validate_value(right)
        }
        Predicate::Regex { target, .. } => validate_value(target),
        Predicate::StringTest { value, pattern, .. } => {
            validate_value(value)?;
            validate_value(pattern)
        }
        Predicate::IpClass { value, .. } => validate_value(value),
        Predicate::CidrContains { cidr, ip } => {
            validate_value(cidr)?;
            validate_value(ip)
        }
        Predicate::UrlQueryHas { name, url } => {
            validate_value(url)?;
            validate_value(name)
        }
    }
}
