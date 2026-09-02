use crate::ast::{ArithmeticOperator, NumberOperator, Value};

use super::eval::{EvalContext, evaluate};
use super::value::{EvalError, EvalResult, RuntimeValue, expect_number};

pub(super) fn evaluate_arithmetic(
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
        ArithmeticOperator::Remainder => "%",
    };
    let left = expect_number(evaluate(left, record)?, function, 1)?;
    let right = expect_number(evaluate(right, record)?, function, 2)?;
    if matches!(
        operator,
        ArithmeticOperator::Divide | ArithmeticOperator::Remainder
    ) && right == 0.0
    {
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
        ArithmeticOperator::Remainder => left % right,
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
    Ok(RuntimeValue::Number(
        if matches!(operator, ArithmeticOperator::Remainder) && result == 0.0 {
            0.0
        } else {
            result
        },
    ))
}

pub(super) fn evaluate_operation(
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

pub(super) fn format_fixed(
    value: &Value,
    digits: &Value,
    record: &EvalContext<'_, '_, '_>,
) -> EvalResult<RuntimeValue> {
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

pub(super) fn evaluate_extreme(
    values: &[Value],
    function: &'static str,
    choose: impl Fn(f64, f64) -> f64,
    record: &EvalContext<'_, '_, '_>,
) -> EvalResult<RuntimeValue> {
    let mut numbers = values
        .iter()
        .enumerate()
        .map(|(index, value)| expect_number(evaluate(value, record)?, function, index + 1));
    let first = numbers
        .next()
        .expect("n/min and n/max require at least one argument")?;
    let result = numbers.try_fold(first, |result, number| {
        number.map(|number| choose(result, number))
    })?;
    Ok(RuntimeValue::Number(if result == 0.0 {
        0.0
    } else {
        result
    }))
}

pub(super) fn clamp(
    value: &Value,
    minimum: &Value,
    maximum: &Value,
    record: &EvalContext<'_, '_, '_>,
) -> EvalResult<RuntimeValue> {
    let value = expect_number(evaluate(value, record)?, "n/clamp", 1)?;
    let minimum = expect_number(evaluate(minimum, record)?, "n/clamp", 2)?;
    let maximum = expect_number(evaluate(maximum, record)?, "n/clamp", 3)?;
    if minimum > maximum {
        return Err(EvalError::conversion(
            "n/clamp",
            3,
            "Number greater than or equal to MIN",
            maximum.to_string(),
            "is less than MIN",
        ));
    }
    let result = value.clamp(minimum, maximum);
    Ok(RuntimeValue::Number(if result == 0.0 {
        0.0
    } else {
        result
    }))
}
