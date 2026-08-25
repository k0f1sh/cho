use super::*;

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

pub(super) fn string_test_name(kind: &StringTest) -> &'static str {
    match kind {
        StringTest::StartsWith => "s/starts-with?",
        StringTest::EndsWith => "s/ends-with?",
        StringTest::Contains => "s/contains?",
    }
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
