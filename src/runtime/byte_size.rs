use rust_decimal::Decimal;

use super::value::{EvalError, EvalResult, RuntimeValue};

const MAX_SAFE_INTEGER: u64 = (1_u64 << 53) - 1;
const UNITS: [(&str, u128); 11] = [
    ("PiB", 1_u128 << 50),
    ("TiB", 1_u128 << 40),
    ("GiB", 1_u128 << 30),
    ("MiB", 1_u128 << 20),
    ("KiB", 1_u128 << 10),
    ("PB", 1_000_000_000_000_000),
    ("TB", 1_000_000_000_000),
    ("GB", 1_000_000_000),
    ("MB", 1_000_000),
    ("kB", 1_000),
    ("B", 1),
];

pub(super) fn expect(
    value: RuntimeValue,
    function: &'static str,
    argument: usize,
) -> EvalResult<Decimal> {
    match value {
        RuntimeValue::ByteSize(value) => Ok(value),
        RuntimeValue::String(value) => parse(&value)
            .map_err(|reason| EvalError::conversion(function, argument, "ByteSize", value, reason)),
        value => Err(EvalError::conversion(
            function,
            argument,
            "ByteSize",
            value.render(),
            format!("has type {}", value.type_name()),
        )),
    }
}

fn parse(input: &str) -> Result<Decimal, &'static str> {
    let (number, multiplier) = UNITS
        .iter()
        .find_map(|(unit, multiplier)| input.strip_suffix(unit).map(|number| (number, *multiplier)))
        .ok_or("is not a valid byte size")?;
    let number = number.trim_end_matches([' ', '\t']);
    if number.is_empty() || !valid_number(number) {
        return Err("is not a valid byte size");
    }
    let number = Decimal::from_str_exact(number)
        .map(|value| value.normalize())
        .map_err(|_| "is outside the supported range")?;
    multiply_exact(number, multiplier).ok_or("is outside the supported range")
}

fn multiply_exact(number: Decimal, mut multiplier: u128) -> Option<Decimal> {
    let mut mantissa = u128::try_from(number.mantissa()).ok()?;
    let mut scale = number.scale();

    while scale > 0 && multiplier.is_multiple_of(10) {
        multiplier /= 10;
        scale -= 1;
    }
    while scale > 0 && multiplier.is_multiple_of(2) && mantissa.is_multiple_of(5) {
        multiplier /= 2;
        mantissa /= 5;
        scale -= 1;
    }

    let mantissa = mantissa.checked_mul(multiplier)?;
    let mantissa = i128::try_from(mantissa).ok()?;
    Decimal::try_from_i128_with_scale(mantissa, scale)
        .ok()
        .map(|value| value.normalize())
}

fn valid_number(number: &str) -> bool {
    let mut parts = number.split('.');
    let integer = parts.next().unwrap_or_default();
    let fraction = parts.next();
    !integer.is_empty()
        && integer.bytes().all(|byte| byte.is_ascii_digit())
        && fraction.is_none_or(|fraction| {
            !fraction.is_empty() && fraction.bytes().all(|byte| byte.is_ascii_digit())
        })
        && parts.next().is_none()
}

pub(super) fn to_number(bytes: Decimal) -> Result<RuntimeValue, EvalError> {
    if bytes > Decimal::from(MAX_SAFE_INTEGER) {
        return Err(EvalError::conversion(
            "bs/to-b",
            1,
            "ByteSize no greater than 2^53 - 1 B",
            format!("{}B", bytes.normalize()),
            "is outside Number's safe integer range",
        ));
    }
    let number = f64::try_from(bytes).expect("supported ByteSize values convert to finite f64");
    Ok(RuntimeValue::Number(number))
}
