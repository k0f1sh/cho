use std::fmt;
use std::io::{self, BufRead, Write};
use std::net::IpAddr;
use std::ops::Deref;
use std::time::SystemTime;

use chrono::format::{Item, StrftimeItems};
use chrono::{
    DateTime, FixedOffset, LocalResult, NaiveDateTime, SecondsFormat, TimeDelta, TimeZone,
    Timelike, Utc,
};
use chrono_tz::Tz;
use ipnet::IpNet;
use regex::Regex;
use semver::Version;

use crate::ast::{
    ArithmeticOperator, CidrPart, ComparisonOperator, ComparisonType, DateTimeFloorUnit, Form,
    IpClass, NumberOperator, Predicate, Program, ReplaceMode, SemVerPart, StringQuote, StringTest,
    StringTrim, UrlEncoding, UrlPart, Value,
};
use crate::parser::parse;

struct Record<'line> {
    line: &'line str,
    number: usize,
    field_spans: Vec<(usize, usize)>,
    csv_fields: Option<&'line [String]>,
    now: DateTime<Utc>,
}

struct EvalContext<'record, 'line, 'program> {
    record: &'record Record<'line>,
    regexes: &'program [Regex],
}

impl<'line> Deref for EvalContext<'_, 'line, '_> {
    type Target = Record<'line>;

    fn deref(&self) -> &Self::Target {
        self.record
    }
}

#[derive(Debug, Clone)]
enum RuntimeValue {
    String(String),
    Number(f64),
    Boolean(bool),
    DateTime(DateTime<Utc>),
    Duration(TimeDelta),
    IpAddr(IpAddr),
}

impl RuntimeValue {
    fn render(&self) -> String {
        match self {
            Self::String(value) => value.clone(),
            Self::Number(value) => value.to_string(),
            Self::Boolean(value) => value.to_string(),
            Self::DateTime(value) => value.to_rfc3339_opts(SecondsFormat::AutoSi, true),
            Self::Duration(value) => render_duration(value),
            Self::IpAddr(value) => value.to_string(),
        }
    }

    fn type_name(&self) -> &'static str {
        match self {
            Self::String(_) => "String",
            Self::Number(_) => "Number",
            Self::Boolean(_) => "Boolean",
            Self::DateTime(_) => "DateTime",
            Self::Duration(_) => "Duration",
            Self::IpAddr(_) => "IpAddr",
        }
    }

    fn is_empty(&self) -> bool {
        matches!(self, Self::String(value) if value.is_empty())
    }
}

#[derive(Debug)]
struct EvalError {
    function: &'static str,
    argument: usize,
    expected: &'static str,
    input: String,
    reason: String,
}

impl EvalError {
    fn conversion(
        function: &'static str,
        argument: usize,
        expected: &'static str,
        input: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            function,
            argument,
            expected,
            input: input.into(),
            reason: reason.into(),
        }
    }
}

impl fmt::Display for EvalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}: argument {} expects {}, but {:?} {}",
            self.function, self.argument, self.expected, self.input, self.reason
        )
    }
}

type EvalResult<T> = Result<T, EvalError>;

impl Record<'_> {
    fn field(&self, number: usize) -> Option<&str> {
        if let Some(fields) = self.csv_fields {
            return fields.get(number - 1).map(String::as_str);
        }
        self.field_spans
            .get(number - 1)
            .map(|(start, end)| &self.line[*start..*end])
    }

    fn field_count(&self) -> usize {
        if let Some(fields) = self.csv_fields {
            return fields.len();
        }
        self.field_spans.len()
    }

    fn field_range(&self, start: Option<usize>, end: Option<usize>) -> &str {
        if self.field_spans.is_empty() {
            return "";
        }
        let start = match start {
            Some(start) => {
                let Some((start, _)) = self.field_spans.get(start - 1) else {
                    return "";
                };
                *start
            }
            None => 0,
        };
        let end = match end {
            Some(end) => {
                let end = end.min(self.field_spans.len());
                let Some((_, end)) = self.field_spans.get(end - 1) else {
                    return "";
                };
                *end
            }
            None => self.line.len(),
        };
        &self.line[start..end]
    }
}

fn evaluate(value: &Value, record: &EvalContext<'_, '_, '_>) -> EvalResult<RuntimeValue> {
    match value {
        Value::Field(0) => Ok(RuntimeValue::String(record.line.to_owned())),
        Value::Field(number) => Ok(RuntimeValue::String(
            record.field(*number).unwrap_or("").to_owned(),
        )),
        Value::FieldRange { start, end } => Ok(RuntimeValue::String(
            record.field_range(*start, *end).to_owned(),
        )),
        Value::RecordNumber => Ok(RuntimeValue::Number(record.number as f64)),
        Value::FieldCount => Ok(RuntimeValue::Number(record.field_count() as f64)),
        Value::String(value) => Ok(RuntimeValue::String(value.clone())),
        Value::Number(number) => Ok(RuntimeValue::Number(*number)),
        Value::Boolean(value) => Ok(RuntimeValue::Boolean(*value)),
        Value::Arithmetic {
            operator,
            left,
            right,
        } => evaluate_arithmetic(operator, left, right, record),
        Value::NumberOperation { operator, value } => {
            evaluate_number_operation(operator, value, record)
        }
        Value::FormatNumberFixed { digits, value } => {
            let value = expect_number(evaluate(value, record)?, "n/fixed", 1)?;
            let digits = expect_number(evaluate(digits, record)?, "n/fixed", 2)?;
            if digits.fract() != 0.0 || !(0.0..=100.0).contains(&digits) {
                return Err(EvalError::conversion(
                    "n/fixed",
                    2,
                    "Number (whole digits from 0 to 100)",
                    digits.to_string(),
                    "is outside the supported precision range",
                ));
            }
            Ok(RuntimeValue::String(format!(
                "{value:.digits$}",
                digits = digits as usize
            )))
        }
        Value::UrlPart { part, value } => {
            let function = url_part_name(part);
            let input = expect_string(evaluate(value, record)?, function, 1)?;
            let url = url::Url::parse(&input).map_err(|_| {
                EvalError::conversion(
                    function,
                    1,
                    "Url (absolute URL)",
                    input,
                    "is not a valid absolute URL",
                )
            })?;
            let part = match part {
                UrlPart::Scheme => url.scheme().to_owned(),
                UrlPart::Host => url.host_str().unwrap_or("").to_owned(),
                UrlPart::Port => url.port().map(|port| port.to_string()).unwrap_or_default(),
                UrlPart::Path => url.path().to_owned(),
                UrlPart::Query => url.query().unwrap_or("").to_owned(),
                UrlPart::Fragment => url.fragment().unwrap_or("").to_owned(),
            };
            Ok(RuntimeValue::String(part))
        }
        Value::UrlEncoding { operation, value } => {
            let function = url_encoding_name(operation);
            let value = expect_string(evaluate(value, record)?, function, 1)?;
            match operation {
                UrlEncoding::Encode => Ok(RuntimeValue::String(encode_url_component(&value))),
                UrlEncoding::Decode => decode_url_component(&value)
                    .map(RuntimeValue::String)
                    .map_err(|reason| {
                        EvalError::conversion(function, 1, "String (URL component)", value, reason)
                    }),
            }
        }
        Value::UrlQueryGet { name, url } => {
            let input = expect_string(evaluate(url, record)?, "url/query-get", 1)?;
            let url = parse_absolute_url(&input, "url/query-get", 1)?;
            let name = expect_string(evaluate(name, record)?, "url/query-get", 2)?;
            Ok(RuntimeValue::String(
                url.query_pairs()
                    .find_map(|(key, value)| (key == name).then(|| value.into_owned()))
                    .unwrap_or_default(),
            ))
        }
        Value::IpVersion(value) => {
            let ip = expect_ip(evaluate(value, record)?, "ip/version", 1)?;
            Ok(RuntimeValue::Number(match ip {
                IpAddr::V4(_) => 4.0,
                IpAddr::V6(_) => 6.0,
            }))
        }
        Value::CidrPart { part, value } => {
            let function = cidr_part_name(part);
            let cidr = expect_cidr(evaluate(value, record)?, function, 1)?;
            match part {
                CidrPart::Network | CidrPart::First => Ok(RuntimeValue::IpAddr(cidr.network())),
                CidrPart::Prefix => Ok(RuntimeValue::Number(cidr.prefix_len() as f64)),
                CidrPart::Last => Ok(RuntimeValue::IpAddr(cidr.broadcast())),
                CidrPart::Size => {
                    let address_bits = if cidr.addr().is_ipv4() { 32 } else { 128 };
                    let host_bits = address_bits - cidr.prefix_len();
                    if host_bits >= 53 {
                        return Err(EvalError::conversion(
                            function,
                            1,
                            "Cidr whose size fits Number's safe integer range",
                            cidr.to_string(),
                            "contains more than 2^53 - 1 addresses",
                        ));
                    }
                    exact_u64_number(1_u64 << host_bits, function, 1, cidr.to_string())
                }
            }
        }
        Value::SemVerPart { part, value } => {
            let function = semver_part_name(part);
            let version = expect_semver(evaluate(value, record)?, function, 1)?;
            match part {
                SemVerPart::Major => {
                    exact_u64_number(version.major, function, 1, version.to_string())
                }
                SemVerPart::Minor => {
                    exact_u64_number(version.minor, function, 1, version.to_string())
                }
                SemVerPart::Patch => {
                    exact_u64_number(version.patch, function, 1, version.to_string())
                }
                SemVerPart::Prerelease => Ok(RuntimeValue::String(version.pre.to_string())),
            }
        }
        Value::Predicate(predicate) => matches(predicate, record).map(RuntimeValue::Boolean),
        Value::Not(value) => {
            let value = expect_boolean(evaluate(value, record)?, "not", 1)?;
            Ok(RuntimeValue::Boolean(!value))
        }
        Value::And(values) => {
            for (index, value) in values.iter().enumerate() {
                if !expect_boolean(evaluate(value, record)?, "and", index + 1)? {
                    return Ok(RuntimeValue::Boolean(false));
                }
            }
            Ok(RuntimeValue::Boolean(true))
        }
        Value::Or(values) => {
            for (index, value) in values.iter().enumerate() {
                if expect_boolean(evaluate(value, record)?, "or", index + 1)? {
                    return Ok(RuntimeValue::Boolean(true));
                }
            }
            Ok(RuntimeValue::Boolean(false))
        }
        Value::DateTimeFromUnix(value) => {
            let seconds = expect_number(evaluate(value, record)?, "dt/unix", 1)?;
            if seconds.fract() != 0.0 || seconds < i64::MIN as f64 || seconds > i64::MAX as f64 {
                return Err(EvalError::conversion(
                    "dt/unix",
                    1,
                    "Number (whole Unix seconds)",
                    seconds.to_string(),
                    "is not a representable whole Unix timestamp",
                ));
            }
            DateTime::from_timestamp(seconds as i64, 0)
                .map(RuntimeValue::DateTime)
                .ok_or_else(|| {
                    EvalError::conversion(
                        "dt/unix",
                        1,
                        "Number (whole Unix seconds)",
                        seconds.to_string(),
                        "is outside the supported DateTime range",
                    )
                })
        }
        Value::FormatDateTime {
            format,
            timezone,
            value,
        } => {
            let datetime = expect_datetime(evaluate(value, record)?, "dt/fmt", 1)?;
            let format = expect_string(evaluate(format, record)?, "dt/fmt", 2)?;
            if StrftimeItems::new(&format).any(|item| item == Item::Error) {
                return Err(EvalError::conversion(
                    "dt/fmt",
                    2,
                    "String (valid strftime format)",
                    format,
                    "contains an invalid format specifier",
                ));
            }
            let timezone = timezone
                .as_ref()
                .map(|timezone| expect_string(evaluate(timezone, record)?, "dt/fmt", 3))
                .transpose()?;
            let formatted = match timezone {
                Some(timezone) => format_datetime_in_timezone(datetime, &format, timezone)?,
                None => datetime.format(&format).to_string(),
            };
            Ok(RuntimeValue::String(formatted))
        }
        Value::DurationSeconds(value) => duration_from_value(value, 1.0, "du/s", record),
        Value::DurationMilliseconds(value) => duration_from_value(value, 0.001, "du/ms", record),
        Value::DurationMinutes(value) => duration_from_value(value, 60.0, "du/m", record),
        Value::DurationHours(value) => duration_from_value(value, 3600.0, "du/h", record),
        Value::DurationDays(value) => duration_from_value(value, 86_400.0, "du/d", record),
        Value::DurationToMilliseconds(value) => {
            duration_as_number(value, 0.001, "du/to-ms", record)
        }
        Value::DurationToSeconds(value) => duration_as_number(value, 1.0, "du/to-s", record),
        Value::DurationToMinutes(value) => duration_as_number(value, 60.0, "du/to-m", record),
        Value::DurationToHours(value) => duration_as_number(value, 3600.0, "du/to-h", record),
        Value::DurationToDays(value) => duration_as_number(value, 86_400.0, "du/to-d", record),
        Value::DateTimeNow => Ok(RuntimeValue::DateTime(record.now)),
        Value::FloorDateTime {
            unit,
            timezone,
            value,
        } => {
            let function = floor_name(unit);
            let datetime = expect_datetime(evaluate(value, record)?, function, 1)?;
            let timezone = timezone
                .as_ref()
                .map(|timezone| expect_string(evaluate(timezone, record)?, function, 2))
                .transpose()?;
            let floored = match timezone {
                Some(timezone) => floor_datetime_in_timezone(datetime, unit, timezone)?,
                None => floor_datetime(datetime, unit),
            };
            Ok(RuntimeValue::DateTime(floored))
        }
        Value::AddDateTime { datetime, duration } => {
            let datetime = expect_datetime(evaluate(datetime, record)?, "dt/add", 1)?;
            let duration = expect_duration(evaluate(duration, record)?, "dt/add", 2)?;
            datetime
                .checked_add_signed(duration)
                .map(RuntimeValue::DateTime)
                .ok_or_else(|| {
                    EvalError::conversion(
                        "dt/add",
                        2,
                        "Duration producing a representable DateTime",
                        render_duration(&duration),
                        "overflows the supported DateTime range",
                    )
                })
        }
        Value::SubtractDateTime { datetime, duration } => {
            let datetime = expect_datetime(evaluate(datetime, record)?, "dt/sub", 1)?;
            let duration = expect_duration(evaluate(duration, record)?, "dt/sub", 2)?;
            datetime
                .checked_sub_signed(duration)
                .map(RuntimeValue::DateTime)
                .ok_or_else(|| {
                    EvalError::conversion(
                        "dt/sub",
                        2,
                        "Duration producing a representable DateTime",
                        render_duration(&duration),
                        "overflows the supported DateTime range",
                    )
                })
        }
        Value::DifferenceDateTime { left, right } => {
            let left = expect_datetime(evaluate(left, record)?, "dt/diff", 1)?;
            let right = expect_datetime(evaluate(right, record)?, "dt/diff", 2)?;
            let duration = left.signed_duration_since(right);
            if duration.num_nanoseconds().is_none() {
                return Err(EvalError::conversion(
                    "dt/diff",
                    1,
                    "DateTime within the supported Duration range of argument 2",
                    left.to_rfc3339(),
                    "produces a Duration outside the supported nanosecond range",
                ));
            }
            Ok(RuntimeValue::Duration(duration))
        }
        Value::Concat(values) => Ok(RuntimeValue::String(
            values
                .iter()
                .map(|value| evaluate(value, record).map(|value| value.render()))
                .collect::<EvalResult<String>>()?,
        )),
        Value::Join { separator, values } => {
            let separator = evaluate(separator, record)?.render();
            let values = values
                .iter()
                .map(|value| evaluate(value, record).map(|value| value.render()))
                .collect::<EvalResult<Vec<_>>>()?;
            Ok(RuntimeValue::String(values.join(&separator)))
        }
        Value::Replace {
            mode,
            value,
            from,
            to,
        } => {
            let value = evaluate(value, record)?.render();
            let from = evaluate(from, record)?.render();
            let to = evaluate(to, record)?.render();
            let replaced = match mode {
                ReplaceMode::First => value.replacen(&from, &to, 1),
                ReplaceMode::All => value.replace(&from, &to),
            };
            Ok(RuntimeValue::String(replaced))
        }
        Value::RegexReplace {
            mode,
            regex,
            replacement,
            value,
        } => {
            let value = evaluate(value, record)?.render();
            let replacement = evaluate(replacement, record)?.render();
            let regex = &record.regexes[regex.0];
            let replaced = match mode {
                ReplaceMode::First => regex.replace(&value, replacement.as_str()),
                ReplaceMode::All => regex.replace_all(&value, replacement.as_str()),
            };
            Ok(RuntimeValue::String(replaced.into_owned()))
        }
        Value::Part {
            delimiter,
            position,
            value,
        } => {
            let value = evaluate(value, record)?.render();
            let delimiter = evaluate(delimiter, record)?.render();
            if delimiter.is_empty() {
                return Err(EvalError::conversion(
                    "s/part",
                    2,
                    "a non-empty delimiter",
                    delimiter,
                    "is empty",
                ));
            }
            let position = expect_number(evaluate(position, record)?, "s/part", 3)?;
            if position.fract() != 0.0 || position < 1.0 {
                return Err(EvalError::conversion(
                    "s/part",
                    3,
                    "Number (positive whole part position)",
                    position.to_string(),
                    "is not a positive whole number",
                ));
            }
            let position_input = position.to_string();
            let position = position as u128;
            if position > usize::MAX as u128 {
                return Err(EvalError::conversion(
                    "s/part",
                    3,
                    "Number (representable part position)",
                    position_input,
                    "is outside the supported position range",
                ));
            }
            Ok(RuntimeValue::String(
                value
                    .split(&delimiter)
                    .nth(position as usize - 1)
                    .unwrap_or("")
                    .to_owned(),
            ))
        }
        Value::Slice {
            start,
            length,
            value,
        } => evaluate_string_slice(start, length.as_deref(), value, record),
        Value::Count(value) => Ok(RuntimeValue::Number(
            evaluate(value, record)?.render().chars().count() as f64,
        )),
        Value::Escape(value) => Ok(RuntimeValue::String(escape(
            &evaluate(value, record)?.render(),
        ))),
        Value::Quote { kind, value } => Ok(RuntimeValue::String(quote(
            &evaluate(value, record)?.render(),
            kind,
        ))),
        Value::If {
            condition,
            then_value,
            else_value,
        } => {
            if expect_boolean(evaluate(condition, record)?, "if", 1)? {
                evaluate(then_value, record)
            } else {
                evaluate(else_value, record)
            }
        }
        Value::Lower(value) => Ok(RuntimeValue::String(
            evaluate(value, record)?.render().to_lowercase(),
        )),
        Value::Upper(value) => Ok(RuntimeValue::String(
            evaluate(value, record)?.render().to_uppercase(),
        )),
        Value::Trim { kind, value } => {
            let value = evaluate(value, record)?.render();
            let trimmed = match kind {
                StringTrim::Both => value.trim(),
                StringTrim::Left => value.trim_start(),
                StringTrim::Right => value.trim_end(),
            };
            Ok(RuntimeValue::String(trimmed.to_owned()))
        }
        Value::Default { value, fallback } => match evaluate(value, record) {
            Ok(value) if !value.is_empty() => Ok(value),
            Ok(_) | Err(_) => evaluate(fallback, record),
        },
    }
}

fn evaluate_arithmetic(
    operator: &ArithmeticOperator,
    left: &Value,
    right: &Value,
    record: &EvalContext<'_, '_, '_>,
) -> EvalResult<RuntimeValue> {
    let function = match operator {
        ArithmeticOperator::Add => "+",
        ArithmeticOperator::Subtract => "-",
        ArithmeticOperator::Multiply => "*",
        ArithmeticOperator::Divide => "/",
    };
    let left = expect_number(evaluate(left, record)?, function, 1)?;
    let right = expect_number(evaluate(right, record)?, function, 2)?;
    if matches!(operator, ArithmeticOperator::Divide) && right == 0.0 {
        return Err(EvalError::conversion(
            function,
            2,
            "a non-zero Number",
            right.to_string(),
            "is zero",
        ));
    }
    let result = match operator {
        ArithmeticOperator::Add => left + right,
        ArithmeticOperator::Subtract => left - right,
        ArithmeticOperator::Multiply => left * right,
        ArithmeticOperator::Divide => left / right,
    };
    if !result.is_finite() {
        return Err(EvalError::conversion(
            function,
            2,
            "Number producing a finite result with argument 1",
            right.to_string(),
            "produces a non-finite result",
        ));
    }
    Ok(RuntimeValue::Number(result))
}

fn evaluate_number_operation(
    operator: &NumberOperator,
    value: &Value,
    record: &EvalContext<'_, '_, '_>,
) -> EvalResult<RuntimeValue> {
    let function = match operator {
        NumberOperator::Truncate => "n/trunc",
        NumberOperator::Floor => "n/floor",
        NumberOperator::Ceil => "n/ceil",
        NumberOperator::Round => "n/round",
        NumberOperator::Absolute => "n/abs",
    };
    let number = expect_number(evaluate(value, record)?, function, 1)?;
    let result = match operator {
        NumberOperator::Truncate => number.trunc(),
        NumberOperator::Floor => number.floor(),
        NumberOperator::Ceil => number.ceil(),
        NumberOperator::Round => number.round(),
        NumberOperator::Absolute => number.abs(),
    };
    Ok(RuntimeValue::Number(if result == 0.0 {
        0.0
    } else {
        result
    }))
}

fn duration_from_value(
    value: &Value,
    multiplier: f64,
    function: &'static str,
    record: &EvalContext<'_, '_, '_>,
) -> EvalResult<RuntimeValue> {
    let number = expect_number(evaluate(value, record)?, function, 1)?;
    let nanoseconds = number * multiplier * 1_000_000_000.0;
    if !nanoseconds.is_finite() || nanoseconds < i64::MIN as f64 || nanoseconds > i64::MAX as f64 {
        return Err(EvalError::conversion(
            function,
            1,
            "Number producing a representable Duration",
            number.to_string(),
            "is outside the supported Duration range",
        ));
    }
    Ok(RuntimeValue::Duration(TimeDelta::nanoseconds(
        nanoseconds.round() as i64,
    )))
}

fn duration_as_number(
    value: &Value,
    divisor: f64,
    function: &'static str,
    record: &EvalContext<'_, '_, '_>,
) -> EvalResult<RuntimeValue> {
    let duration = expect_duration(evaluate(value, record)?, function, 1)?;
    let nanoseconds = duration
        .num_nanoseconds()
        .expect("Duration values are constrained to nanoseconds");
    Ok(RuntimeValue::Number(
        nanoseconds as f64 / 1_000_000_000.0 / divisor,
    ))
}

fn expect_number(value: RuntimeValue, function: &'static str, argument: usize) -> EvalResult<f64> {
    let input = value.render();
    let number = match value {
        RuntimeValue::Number(number) => number,
        RuntimeValue::String(value) => value.parse::<f64>().map_err(|_| {
            EvalError::conversion(
                function,
                argument,
                "Number",
                value,
                "cannot be parsed as a number",
            )
        })?,
        value => {
            return Err(EvalError::conversion(
                function,
                argument,
                "Number",
                input,
                format!("has type {}", value.type_name()),
            ));
        }
    };
    if !number.is_finite() {
        return Err(EvalError::conversion(
            function,
            argument,
            "finite Number",
            input,
            "is not finite",
        ));
    }
    Ok(number)
}

fn evaluate_string_slice(
    start: &Value,
    length: Option<&Value>,
    value: &Value,
    record: &EvalContext<'_, '_, '_>,
) -> EvalResult<RuntimeValue> {
    let value = evaluate(value, record)?.render();
    let start = expect_slice_index(start, "s/slice", 2, false, record)?;
    let length = length
        .map(|length| expect_slice_index(length, "s/slice", 3, true, record))
        .transpose()?;
    let Some((start, _)) = value.char_indices().nth(start - 1) else {
        return Ok(RuntimeValue::String(String::new()));
    };
    let end = length
        .and_then(|length| {
            value[start..]
                .char_indices()
                .nth(length)
                .map(|(end, _)| start + end)
        })
        .unwrap_or(value.len());
    Ok(RuntimeValue::String(value[start..end].to_owned()))
}

fn expect_slice_index(
    value: &Value,
    function: &'static str,
    argument: usize,
    allow_zero: bool,
    record: &EvalContext<'_, '_, '_>,
) -> EvalResult<usize> {
    let number = expect_number(evaluate(value, record)?, function, argument)?;
    let minimum = if allow_zero { 0.0 } else { 1.0 };
    if number.fract() != 0.0 || number < minimum {
        let expected = if allow_zero {
            "Number (non-negative whole slice length)"
        } else {
            "Number (positive whole slice start)"
        };
        return Err(EvalError::conversion(
            function,
            argument,
            expected,
            number.to_string(),
            "is outside the supported slice range",
        ));
    }
    Ok(number as usize)
}

fn expect_boolean(
    value: RuntimeValue,
    function: &'static str,
    argument: usize,
) -> EvalResult<bool> {
    match value {
        RuntimeValue::Boolean(value) => Ok(value),
        value => Err(EvalError::conversion(
            function,
            argument,
            "Boolean",
            value.render(),
            format!("has type {}", value.type_name()),
        )),
    }
}

fn exact_u64_number(
    value: u64,
    function: &'static str,
    argument: usize,
    input: String,
) -> EvalResult<RuntimeValue> {
    const MAX_SAFE_INTEGER: u64 = (1_u64 << 53) - 1;
    if value > MAX_SAFE_INTEGER {
        return Err(EvalError::conversion(
            function,
            argument,
            "value within Number's safe integer range",
            input,
            "produces an integer greater than 2^53 - 1",
        ));
    }
    Ok(RuntimeValue::Number(value as f64))
}

fn expect_string(
    value: RuntimeValue,
    function: &'static str,
    argument: usize,
) -> EvalResult<String> {
    match value {
        RuntimeValue::String(value) => Ok(value),
        value => Err(EvalError::conversion(
            function,
            argument,
            "String",
            value.render(),
            format!("has type {}", value.type_name()),
        )),
    }
}

fn expect_datetime(
    value: RuntimeValue,
    function: &'static str,
    argument: usize,
) -> EvalResult<DateTime<Utc>> {
    match value {
        RuntimeValue::DateTime(value) => Ok(value),
        RuntimeValue::String(value) => DateTime::parse_from_rfc3339(&value)
            .map(|value| value.with_timezone(&Utc))
            .map_err(|_| {
                EvalError::conversion(
                    function,
                    argument,
                    "DateTime",
                    value,
                    "is not valid RFC 3339",
                )
            }),
        value => Err(EvalError::conversion(
            function,
            argument,
            "DateTime",
            value.render(),
            format!("has type {}", value.type_name()),
        )),
    }
}

fn format_datetime_in_timezone(
    datetime: DateTime<Utc>,
    format: &str,
    timezone: String,
) -> EvalResult<String> {
    if let Ok(timezone) = timezone.parse::<Tz>() {
        return Ok(datetime.with_timezone(&timezone).format(format).to_string());
    }
    if let Some(offset) = parse_utc_offset(&timezone) {
        return Ok(datetime.with_timezone(&offset).format(format).to_string());
    }
    Err(EvalError::conversion(
        "dt/fmt",
        3,
        "String (IANA time zone or UTC offset ±HH:MM)",
        timezone,
        "is not a recognized time zone",
    ))
}

fn parse_utc_offset(value: &str) -> Option<FixedOffset> {
    let bytes = value.as_bytes();
    if bytes.len() != 6 || !matches!(bytes[0], b'+' | b'-') || bytes[3] != b':' {
        return None;
    }
    let digits = [bytes[1], bytes[2], bytes[4], bytes[5]];
    if !digits.iter().all(u8::is_ascii_digit) {
        return None;
    }
    let hours = i32::from(bytes[1] - b'0') * 10 + i32::from(bytes[2] - b'0');
    let minutes = i32::from(bytes[4] - b'0') * 10 + i32::from(bytes[5] - b'0');
    if minutes >= 60 {
        return None;
    }
    let seconds = hours.checked_mul(3_600)?.checked_add(minutes * 60)?;
    match bytes[0] {
        b'+' => FixedOffset::east_opt(seconds),
        b'-' => FixedOffset::west_opt(seconds),
        _ => None,
    }
}

fn expect_duration(
    value: RuntimeValue,
    function: &'static str,
    argument: usize,
) -> EvalResult<TimeDelta> {
    match value {
        RuntimeValue::Duration(value) => Ok(value),
        RuntimeValue::String(value) => {
            let number = value.parse::<f64>().map_err(|_| {
                EvalError::conversion(
                    function,
                    argument,
                    "Duration (seconds)",
                    value.clone(),
                    "cannot be parsed as seconds",
                )
            })?;
            let nanos = number * 1_000_000_000.0;
            if !nanos.is_finite() || nanos < i64::MIN as f64 || nanos > i64::MAX as f64 {
                return Err(EvalError::conversion(
                    function,
                    argument,
                    "Duration (seconds)",
                    value,
                    "is outside the supported Duration range",
                ));
            }
            Ok(TimeDelta::nanoseconds(nanos.round() as i64))
        }
        value => Err(EvalError::conversion(
            function,
            argument,
            "Duration",
            value.render(),
            format!("has type {}", value.type_name()),
        )),
    }
}

fn expect_ip(value: RuntimeValue, function: &'static str, argument: usize) -> EvalResult<IpAddr> {
    match value {
        RuntimeValue::IpAddr(value) => Ok(value),
        RuntimeValue::String(value) => value.parse().map_err(|_| {
            EvalError::conversion(
                function,
                argument,
                "IpAddr",
                value,
                "is not a valid IPv4 or IPv6 address",
            )
        }),
        value => Err(EvalError::conversion(
            function,
            argument,
            "IpAddr",
            value.render(),
            format!("has type {}", value.type_name()),
        )),
    }
}

fn cidr_part_name(part: &CidrPart) -> &'static str {
    match part {
        CidrPart::Network => "cidr/network",
        CidrPart::Prefix => "cidr/prefix",
        CidrPart::First => "cidr/first",
        CidrPart::Last => "cidr/last",
        CidrPart::Size => "cidr/size",
    }
}

fn semver_part_name(part: &SemVerPart) -> &'static str {
    match part {
        SemVerPart::Major => "semver/major",
        SemVerPart::Minor => "semver/minor",
        SemVerPart::Patch => "semver/patch",
        SemVerPart::Prerelease => "semver/prerelease",
    }
}

fn expect_cidr(value: RuntimeValue, function: &'static str, argument: usize) -> EvalResult<IpNet> {
    match value {
        RuntimeValue::String(value) => value.parse().map_err(|_| {
            EvalError::conversion(
                function,
                argument,
                "Cidr",
                value,
                "is not a valid IPv4 or IPv6 network",
            )
        }),
        value => Err(EvalError::conversion(
            function,
            argument,
            "Cidr",
            value.render(),
            format!("has type {}", value.type_name()),
        )),
    }
}

fn expect_semver(
    value: RuntimeValue,
    function: &'static str,
    argument: usize,
) -> EvalResult<Version> {
    match value {
        RuntimeValue::String(value) => value.parse().map_err(|_| {
            EvalError::conversion(
                function,
                argument,
                "SemVer",
                value,
                "is not a valid MAJOR.MINOR.PATCH semantic version",
            )
        }),
        value => Err(EvalError::conversion(
            function,
            argument,
            "SemVer",
            value.render(),
            format!("has type {}", value.type_name()),
        )),
    }
}

fn parse_absolute_url(
    input: &str,
    function: &'static str,
    argument: usize,
) -> EvalResult<url::Url> {
    url::Url::parse(input).map_err(|_| {
        EvalError::conversion(
            function,
            argument,
            "Url (absolute URL)",
            input,
            "is not a valid absolute URL",
        )
    })
}

fn matches(predicate: &Predicate, record: &EvalContext<'_, '_, '_>) -> EvalResult<bool> {
    match predicate {
        Predicate::Compare {
            kind,
            operator,
            left,
            right,
        } => compare(kind, operator, left, right, record),
        Predicate::Regex { target, regex } => Ok(record
            .regexes
            .get(regex.0)
            .expect("RegexId is assigned from this program's regex pool")
            .is_match(&evaluate(target, record)?.render())),
        Predicate::StringTest {
            kind,
            value,
            pattern,
        } => {
            let function = string_test_name(kind);
            let value = expect_string(evaluate(value, record)?, function, 1)?;
            let pattern = expect_string(evaluate(pattern, record)?, function, 2)?;
            Ok(match kind {
                StringTest::StartsWith => value.starts_with(&pattern),
                StringTest::EndsWith => value.ends_with(&pattern),
                StringTest::Contains => value.contains(&pattern),
            })
        }
        Predicate::IpClass { kind, value } => {
            let ip = expect_ip(evaluate(value, record)?, ip_class_name(kind), 1)?;
            Ok(matches_ip_class(ip, kind))
        }
        Predicate::CidrContains { cidr, ip } => {
            let cidr = expect_cidr(evaluate(cidr, record)?, "cidr/contains?", 1)?;
            let ip = expect_ip(evaluate(ip, record)?, "cidr/contains?", 2)?;
            Ok(cidr.contains(&ip))
        }
        Predicate::UrlQueryHas { name, url } => {
            let input = expect_string(evaluate(url, record)?, "url/query-has?", 1)?;
            let url = parse_absolute_url(&input, "url/query-has?", 1)?;
            let name = expect_string(evaluate(name, record)?, "url/query-has?", 2)?;
            Ok(url.query_pairs().any(|(key, _)| key == name))
        }
    }
}

fn string_test_name(kind: &StringTest) -> &'static str {
    match kind {
        StringTest::StartsWith => "s/starts-with?",
        StringTest::EndsWith => "s/ends-with?",
        StringTest::Contains => "s/contains?",
    }
}

fn compare(
    kind: &ComparisonType,
    operator: &ComparisonOperator,
    left: &Value,
    right: &Value,
    record: &EvalContext<'_, '_, '_>,
) -> EvalResult<bool> {
    let function = comparison_name(kind, operator);
    match kind {
        ComparisonType::Number => {
            let left = expect_number(evaluate(left, record)?, function, 1)?;
            let right = expect_number(evaluate(right, record)?, function, 2)?;
            Ok(apply_ordering(operator, left.partial_cmp(&right)))
        }
        ComparisonType::String => {
            let left = expect_string(evaluate(left, record)?, function, 1)?;
            let right = expect_string(evaluate(right, record)?, function, 2)?;
            Ok(apply_ordering(operator, Some(left.cmp(&right))))
        }
        ComparisonType::DateTime => {
            let left = expect_datetime(evaluate(left, record)?, function, 1)?;
            let right = expect_datetime(evaluate(right, record)?, function, 2)?;
            Ok(apply_ordering(operator, Some(left.cmp(&right))))
        }
        ComparisonType::IpAddr => {
            let left = expect_ip(evaluate(left, record)?, function, 1)?;
            let right = expect_ip(evaluate(right, record)?, function, 2)?;
            Ok(match operator {
                ComparisonOperator::Equal => left == right,
                ComparisonOperator::NotEqual => left != right,
                _ => unreachable!("the parser only accepts IP equality comparisons"),
            })
        }
        ComparisonType::SemVer => {
            let left = expect_semver(evaluate(left, record)?, function, 1)?;
            let right = expect_semver(evaluate(right, record)?, function, 2)?;
            Ok(apply_ordering(operator, Some(left.cmp_precedence(&right))))
        }
    }
}

fn apply_ordering(operator: &ComparisonOperator, ordering: Option<std::cmp::Ordering>) -> bool {
    use std::cmp::Ordering::{Equal, Greater, Less};
    match operator {
        ComparisonOperator::GreaterThan => ordering == Some(Greater),
        ComparisonOperator::GreaterThanOrEqual => matches!(ordering, Some(Greater | Equal)),
        ComparisonOperator::LessThan => ordering == Some(Less),
        ComparisonOperator::LessThanOrEqual => matches!(ordering, Some(Less | Equal)),
        ComparisonOperator::Equal => ordering == Some(Equal),
        ComparisonOperator::NotEqual => ordering != Some(Equal),
    }
}

fn comparison_name(kind: &ComparisonType, operator: &ComparisonOperator) -> &'static str {
    match (kind, operator) {
        (ComparisonType::Number, ComparisonOperator::GreaterThan) => ">",
        (ComparisonType::Number, ComparisonOperator::GreaterThanOrEqual) => ">=",
        (ComparisonType::Number, ComparisonOperator::LessThan) => "<",
        (ComparisonType::Number, ComparisonOperator::LessThanOrEqual) => "<=",
        (ComparisonType::Number, ComparisonOperator::Equal) => "=",
        (ComparisonType::Number, ComparisonOperator::NotEqual) => "!=",
        (ComparisonType::String, ComparisonOperator::GreaterThan) => "s/>",
        (ComparisonType::String, ComparisonOperator::GreaterThanOrEqual) => "s/>=",
        (ComparisonType::String, ComparisonOperator::LessThan) => "s/<",
        (ComparisonType::String, ComparisonOperator::LessThanOrEqual) => "s/<=",
        (ComparisonType::String, ComparisonOperator::Equal) => "s/=",
        (ComparisonType::String, ComparisonOperator::NotEqual) => "s/!=",
        (ComparisonType::DateTime, ComparisonOperator::GreaterThan) => "dt/>",
        (ComparisonType::DateTime, ComparisonOperator::GreaterThanOrEqual) => "dt/>=",
        (ComparisonType::DateTime, ComparisonOperator::LessThan) => "dt/<",
        (ComparisonType::DateTime, ComparisonOperator::LessThanOrEqual) => "dt/<=",
        (ComparisonType::DateTime, ComparisonOperator::Equal) => "dt/=",
        (ComparisonType::DateTime, ComparisonOperator::NotEqual) => "dt/!=",
        (ComparisonType::IpAddr, ComparisonOperator::Equal) => "ip/=",
        (ComparisonType::IpAddr, ComparisonOperator::NotEqual) => "ip/!=",
        (ComparisonType::IpAddr, _) => unreachable!(),
        (ComparisonType::SemVer, ComparisonOperator::GreaterThan) => "semver/>",
        (ComparisonType::SemVer, ComparisonOperator::GreaterThanOrEqual) => "semver/>=",
        (ComparisonType::SemVer, ComparisonOperator::LessThan) => "semver/<",
        (ComparisonType::SemVer, ComparisonOperator::LessThanOrEqual) => "semver/<=",
        (ComparisonType::SemVer, ComparisonOperator::Equal) => "semver/=",
        (ComparisonType::SemVer, ComparisonOperator::NotEqual) => "semver/!=",
    }
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn quote(value: &str, kind: &StringQuote) -> String {
    let delimiter = match kind {
        StringQuote::Double => '"',
        StringQuote::Single => '\'',
    };
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push(delimiter);
    for character in value.chars() {
        match character {
            '\\' => quoted.push_str("\\\\"),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            character if character == delimiter => {
                quoted.push('\\');
                quoted.push(character);
            }
            character => quoted.push(character),
        }
    }
    quoted.push(delimiter);
    quoted
}

fn is_private_ipv4(ip: std::net::Ipv4Addr) -> bool {
    let [first, second, ..] = ip.octets();
    first == 10 || (first == 172 && (16..=31).contains(&second)) || (first == 192 && second == 168)
}

fn matches_ip_class(ip: IpAddr, kind: &IpClass) -> bool {
    match kind {
        IpClass::Private => match ip {
            IpAddr::V4(ip) => is_private_ipv4(ip),
            IpAddr::V6(ip) => ip.segments()[0] & 0xfe00 == 0xfc00,
        },
        IpClass::Loopback => match ip {
            IpAddr::V4(ip) => ip.octets()[0] == 127,
            IpAddr::V6(ip) => ip == std::net::Ipv6Addr::LOCALHOST,
        },
        IpClass::LinkLocal => match ip {
            IpAddr::V4(ip) => matches!(ip.octets(), [169, 254, _, _]),
            IpAddr::V6(ip) => ip.segments()[0] & 0xffc0 == 0xfe80,
        },
        IpClass::Multicast => match ip {
            IpAddr::V4(ip) => (224..=239).contains(&ip.octets()[0]),
            IpAddr::V6(ip) => ip.octets()[0] == 0xff,
        },
    }
}

fn ip_class_name(kind: &IpClass) -> &'static str {
    match kind {
        IpClass::Private => "ip/private?",
        IpClass::Loopback => "ip/loopback?",
        IpClass::LinkLocal => "ip/link-local?",
        IpClass::Multicast => "ip/multicast?",
    }
}

fn url_part_name(part: &UrlPart) -> &'static str {
    match part {
        UrlPart::Scheme => "url/scheme",
        UrlPart::Host => "url/host",
        UrlPart::Port => "url/port",
        UrlPart::Path => "url/path",
        UrlPart::Query => "url/query",
        UrlPart::Fragment => "url/fragment",
    }
}

fn url_encoding_name(operation: &UrlEncoding) -> &'static str {
    match operation {
        UrlEncoding::Encode => "url/encode",
        UrlEncoding::Decode => "url/decode",
    }
}

fn encode_url_component(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push(char::from(HEX[usize::from(byte >> 4)]));
            encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
    }
    encoded
}

fn decode_url_component(value: &str) -> Result<String, String> {
    let input = value.as_bytes();
    let mut decoded = Vec::with_capacity(input.len());
    let mut index = 0;
    while index < input.len() {
        if input[index] != b'%' {
            decoded.push(input[index]);
            index += 1;
            continue;
        }
        let high = input
            .get(index + 1)
            .and_then(|byte| hex_value(*byte))
            .ok_or_else(|| format!("contains an invalid percent escape at byte {index}"))?;
        let low = input
            .get(index + 2)
            .and_then(|byte| hex_value(*byte))
            .ok_or_else(|| format!("contains an invalid percent escape at byte {index}"))?;
        decoded.push(high << 4 | low);
        index += 3;
    }
    String::from_utf8(decoded).map_err(|_| "decodes to bytes that are not valid UTF-8".to_owned())
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn render_duration(value: &TimeDelta) -> String {
    let nanoseconds = value
        .num_nanoseconds()
        .expect("Duration values are constrained to nanoseconds");
    let negative = nanoseconds < 0;
    let absolute = i128::from(nanoseconds).abs();
    let seconds = absolute / 1_000_000_000;
    let fraction = absolute % 1_000_000_000;
    let sign = if negative { "-" } else { "" };
    if fraction == 0 {
        format!("{sign}{seconds}")
    } else {
        let fraction = format!("{fraction:09}").trim_end_matches('0').to_owned();
        format!("{sign}{seconds}.{fraction}")
    }
}

fn floor_datetime(datetime: DateTime<Utc>, unit: &DateTimeFloorUnit) -> DateTime<Utc> {
    match unit {
        DateTimeFloorUnit::Second => datetime
            .with_nanosecond(0)
            .expect("zero nanoseconds is always valid"),
        DateTimeFloorUnit::Minute => datetime
            .with_second(0)
            .and_then(|value| value.with_nanosecond(0))
            .expect("zero seconds and nanoseconds are always valid"),
        DateTimeFloorUnit::Hour => datetime
            .with_minute(0)
            .and_then(|value| value.with_second(0))
            .and_then(|value| value.with_nanosecond(0))
            .expect("the start of an hour is always valid in UTC"),
        DateTimeFloorUnit::Day => datetime
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .expect("the start of a day is always valid")
            .and_utc(),
    }
}

fn floor_datetime_in_timezone(
    datetime: DateTime<Utc>,
    unit: &DateTimeFloorUnit,
    timezone: String,
) -> EvalResult<DateTime<Utc>> {
    if let Ok(timezone) = timezone.parse::<Tz>() {
        let local = datetime.with_timezone(&timezone);
        let boundary = floor_naive_datetime(local.naive_local(), unit);
        return Ok(resolve_local_floor(&timezone, boundary, datetime));
    }
    if let Some(offset) = parse_utc_offset(&timezone) {
        let local = datetime.with_timezone(&offset);
        let boundary = floor_naive_datetime(local.naive_local(), unit);
        return Ok(offset
            .from_local_datetime(&boundary)
            .single()
            .expect("a fixed offset has exactly one local mapping")
            .with_timezone(&Utc));
    }
    Err(EvalError::conversion(
        floor_name(unit),
        2,
        "String (IANA time zone or UTC offset ±HH:MM)",
        timezone,
        "is not a recognized time zone",
    ))
}

fn floor_naive_datetime(datetime: NaiveDateTime, unit: &DateTimeFloorUnit) -> NaiveDateTime {
    match unit {
        DateTimeFloorUnit::Second => datetime.with_nanosecond(0).expect("valid second boundary"),
        DateTimeFloorUnit::Minute => datetime
            .with_second(0)
            .and_then(|value| value.with_nanosecond(0))
            .expect("valid minute boundary"),
        DateTimeFloorUnit::Hour => datetime
            .with_minute(0)
            .and_then(|value| value.with_second(0))
            .and_then(|value| value.with_nanosecond(0))
            .expect("valid hour boundary"),
        DateTimeFloorUnit::Day => datetime
            .date()
            .and_hms_opt(0, 0, 0)
            .expect("valid day boundary"),
    }
}

fn resolve_local_floor<T: TimeZone>(
    timezone: &T,
    mut boundary: NaiveDateTime,
    original: DateTime<Utc>,
) -> DateTime<Utc> {
    loop {
        let candidate = match timezone.from_local_datetime(&boundary) {
            LocalResult::Single(candidate) => Some(candidate.with_timezone(&Utc)),
            LocalResult::Ambiguous(first, second) => [first, second]
                .into_iter()
                .map(|candidate| candidate.with_timezone(&Utc))
                .filter(|candidate| *candidate <= original)
                .max(),
            LocalResult::None => None,
        };
        if let Some(candidate) = candidate.filter(|candidate| *candidate <= original) {
            return candidate;
        }
        boundary = boundary
            .checked_add_signed(TimeDelta::seconds(1))
            .expect("a local boundary near a representable DateTime is representable");
    }
}

fn floor_name(unit: &DateTimeFloorUnit) -> &'static str {
    match unit {
        DateTimeFloorUnit::Second => "dt/floor-s",
        DateTimeFloorUnit::Minute => "dt/floor-m",
        DateTimeFloorUnit::Hour => "dt/floor-h",
        DateTimeFloorUnit::Day => "dt/floor-d",
    }
}

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

struct CompiledProgram {
    program: Program,
    regexes: Vec<Regex>,
}

fn compile_program(source: &str) -> io::Result<CompiledProgram> {
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
