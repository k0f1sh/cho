use std::fmt;
use std::net::IpAddr;

use chrono::{DateTime, SecondsFormat, TimeDelta, Utc};
use ulid::Ulid;
use uuid::Uuid;

use super::datetime::render_duration;

// Only types produced by value expressions and passed to other value expressions have runtime
// variants. Cidr, Url, and SemVer remain contextually parsed by their consumers until the
// language has expressions that naturally produce values of those types.
#[derive(Debug, Clone)]
pub(super) enum RuntimeValue {
    String(String),
    Number(f64),
    Boolean(bool),
    DateTime(DateTime<Utc>),
    Duration(TimeDelta),
    IpAddr(IpAddr),
    Uuid(Uuid),
    Ulid(Ulid),
}

impl RuntimeValue {
    pub(super) fn render(&self) -> String {
        match self {
            Self::String(value) => value.clone(),
            Self::Number(value) => value.to_string(),
            Self::Boolean(value) => value.to_string(),
            Self::DateTime(value) => value.to_rfc3339_opts(SecondsFormat::AutoSi, true),
            Self::Duration(value) => render_duration(value),
            Self::IpAddr(value) => value.to_string(),
            Self::Uuid(value) => value.to_string(),
            Self::Ulid(value) => value.to_string(),
        }
    }

    pub(super) fn type_name(&self) -> &'static str {
        match self {
            Self::String(_) => "String",
            Self::Number(_) => "Number",
            Self::Boolean(_) => "Boolean",
            Self::DateTime(_) => "DateTime",
            Self::Duration(_) => "Duration",
            Self::IpAddr(_) => "IpAddr",
            Self::Uuid(_) => "UUID",
            Self::Ulid(_) => "ULID",
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        matches!(self, Self::String(value) if value.is_empty())
    }
}

#[derive(Debug)]
pub(super) struct EvalError {
    function: &'static str,
    argument: usize,
    expected: &'static str,
    input: String,
    reason: String,
}

impl EvalError {
    pub(super) fn conversion(
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

pub(super) type EvalResult<T> = Result<T, EvalError>;

pub(super) fn expect_number(
    value: RuntimeValue,
    function: &'static str,
    argument: usize,
) -> EvalResult<f64> {
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

pub(super) fn expect_boolean(
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

pub(super) fn exact_u64_number(
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

pub(super) fn expect_string(
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
