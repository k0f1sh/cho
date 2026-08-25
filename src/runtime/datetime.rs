use super::*;

pub(super) fn duration_from_value(
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

pub(super) fn duration_as_number(
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

pub(super) fn expect_datetime(
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

pub(super) fn format_datetime_in_timezone(
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

pub(super) fn parse_utc_offset(value: &str) -> Option<FixedOffset> {
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

pub(super) fn expect_duration(
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

pub(super) fn render_duration(value: &TimeDelta) -> String {
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

pub(super) fn floor_datetime(datetime: DateTime<Utc>, unit: &DateTimeFloorUnit) -> DateTime<Utc> {
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

pub(super) fn floor_datetime_in_timezone(
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

pub(super) fn floor_naive_datetime(
    datetime: NaiveDateTime,
    unit: &DateTimeFloorUnit,
) -> NaiveDateTime {
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

pub(super) fn resolve_local_floor<T: TimeZone>(
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

pub(super) fn floor_name(unit: &DateTimeFloorUnit) -> &'static str {
    match unit {
        DateTimeFloorUnit::Second => "dt/floor-s",
        DateTimeFloorUnit::Minute => "dt/floor-m",
        DateTimeFloorUnit::Hour => "dt/floor-h",
        DateTimeFloorUnit::Day => "dt/floor-d",
    }
}
