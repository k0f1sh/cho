use super::*;

pub(super) fn matches(predicate: &Predicate, record: &EvalContext<'_, '_, '_>) -> EvalResult<bool> {
    match predicate {
        Predicate::Compare {
            kind,
            operator,
            left,
            right,
        } => compare(kind, operator, left, right, record),
        Predicate::Regex { target, regex } => Ok(record
            .regexes
            .get(regex.0)
            .expect("RegexId is assigned from this program's regex pool")
            .is_match(&evaluate(target, record)?.render())),
        Predicate::StringTest {
            kind,
            value,
            pattern,
        } => {
            let function = string_test_name(kind);
            let value = expect_string(evaluate(value, record)?, function, 1)?;
            let pattern = expect_string(evaluate(pattern, record)?, function, 2)?;
            Ok(match kind {
                StringTest::StartsWith => value.starts_with(&pattern),
                StringTest::EndsWith => value.ends_with(&pattern),
                StringTest::Contains => value.contains(&pattern),
            })
        }
        Predicate::IpClass { kind, value } => {
            let ip = expect_ip(evaluate(value, record)?, ip_class_name(kind), 1)?;
            Ok(matches_ip_class(ip, kind))
        }
        Predicate::CidrContains { cidr, ip } => {
            let cidr = expect_cidr(evaluate(cidr, record)?, "cidr/contains?", 1)?;
            let ip = expect_ip(evaluate(ip, record)?, "cidr/contains?", 2)?;
            Ok(cidr.contains(&ip))
        }
        Predicate::UrlQueryHas { name, url } => {
            let input = expect_string(evaluate(url, record)?, "url/query-has?", 1)?;
            let url = parse_absolute_url(&input, "url/query-has?", 1)?;
            let name = expect_string(evaluate(name, record)?, "url/query-has?", 2)?;
            Ok(url.query_pairs().any(|(key, _)| key == name))
        }
    }
}

pub(super) fn compare(
    kind: &ComparisonType,
    operator: &ComparisonOperator,
    left: &Value,
    right: &Value,
    record: &EvalContext<'_, '_, '_>,
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
        ComparisonType::SemVer => semver::compare(operator, left, right, record),
    }
}

pub(super) fn apply_ordering(
    operator: &ComparisonOperator,
    ordering: Option<std::cmp::Ordering>,
) -> bool {
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

pub(super) fn comparison_name(
    kind: &ComparisonType,
    operator: &ComparisonOperator,
) -> &'static str {
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
        (ComparisonType::SemVer, ComparisonOperator::GreaterThan) => "semver/>",
        (ComparisonType::SemVer, ComparisonOperator::GreaterThanOrEqual) => "semver/>=",
        (ComparisonType::SemVer, ComparisonOperator::LessThan) => "semver/<",
        (ComparisonType::SemVer, ComparisonOperator::LessThanOrEqual) => "semver/<=",
        (ComparisonType::SemVer, ComparisonOperator::Equal) => "semver/=",
        (ComparisonType::SemVer, ComparisonOperator::NotEqual) => "semver/!=",
    }
}
