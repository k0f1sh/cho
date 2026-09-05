use std::cell::RefCell;
use std::net::IpAddr;
use std::ops::Deref;

use chrono::format::{Item, StrftimeItems};
use chrono::{DateTime, Utc};
use regex::Regex;

use crate::ast::{CidrPart, ReplaceMode, StringBoundary, StringTrim, UrlEncoding, UrlPart, Value};

use super::date::{checked_result, expect_date, expect_days, part as date_part, part_name};
use super::datetime::{
    duration_as_number, duration_from_value, expect_datetime, expect_duration, floor_datetime,
    floor_datetime_in_timezone, floor_name, format_datetime_in_timezone, render_duration,
};
use super::identifier::{expect_ulid, expect_uuid, ulid_time, uuid_time};
use super::network::{cidr_part_name, expect_cidr, expect_ip};
use super::number;
use super::predicate::matches;
use super::semver;
use super::string::{
    escape, evaluate_string_padding, evaluate_string_repeat, evaluate_string_slice,
    expect_part_position, quote, shell_quote, unquote,
};
use super::url::{
    decode_url_component, encode_url_component, parse_absolute_url, url_encoding_name,
    url_part_name,
};
use super::value::{
    EvalError, EvalResult, RuntimeValue, exact_u64_number, expect_boolean, expect_number,
    expect_string,
};

pub(super) struct Record<'line> {
    pub(super) line: &'line str,
    pub(super) number: usize,
    pub(super) field_spans: Vec<(usize, usize)>,
    pub(super) csv_fields: Option<&'line [String]>,
    pub(super) now: DateTime<Utc>,
}

pub(super) struct EvalContext<'record, 'line, 'program> {
    pub(super) record: &'record Record<'line>,
    pub(super) regexes: &'program [Regex],
    pub(super) ulid_generator: &'program RefCell<ulid::Generator>,
}

impl<'line> Deref for EvalContext<'_, 'line, '_> {
    type Target = Record<'line>;

    fn deref(&self) -> &Self::Target {
        self.record
    }
}

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

pub(super) fn evaluate(
    value: &Value,
    record: &EvalContext<'_, '_, '_>,
) -> EvalResult<RuntimeValue> {
    stacker::maybe_grow(64 * 1024, 1024 * 1024, || evaluate_inner(value, record))
}

fn evaluate_inner(value: &Value, record: &EvalContext<'_, '_, '_>) -> EvalResult<RuntimeValue> {
    match value {
        Value::Field(0) => Ok(RuntimeValue::String(record.line.to_owned())),
        Value::Field(number) => Ok(RuntimeValue::String(
            record.field(*number).unwrap_or("").to_owned(),
        )),
        Value::DynamicField(number) => {
            let input = evaluate(number, record)?;
            let rendered = input.render();
            let number = expect_number(input, "field", 1)?;
            if number < 0.0 || number.fract() != 0.0 || number > usize::MAX as f64 {
                return Err(EvalError::conversion(
                    "field",
                    1,
                    "Number (non-negative whole field number)",
                    rendered,
                    "is not a non-negative whole number",
                ));
            }
            let number = number as usize;
            Ok(RuntimeValue::String(if number == 0 {
                record.line.to_owned()
            } else {
                record.field(number).unwrap_or("").to_owned()
            }))
        }
        Value::FieldRange { start, end } => Ok(RuntimeValue::String(
            record.field_range(*start, *end).to_owned(),
        )),
        Value::DynamicFieldRange { start, end } => {
            let function = match (start, end) {
                (Some(_), Some(_)) => "fields",
                (Some(_), None) => "fields-from",
                (None, Some(_)) => "fields-to",
                (None, None) => unreachable!("a dynamic field range has at least one bound"),
            };
            let start = start
                .as_deref()
                .map(|value| evaluate_field_bound(value, record, function, 1))
                .transpose()?;
            let end_argument = usize::from(start.is_some()) + 1;
            let end = end
                .as_deref()
                .map(|value| evaluate_field_bound(value, record, function, end_argument))
                .transpose()?;
            if let (Some(start), Some(end)) = (start, end)
                && start > end
            {
                return Err(EvalError::conversion(
                    function,
                    2,
                    "field number greater than or equal to START",
                    end.to_string(),
                    "is before START",
                ));
            }
            Ok(RuntimeValue::String(
                record.field_range(start, end).to_owned(),
            ))
        }
        Value::RecordNumber => Ok(RuntimeValue::Number(record.number as f64)),
        Value::FieldCount => Ok(RuntimeValue::Number(record.field_count() as f64)),
        Value::String(value) => Ok(RuntimeValue::String(value.clone())),
        Value::Number(number) => Ok(RuntimeValue::Number(*number)),
        Value::Boolean(value) => Ok(RuntimeValue::Boolean(*value)),
        Value::StringEmpty(value) => Ok(RuntimeValue::Boolean(
            evaluate(value, record)?.render().is_empty(),
        )),
        Value::Arithmetic {
            operator,
            left,
            right,
        } => number::evaluate_arithmetic(operator, left, right, record),
        Value::NumberOperation { operator, value } => {
            number::evaluate_operation(operator, value, record)
        }
        Value::FormatNumberFixed { digits, value } => number::format_fixed(value, digits, record),
        Value::NumberMinimum(values) => number::evaluate_extreme(values, "n/min", f64::min, record),
        Value::NumberMaximum(values) => number::evaluate_extreme(values, "n/max", f64::max, record),
        Value::ClampNumber {
            value,
            minimum,
            maximum,
        } => number::clamp(value, minimum, maximum, record),
        Value::NormalizeByteSize(value) => {
            let value = super::byte_size::expect(evaluate(value, record)?, "bs", 1)?;
            Ok(RuntimeValue::ByteSize(value))
        }
        Value::ByteSizeToBytes(value) => {
            let value = super::byte_size::expect(evaluate(value, record)?, "bs/to-b", 1)?;
            super::byte_size::to_number(value)
        }
        Value::UrlPart { part, value } => {
            let function = url_part_name(part);
            let input = expect_string(evaluate(value, record)?, function, 1)?;
            let url = parse_absolute_url(&input, function, 1)?;
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
        Value::PathPart { part, value } => {
            let function = super::path::function_name(part);
            let value = expect_string(evaluate(value, record)?, function, 1)?;
            Ok(RuntimeValue::String(super::path::part(&value, *part)))
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
        Value::SemVerPart { part, value } => semver::evaluate_part(part, value, record),
        Value::NormalizeUuid(value) => {
            expect_uuid(evaluate(value, record)?, "uuid", 1).map(RuntimeValue::Uuid)
        }
        Value::UuidV4 => Ok(RuntimeValue::Uuid(uuid::Uuid::new_v4())),
        Value::UuidV7 => Ok(RuntimeValue::Uuid(uuid::Uuid::now_v7())),
        Value::UuidVersion(value) => {
            let uuid = expect_uuid(evaluate(value, record)?, "uuid/version", 1)?;
            Ok(RuntimeValue::Number(uuid.get_version_num() as f64))
        }
        Value::UuidTime(value) => {
            let uuid = expect_uuid(evaluate(value, record)?, "uuid/time", 1)?;
            uuid_time(uuid)
        }
        Value::NormalizeUlid(value) => {
            expect_ulid(evaluate(value, record)?, "ulid", 1).map(RuntimeValue::Ulid)
        }
        Value::UlidNew => {
            let mut generator = record.ulid_generator.borrow_mut();
            let ulid = generator
                .generate()
                .unwrap_or_else(|overflow| overflow.commit_overflow_increment());
            Ok(RuntimeValue::Ulid(ulid))
        }
        Value::UlidTime(value) => {
            let ulid = expect_ulid(evaluate(value, record)?, "ulid/time", 1)?;
            ulid_time(ulid)
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
        Value::NormalizeDate(value) => {
            expect_date(evaluate(value, record)?, "date", 1).map(RuntimeValue::Date)
        }
        Value::DatePart { part, value } => {
            let function = part_name(part);
            let date = expect_date(evaluate(value, record)?, function, 1)?;
            Ok(date_part(date, part))
        }
        Value::AddDate { date, days } => {
            let date = expect_date(evaluate(date, record)?, "d/add", 1)?;
            let days = expect_days(evaluate(days, record)?, "d/add")?;
            checked_result(date.checked_add_signed(days), "d/add", &days)
        }
        Value::SubtractDate { date, days } => {
            let date = expect_date(evaluate(date, record)?, "d/sub", 1)?;
            let days = expect_days(evaluate(days, record)?, "d/sub")?;
            checked_result(date.checked_sub_signed(days), "d/sub", &days)
        }
        Value::DifferenceDate { left, right } => {
            let left = expect_date(evaluate(left, record)?, "d/diff", 1)?;
            let right = expect_date(evaluate(right, record)?, "d/diff", 2)?;
            Ok(RuntimeValue::Number(
                left.signed_duration_since(right).num_days() as f64,
            ))
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
        Value::DateTimeToUnix(value) => {
            let datetime = expect_datetime(evaluate(value, record)?, "dt/to-unix", 1)?;
            Ok(RuntimeValue::Number(
                datetime.timestamp() as f64
                    + f64::from(datetime.timestamp_subsec_nanos()) / 1_000_000_000.0,
            ))
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
        Value::CsvJoin(values) => {
            let values = values
                .iter()
                .map(|value| evaluate(value, record).map(|value| value.render()))
                .collect::<EvalResult<Vec<_>>>()?;
            Ok(RuntimeValue::String(super::csv::join(&values)))
        }
        Value::Repeat { value, count } => evaluate_string_repeat(value, count, record),
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
        Value::RegexPart {
            regex,
            position,
            value,
        } => {
            let value = evaluate(value, record)?.render();
            let position = expect_part_position(position, "re/part", record)?;
            Ok(RuntimeValue::String(
                record.regexes[regex.0]
                    .split(&value)
                    .nth(position - 1)
                    .unwrap_or("")
                    .to_owned(),
            ))
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
            let position = expect_part_position(position, "s/part", record)?;
            Ok(RuntimeValue::String(
                value
                    .split(&delimiter)
                    .nth(position - 1)
                    .unwrap_or("")
                    .to_owned(),
            ))
        }
        Value::Boundary {
            kind,
            delimiter,
            value,
        } => {
            let value = evaluate(value, record)?.render();
            let delimiter = evaluate(delimiter, record)?.render();
            let function = match kind {
                StringBoundary::Before => "s/before",
                StringBoundary::After => "s/after",
            };
            if delimiter.is_empty() {
                return Err(EvalError::conversion(
                    function,
                    2,
                    "a non-empty delimiter",
                    delimiter,
                    "is empty",
                ));
            }
            let result = match (kind, value.split_once(&delimiter)) {
                (StringBoundary::Before, Some((before, _))) => before,
                (StringBoundary::After, Some((_, after))) => after,
                (StringBoundary::Before, None) => &value,
                (StringBoundary::After, None) => "",
            };
            Ok(RuntimeValue::String(result.to_owned()))
        }
        Value::Slice {
            start,
            length,
            value,
        } => evaluate_string_slice(start, length.as_deref(), value, record),
        Value::Pad {
            kind,
            value,
            width,
            fill,
        } => evaluate_string_padding(kind, value, width, fill.as_deref(), record),
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
        Value::Unquote(value) => unquote(&evaluate(value, record)?.render()),
        Value::ShellQuote(value) => Ok(RuntimeValue::String(shell_quote(
            &evaluate(value, record)?.render(),
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
        Value::Reverse(value) => Ok(RuntimeValue::String(
            evaluate(value, record)?.render().chars().rev().collect(),
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
        Value::TrimAffixes {
            value,
            prefix,
            suffix,
        } => {
            let value = evaluate(value, record)?.render();
            let prefix = prefix
                .as_deref()
                .map(|prefix| evaluate(prefix, record).map(|value| value.render()))
                .transpose()?;
            let suffix = suffix
                .as_deref()
                .map(|suffix| evaluate(suffix, record).map(|value| value.render()))
                .transpose()?;
            let without_prefix = prefix
                .as_deref()
                .and_then(|prefix| value.strip_prefix(prefix))
                .unwrap_or(&value);
            let without_suffix = suffix
                .as_deref()
                .and_then(|suffix| without_prefix.strip_suffix(suffix))
                .unwrap_or(without_prefix);
            Ok(RuntimeValue::String(without_suffix.to_owned()))
        }
        Value::Default { value, fallback } => match evaluate(value, record) {
            Ok(value) if !value.is_empty() => Ok(value),
            Ok(_) | Err(_) => evaluate(fallback, record),
        },
    }
}

fn evaluate_field_bound(
    value: &Value,
    record: &EvalContext<'_, '_, '_>,
    function: &'static str,
    argument: usize,
) -> EvalResult<usize> {
    let input = evaluate(value, record)?;
    let rendered = input.render();
    let number = expect_number(input, function, argument)?;
    if number < 1.0 || number.fract() != 0.0 || number > usize::MAX as f64 {
        return Err(EvalError::conversion(
            function,
            argument,
            "Number (positive whole field number)",
            rendered,
            "is not a positive whole number",
        ));
    }
    Ok(number as usize)
}
