use std::fmt;
use std::io::{self, BufRead, Write};
use std::net::IpAddr;
use std::time::SystemTime;

use chrono::format::{Item, StrftimeItems};
use chrono::{DateTime, SecondsFormat, TimeDelta, Timelike, Utc};
use ipnet::IpNet;
use regex::Regex;

use crate::ast::{ComparisonOperator, ComparisonType, DateTimeFloorUnit, Expr, Predicate, Value};
use crate::parser::parse;

struct Record<'line, 'separator> {
    line: &'line str,
    number: usize,
    field_separator: Option<&'separator Regex>,
    csv_fields: Option<&'line [String]>,
    now: DateTime<Utc>,
}

#[derive(Debug, Clone)]
enum RuntimeValue {
    String(String),
    Number(f64),
    DateTime(DateTime<Utc>),
    Duration(TimeDelta),
}

impl RuntimeValue {
    fn render(&self) -> String {
        match self {
            Self::String(value) => value.clone(),
            Self::Number(value) => value.to_string(),
            Self::DateTime(value) => value.to_rfc3339_opts(SecondsFormat::AutoSi, true),
            Self::Duration(value) => render_duration(value),
        }
    }

    fn type_name(&self) -> &'static str {
        match self {
            Self::String(_) => "String",
            Self::Number(_) => "Number",
            Self::DateTime(_) => "DateTime",
            Self::Duration(_) => "Duration",
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

enum PreparedPredicate<'a> {
    Compare {
        kind: &'a ComparisonType,
        operator: &'a ComparisonOperator,
        left: &'a Value,
        right: &'a Value,
    },
    Regex {
        target: &'a Value,
        regex: Regex,
    },
    IpPrivate(&'a Value),
    CidrContains {
        cidr: &'a Value,
        ip: &'a Value,
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

fn evaluate(value: &Value, record: &Record<'_, '_>) -> EvalResult<RuntimeValue> {
    match value {
        Value::Field(0) => Ok(RuntimeValue::String(record.line.to_owned())),
        Value::Field(number) => Ok(RuntimeValue::String(
            record.field(*number).unwrap_or("").to_owned(),
        )),
        Value::RecordNumber => Ok(RuntimeValue::Number(record.number as f64)),
        Value::FieldCount => Ok(RuntimeValue::Number(record.field_count() as f64)),
        Value::String(value) => Ok(RuntimeValue::String(value.clone())),
        Value::Number(number) => Ok(RuntimeValue::Number(*number)),
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
        Value::FormatDateTime { format, value } => {
            let format = expect_string(evaluate(format, record)?, "dt/fmt", 1)?;
            if StrftimeItems::new(&format).any(|item| item == Item::Error) {
                return Err(EvalError::conversion(
                    "dt/fmt",
                    1,
                    "String (valid strftime format)",
                    format,
                    "contains an invalid format specifier",
                ));
            }
            let datetime = expect_datetime(evaluate(value, record)?, "dt/fmt", 2)?;
            Ok(RuntimeValue::String(datetime.format(&format).to_string()))
        }
        Value::DurationSeconds(value) => duration_from_value(value, 1.0, "du/s", record),
        Value::DurationMinutes(value) => duration_from_value(value, 60.0, "du/m", record),
        Value::DurationHours(value) => duration_from_value(value, 3600.0, "du/h", record),
        Value::DateTimeNow => Ok(RuntimeValue::DateTime(record.now)),
        Value::FloorDateTime { unit, value } => {
            let function = floor_name(unit);
            let datetime = expect_datetime(evaluate(value, record)?, function, 1)?;
            Ok(RuntimeValue::DateTime(floor_datetime(datetime, unit)))
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
        Value::Part {
            delimiter,
            position,
            value,
        } => {
            let delimiter = evaluate(delimiter, record)?.render();
            if delimiter.is_empty() {
                return Err(EvalError::conversion(
                    "s/part",
                    1,
                    "a non-empty delimiter",
                    delimiter,
                    "is empty",
                ));
            }
            let position = expect_number(evaluate(position, record)?, "s/part", 2)?;
            if position.fract() != 0.0 || position < 1.0 {
                return Err(EvalError::conversion(
                    "s/part",
                    2,
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
                    2,
                    "Number (representable part position)",
                    position_input,
                    "is outside the supported position range",
                ));
            }
            let value = evaluate(value, record)?.render();
            value
                .split(&delimiter)
                .nth(position as usize - 1)
                .map(|part| RuntimeValue::String(part.to_owned()))
                .ok_or_else(|| {
                    EvalError::conversion(
                        "s/part",
                        2,
                        "an existing part position",
                        position.to_string(),
                        "is out of range after splitting argument 3",
                    )
                })
        }
        Value::Count(value) => Ok(RuntimeValue::Number(
            evaluate(value, record)?.render().chars().count() as f64,
        )),
        Value::Escape(value) => Ok(RuntimeValue::String(escape(
            &evaluate(value, record)?.render(),
        ))),
        Value::If {
            predicate,
            then_value,
            else_value,
        } => {
            if matches_unprepared(predicate, record)? {
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
        Value::Default { value, fallback } => match evaluate(value, record) {
            Ok(value) if !value.is_empty() => Ok(value),
            Ok(_) | Err(_) => evaluate(fallback, record),
        },
    }
}

fn duration_from_value(
    value: &Value,
    multiplier: f64,
    function: &'static str,
    record: &Record<'_, '_>,
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

fn matches_unprepared(predicate: &Predicate, record: &Record<'_, '_>) -> EvalResult<bool> {
    match predicate {
        Predicate::Compare {
            kind,
            operator,
            left,
            right,
        } => compare(kind, operator, left, right, record),
        Predicate::Regex { target, pattern } => Ok(Regex::new(pattern)
            .expect("regular expressions are prepared before execution")
            .is_match(&evaluate(target, record)?.render())),
        Predicate::IpPrivate(value) => {
            let ip = expect_ip(evaluate(value, record)?, "ip/private?", 1)?;
            Ok(matches!(ip, IpAddr::V4(ip) if is_private_ipv4(ip)))
        }
        Predicate::CidrContains { cidr, ip } => {
            let cidr = expect_cidr(evaluate(cidr, record)?, "cidr/contains?", 1)?;
            let ip = expect_ip(evaluate(ip, record)?, "cidr/contains?", 2)?;
            Ok(cidr.contains(&ip))
        }
        Predicate::Not(predicate) => Ok(!matches_unprepared(predicate, record)?),
        Predicate::And(predicates) => {
            for predicate in predicates {
                if !matches_unprepared(predicate, record)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        Predicate::Or(predicates) => {
            for predicate in predicates {
                if matches_unprepared(predicate, record)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
    }
}

fn compare(
    kind: &ComparisonType,
    operator: &ComparisonOperator,
    left: &Value,
    right: &Value,
    record: &Record<'_, '_>,
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
    }
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn is_private_ipv4(ip: std::net::Ipv4Addr) -> bool {
    let [first, second, ..] = ip.octets();
    first == 10 || (first == 172 && (16..=31).contains(&second)) || (first == 192 && second == 168)
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

fn floor_name(unit: &DateTimeFloorUnit) -> &'static str {
    match unit {
        DateTimeFloorUnit::Second => "dt/floor-s",
        DateTimeFloorUnit::Minute => "dt/floor-m",
        DateTimeFloorUnit::Hour => "dt/floor-h",
        DateTimeFloorUnit::Day => "dt/floor-d",
    }
}

fn matches(predicate: &PreparedPredicate<'_>, record: &Record<'_, '_>) -> EvalResult<bool> {
    match predicate {
        PreparedPredicate::Compare {
            kind,
            operator,
            left,
            right,
        } => compare(kind, operator, left, right, record),
        PreparedPredicate::Regex { target, regex } => {
            Ok(regex.is_match(&evaluate(target, record)?.render()))
        }
        PreparedPredicate::IpPrivate(value) => {
            let ip = expect_ip(evaluate(value, record)?, "ip/private?", 1)?;
            Ok(matches!(ip, IpAddr::V4(ip) if is_private_ipv4(ip)))
        }
        PreparedPredicate::CidrContains { cidr, ip } => {
            let cidr = expect_cidr(evaluate(cidr, record)?, "cidr/contains?", 1)?;
            let ip = expect_ip(evaluate(ip, record)?, "cidr/contains?", 2)?;
            Ok(cidr.contains(&ip))
        }
        PreparedPredicate::Not(predicate) => Ok(!matches(predicate, record)?),
        PreparedPredicate::And(predicates) => {
            for predicate in predicates {
                if !matches(predicate, record)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        PreparedPredicate::Or(predicates) => {
            for predicate in predicates {
                if matches(predicate, record)? {
                    return Ok(true);
                }
            }
            Ok(false)
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
    let now = current_datetime();

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
            now,
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
    let now = current_datetime();

    for (index, line) in input.lines().enumerate() {
        let line = line?;
        let record = Record {
            line: &line,
            number: index + 1,
            field_separator: field_separator.as_ref(),
            csv_fields: None,
            now,
        };
        execute(&expressions, &record, &mut output)?;
    }
    Ok(())
}

fn current_datetime() -> DateTime<Utc> {
    DateTime::<Utc>::from(SystemTime::now())
        .with_nanosecond(0)
        .expect("zero nanoseconds is always valid")
}

fn execute<W: Write>(
    expressions: &[PreparedExpr<'_>],
    record: &Record<'_, '_>,
    output: &mut W,
) -> io::Result<()> {
    for expression in expressions {
        let result = match expression {
            PreparedExpr::Print(values) => {
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
            PreparedExpr::Filter(predicate) => matches(predicate, record),
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
        Value::Part {
            delimiter,
            position,
            value,
        } => {
            validate_value(delimiter)?;
            validate_value(position)?;
            validate_value(value)
        }
        Value::Count(value)
        | Value::Escape(value)
        | Value::Lower(value)
        | Value::Upper(value)
        | Value::DateTimeFromUnix(value)
        | Value::FloorDateTime { value, .. }
        | Value::DurationSeconds(value)
        | Value::DurationMinutes(value)
        | Value::DurationHours(value) => validate_value(value),
        Value::FormatDateTime { format, value } => {
            validate_value(format)?;
            validate_value(value)
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
        | Value::Number(_)
        | Value::DateTimeNow => Ok(()),
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
        Predicate::IpPrivate(value) => validate_value(value),
        Predicate::CidrContains { cidr, ip } => {
            validate_value(cidr)?;
            validate_value(ip)
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
            kind,
            operator,
            left,
            right,
        } => Ok(PreparedPredicate::Compare {
            kind,
            operator,
            left,
            right,
        }),
        Predicate::Regex { target, pattern } => Ok(PreparedPredicate::Regex {
            target,
            regex: Regex::new(pattern)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?,
        }),
        Predicate::IpPrivate(value) => Ok(PreparedPredicate::IpPrivate(value)),
        Predicate::CidrContains { cidr, ip } => Ok(PreparedPredicate::CidrContains { cidr, ip }),
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
