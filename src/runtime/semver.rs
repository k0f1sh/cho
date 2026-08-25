use ::semver::Version;

use super::*;

pub(super) fn evaluate_part(
    part: &SemVerPart,
    value: &Value,
    record: &EvalContext<'_, '_, '_>,
) -> EvalResult<RuntimeValue> {
    let function = part_name(part);
    let version = expect(evaluate(value, record)?, function, 1)?;
    match part {
        SemVerPart::Major => exact_u64_number(version.major, function, 1, version.to_string()),
        SemVerPart::Minor => exact_u64_number(version.minor, function, 1, version.to_string()),
        SemVerPart::Patch => exact_u64_number(version.patch, function, 1, version.to_string()),
        SemVerPart::Prerelease => Ok(RuntimeValue::String(version.pre.to_string())),
    }
}

pub(super) fn compare(
    operator: &ComparisonOperator,
    left: &Value,
    right: &Value,
    record: &EvalContext<'_, '_, '_>,
) -> EvalResult<bool> {
    let function = comparison_name(operator);
    let left = expect(evaluate(left, record)?, function, 1)?;
    let right = expect(evaluate(right, record)?, function, 2)?;
    Ok(apply_ordering(operator, Some(left.cmp_precedence(&right))))
}

fn expect(value: RuntimeValue, function: &'static str, argument: usize) -> EvalResult<Version> {
    match value {
        RuntimeValue::String(value) => value.parse().map_err(|_| {
            EvalError::conversion(
                function,
                argument,
                "SemVer",
                value,
                "is not a valid MAJOR.MINOR.PATCH semantic version",
            )
        }),
        value => Err(EvalError::conversion(
            function,
            argument,
            "SemVer",
            value.render(),
            format!("has type {}", value.type_name()),
        )),
    }
}

fn part_name(part: &SemVerPart) -> &'static str {
    match part {
        SemVerPart::Major => "semver/major",
        SemVerPart::Minor => "semver/minor",
        SemVerPart::Patch => "semver/patch",
        SemVerPart::Prerelease => "semver/prerelease",
    }
}

fn comparison_name(operator: &ComparisonOperator) -> &'static str {
    match operator {
        ComparisonOperator::GreaterThan => "semver/>",
        ComparisonOperator::GreaterThanOrEqual => "semver/>=",
        ComparisonOperator::LessThan => "semver/<",
        ComparisonOperator::LessThanOrEqual => "semver/<=",
        ComparisonOperator::Equal => "semver/=",
        ComparisonOperator::NotEqual => "semver/!=",
    }
}
