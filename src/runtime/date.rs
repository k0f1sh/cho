use chrono::{Datelike, NaiveDate, TimeDelta};

use crate::ast::DatePart;

use super::value::{EvalError, EvalResult, RuntimeValue, expect_number};

pub(super) fn expect_date(
    value: RuntimeValue,
    function: &'static str,
    argument: usize,
) -> EvalResult<NaiveDate> {
    match value {
        RuntimeValue::Date(value) => Ok(value),
        RuntimeValue::String(value) => parse(&value)
            .map_err(|reason| EvalError::conversion(function, argument, "Date", value, reason)),
        value => Err(EvalError::conversion(
            function,
            argument,
            "Date",
            value.render(),
            format!("has type {}", value.type_name()),
        )),
    }
}

fn parse(value: &str) -> Result<NaiveDate, &'static str> {
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes[..4].iter().all(u8::is_ascii_digit)
        || !bytes[5..7].iter().all(u8::is_ascii_digit)
        || !bytes[8..].iter().all(u8::is_ascii_digit)
    {
        return Err("is not in YYYY-MM-DD format");
    }
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| "is not a valid calendar date")
}

pub(super) fn render(value: &NaiveDate) -> String {
    value.format("%Y-%m-%d").to_string()
}

pub(super) fn part(value: NaiveDate, part: &DatePart) -> RuntimeValue {
    let number = match part {
        DatePart::Year => value.year() as f64,
        DatePart::Month => value.month() as f64,
        DatePart::Day => value.day() as f64,
        DatePart::Weekday => value.weekday().number_from_monday() as f64,
    };
    RuntimeValue::Number(number)
}

pub(super) fn part_name(part: &DatePart) -> &'static str {
    match part {
        DatePart::Year => "d/year",
        DatePart::Month => "d/month",
        DatePart::Day => "d/day",
        DatePart::Weekday => "d/weekday",
    }
}

pub(super) fn expect_days(value: RuntimeValue, function: &'static str) -> EvalResult<TimeDelta> {
    let input = value.render();
    let days = expect_number(value, function, 2)?;
    if days.fract() != 0.0 || days < i64::MIN as f64 || days > i64::MAX as f64 {
        return Err(EvalError::conversion(
            function,
            2,
            "Number (whole calendar days)",
            input,
            "is not a representable whole day count",
        ));
    }
    TimeDelta::try_days(days as i64).ok_or_else(|| {
        EvalError::conversion(
            function,
            2,
            "Number (whole calendar days)",
            input,
            "is outside the supported day range",
        )
    })
}

pub(super) fn checked_result(
    result: Option<NaiveDate>,
    function: &'static str,
    days: &TimeDelta,
) -> EvalResult<RuntimeValue> {
    result
        .filter(|date| (0..=9999).contains(&date.year()))
        .map(RuntimeValue::Date)
        .ok_or_else(|| {
            EvalError::conversion(
                function,
                2,
                "Number of days producing a Date from 0000-01-01 through 9999-12-31",
                days.num_days().to_string(),
                "overflows the supported Date range",
            )
        })
}
