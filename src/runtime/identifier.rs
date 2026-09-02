use chrono::{DateTime, Utc};
use ulid::Ulid;
use uuid::Uuid;

use crate::ast::{ComparisonOperator, Value};

use super::eval::{EvalContext, evaluate};
use super::predicate::apply_ordering;
use super::value::{EvalError, EvalResult, RuntimeValue};

pub(super) fn expect_uuid(
    value: RuntimeValue,
    function: &'static str,
    argument: usize,
) -> EvalResult<Uuid> {
    match value {
        RuntimeValue::Uuid(value) => Ok(value),
        RuntimeValue::String(value) => Uuid::try_parse(&value).map_err(|_| {
            EvalError::conversion(function, argument, "UUID", value, "is not a valid UUID")
        }),
        value => Err(EvalError::conversion(
            function,
            argument,
            "UUID",
            value.render(),
            format!("has type {}", value.type_name()),
        )),
    }
}

pub(super) fn expect_ulid(
    value: RuntimeValue,
    function: &'static str,
    argument: usize,
) -> EvalResult<Ulid> {
    match value {
        RuntimeValue::Ulid(value) => Ok(value),
        RuntimeValue::String(value) => {
            let normalized = value.to_ascii_uppercase();
            if normalized.len() != 26
                || normalized
                    .as_bytes()
                    .first()
                    .is_none_or(|first| *first > b'7')
            {
                return Err(EvalError::conversion(
                    function,
                    argument,
                    "ULID",
                    value,
                    "is not a valid 128-bit ULID",
                ));
            }
            normalized.parse().map_err(|_| {
                EvalError::conversion(function, argument, "ULID", value, "is not a valid ULID")
            })
        }
        value => Err(EvalError::conversion(
            function,
            argument,
            "ULID",
            value.render(),
            format!("has type {}", value.type_name()),
        )),
    }
}

pub(super) fn compare_uuid(
    operator: &ComparisonOperator,
    left: &Value,
    right: &Value,
    record: &EvalContext<'_, '_, '_>,
) -> EvalResult<bool> {
    let function = comparison_name("uuid", operator);
    let left = expect_uuid(evaluate(left, record)?, function, 1)?;
    let right = expect_uuid(evaluate(right, record)?, function, 2)?;
    Ok(apply_ordering(operator, Some(left.cmp(&right))))
}

pub(super) fn compare_ulid(
    operator: &ComparisonOperator,
    left: &Value,
    right: &Value,
    record: &EvalContext<'_, '_, '_>,
) -> EvalResult<bool> {
    let function = comparison_name("ulid", operator);
    let left = expect_ulid(evaluate(left, record)?, function, 1)?;
    let right = expect_ulid(evaluate(right, record)?, function, 2)?;
    Ok(apply_ordering(operator, Some(left.cmp(&right))))
}

pub(super) fn uuid_time(uuid: Uuid) -> EvalResult<RuntimeValue> {
    let Some(timestamp) = uuid.get_timestamp() else {
        return Err(EvalError::conversion(
            "uuid/time",
            1,
            "UUID version 1, 6, or 7",
            uuid.to_string(),
            format!("is version {} and has no timestamp", uuid.get_version_num()),
        ));
    };
    let (seconds, nanoseconds) = match uuid.get_version_num() {
        1 | 6 => {
            let (ticks, _) = timestamp.to_gregorian();
            let unix_ticks =
                i128::from(ticks) - i128::from(uuid::timestamp::UUID_TICKS_BETWEEN_EPOCHS);
            let seconds = unix_ticks.div_euclid(10_000_000);
            let nanoseconds = unix_ticks.rem_euclid(10_000_000) * 100;
            let Ok(seconds) = i64::try_from(seconds) else {
                return Err(uuid_time_range_error(uuid));
            };
            (seconds, nanoseconds as u32)
        }
        7 => {
            let (seconds, nanoseconds) = timestamp.to_unix();
            let Ok(seconds) = i64::try_from(seconds) else {
                return Err(uuid_time_range_error(uuid));
            };
            (seconds, nanoseconds)
        }
        _ => unreachable!("get_timestamp only returns timestamps for UUID versions 1, 6, and 7"),
    };
    DateTime::<Utc>::from_timestamp(seconds, nanoseconds)
        .map(RuntimeValue::DateTime)
        .ok_or_else(|| uuid_time_range_error(uuid))
}

fn uuid_time_range_error(uuid: Uuid) -> EvalError {
    EvalError::conversion(
        "uuid/time",
        1,
        "UUID with a representable timestamp",
        uuid.to_string(),
        "has a timestamp outside the supported DateTime range",
    )
}

pub(super) fn ulid_time(ulid: Ulid) -> EvalResult<RuntimeValue> {
    let milliseconds = ulid.timestamp_ms();
    let seconds = (milliseconds / 1_000) as i64;
    let nanoseconds = ((milliseconds % 1_000) * 1_000_000) as u32;
    DateTime::<Utc>::from_timestamp(seconds, nanoseconds)
        .map(RuntimeValue::DateTime)
        .ok_or_else(|| {
            EvalError::conversion(
                "ulid/time",
                1,
                "ULID with a representable timestamp",
                ulid.to_string(),
                "has a timestamp outside the supported DateTime range",
            )
        })
}

fn comparison_name(prefix: &'static str, operator: &ComparisonOperator) -> &'static str {
    match (prefix, operator) {
        ("uuid", ComparisonOperator::GreaterThan) => "uuid/>",
        ("uuid", ComparisonOperator::GreaterThanOrEqual) => "uuid/>=",
        ("uuid", ComparisonOperator::LessThan) => "uuid/<",
        ("uuid", ComparisonOperator::LessThanOrEqual) => "uuid/<=",
        ("uuid", ComparisonOperator::Equal) => "uuid/=",
        ("uuid", ComparisonOperator::NotEqual) => "uuid/!=",
        ("ulid", ComparisonOperator::GreaterThan) => "ulid/>",
        ("ulid", ComparisonOperator::GreaterThanOrEqual) => "ulid/>=",
        ("ulid", ComparisonOperator::LessThan) => "ulid/<",
        ("ulid", ComparisonOperator::LessThanOrEqual) => "ulid/<=",
        ("ulid", ComparisonOperator::Equal) => "ulid/=",
        ("ulid", ComparisonOperator::NotEqual) => "ulid/!=",
        _ => unreachable!("identifier comparison prefix is fixed"),
    }
}
