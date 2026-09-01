use crate::ast::{StringQuote, Value};

use super::eval::{EvalContext, evaluate};
use super::value::{EvalError, EvalResult, RuntimeValue, expect_number};

pub(super) fn evaluate_string_slice(
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

pub(super) fn expect_slice_index(
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

pub(super) fn expect_part_position(
    value: &Value,
    function: &'static str,
    record: &EvalContext<'_, '_, '_>,
) -> EvalResult<usize> {
    let position = expect_number(evaluate(value, record)?, function, 3)?;
    if position.fract() != 0.0 || position < 1.0 {
        return Err(EvalError::conversion(
            function,
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
            function,
            3,
            "Number (representable part position)",
            position_input,
            "is outside the supported position range",
        ));
    }
    Ok(position as usize)
}

pub(super) fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

pub(super) fn quote(value: &str, kind: &StringQuote) -> String {
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

pub(super) fn unquote(value: &str) -> EvalResult<RuntimeValue> {
    let first = value.chars().next();
    let last = value.chars().next_back();
    let boundary_is_quote = |character| matches!(character, Some('\'' | '"'));

    if !boundary_is_quote(first) && !boundary_is_quote(last) {
        return Ok(RuntimeValue::String(value.to_owned()));
    }
    if first != last || value.len() < 2 {
        return Err(EvalError::conversion(
            "s/unquote",
            1,
            "matching single or double quotes",
            value,
            "has mismatched quote boundaries",
        ));
    }

    let delimiter = first.expect("a quoted boundary was established");
    let inner = &value[delimiter.len_utf8()..value.len() - delimiter.len_utf8()];
    let mut decoded = String::with_capacity(inner.len());
    let mut characters = inner.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            if character == delimiter {
                return Err(EvalError::conversion(
                    "s/unquote",
                    1,
                    "a valid quoted string",
                    value,
                    "contains an unescaped enclosing quote",
                ));
            }
            decoded.push(character);
            continue;
        }
        let Some(escaped) = characters.next() else {
            return Err(EvalError::conversion(
                "s/unquote",
                1,
                "a valid quoted string",
                value,
                "ends with an incomplete escape",
            ));
        };
        match escaped {
            '\\' => decoded.push('\\'),
            'n' => decoded.push('\n'),
            'r' => decoded.push('\r'),
            't' => decoded.push('\t'),
            escaped if escaped == delimiter => decoded.push(escaped),
            escaped => {
                return Err(EvalError::conversion(
                    "s/unquote",
                    1,
                    "a valid quoted string",
                    value,
                    format!("contains unsupported escape \\\\{escaped}"),
                ));
            }
        }
    }
    Ok(RuntimeValue::String(decoded))
}

pub(super) fn shell_quote(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('\'');
    quoted.push_str(&value.replace('\'', "'\\''"));
    quoted.push('\'');
    quoted
}
