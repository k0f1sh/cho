use crate::ast::{
    ArithmeticOperator, ComparisonOperator, ComparisonType, DateTimeFloorUnit, Expr, IpClass,
    Predicate, Program, RegexId, UrlEncoding, UrlPart, Value,
};
use crate::lexer::{Token, tokenize};

#[derive(Debug, PartialEq)]
pub enum ParseError {
    InvalidSyntax,
    InvalidField,
    UnterminatedString,
    UnterminatedRegex,
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
    regex_patterns: Vec<String>,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
            regex_patterns: Vec::new(),
        }
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.position).cloned();
        self.position += usize::from(token.is_some());
        token
    }

    fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut expressions = Vec::new();
        while self.position < self.tokens.len() {
            expressions.push(self.parse_expression()?);
        }
        Ok(Program {
            expressions,
            regex_patterns: std::mem::take(&mut self.regex_patterns),
        })
    }

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        if self.next() != Some(Token::LeftParen) {
            return Err(ParseError::InvalidSyntax);
        }
        match self.next() {
            Some(Token::Atom(operator)) if operator == "print" || operator == "p" => {
                Ok(Expr::Print(self.parse_values_until_right_paren()?))
            }
            Some(Token::Atom(operator)) if operator == "filter" || operator == "f" => {
                let predicate = self.parse_predicate()?;
                if self.next() != Some(Token::RightParen) {
                    return Err(ParseError::InvalidSyntax);
                }
                Ok(Expr::Filter(predicate))
            }
            _ => Err(ParseError::InvalidSyntax),
        }
    }

    fn parse_predicate(&mut self) -> Result<Predicate, ParseError> {
        if self.next() != Some(Token::LeftParen) {
            return Err(ParseError::InvalidSyntax);
        }
        let operator = self.next();
        self.parse_predicate_after_operator(operator)
    }

    fn parse_predicate_after_operator(
        &mut self,
        operator: Option<Token>,
    ) -> Result<Predicate, ParseError> {
        if matches!(operator, Some(Token::Atom(ref value)) if value == "reg" || value == "~") {
            return self.parse_regex_predicate();
        }
        if operator == Some(Token::Atom("not".into())) {
            let predicate = self.parse_predicate()?;
            if self.next() != Some(Token::RightParen) {
                return Err(ParseError::InvalidSyntax);
            }
            return Ok(Predicate::Not(Box::new(predicate)));
        }
        if operator == Some(Token::Atom("and".into())) {
            return Ok(Predicate::And(self.parse_predicates_until_right_paren()?));
        }
        if operator == Some(Token::Atom("or".into())) {
            return Ok(Predicate::Or(self.parse_predicates_until_right_paren()?));
        }
        if let Some(kind) = match operator {
            Some(Token::Atom(ref operator)) => ip_class(operator),
            _ => None,
        } {
            let value = self.parse_value()?;
            self.expect_right_paren()?;
            return Ok(Predicate::IpClass { kind, value });
        }
        if operator == Some(Token::Atom("cidr/contains?".into())) {
            let cidr = self.parse_value()?;
            let ip = self.parse_value()?;
            self.expect_right_paren()?;
            return Ok(Predicate::CidrContains { cidr, ip });
        }
        let (kind, operator) = match operator {
            Some(Token::Atom(value)) => {
                parse_comparison_operator(&value).ok_or(ParseError::InvalidSyntax)?
            }
            _ => return Err(ParseError::InvalidSyntax),
        };
        let left = self.parse_value()?;
        let right = self.parse_value()?;
        if self.next() != Some(Token::RightParen) {
            return Err(ParseError::InvalidSyntax);
        }
        Ok(Predicate::Compare {
            kind,
            operator,
            left,
            right,
        })
    }

    fn expect_right_paren(&mut self) -> Result<(), ParseError> {
        if self.next() == Some(Token::RightParen) {
            Ok(())
        } else {
            Err(ParseError::InvalidSyntax)
        }
    }

    fn parse_regex_predicate(&mut self) -> Result<Predicate, ParseError> {
        if let Some(Token::Regex(pattern)) = self.tokens.get(self.position).cloned() {
            self.position += 1;
            if self.next() != Some(Token::RightParen) {
                return Err(ParseError::InvalidSyntax);
            }
            let regex = self.register_regex(pattern);
            return Ok(Predicate::Regex {
                target: Value::Field(0),
                regex,
            });
        }

        let first = self.parse_value()?;
        let (target, pattern) = if self.tokens.get(self.position) == Some(&Token::RightParen) {
            let Value::String(pattern) = first else {
                return Err(ParseError::InvalidSyntax);
            };
            (Value::Field(0), pattern)
        } else {
            let pattern = match self.next() {
                Some(Token::String(pattern) | Token::Regex(pattern)) => pattern,
                _ => return Err(ParseError::InvalidSyntax),
            };
            (first, pattern)
        };
        if self.next() != Some(Token::RightParen) {
            return Err(ParseError::InvalidSyntax);
        }
        let regex = self.register_regex(pattern);
        Ok(Predicate::Regex { target, regex })
    }

    fn register_regex(&mut self, pattern: String) -> RegexId {
        let id = RegexId(self.regex_patterns.len());
        self.regex_patterns.push(pattern);
        id
    }

    fn parse_predicates_until_right_paren(&mut self) -> Result<Vec<Predicate>, ParseError> {
        let mut predicates = Vec::new();
        while self.tokens.get(self.position) != Some(&Token::RightParen) {
            predicates.push(self.parse_predicate()?);
        }
        self.position += 1;
        if predicates.is_empty() {
            return Err(ParseError::InvalidSyntax);
        }
        Ok(predicates)
    }

    fn parse_values_until_right_paren(&mut self) -> Result<Vec<Value>, ParseError> {
        let mut values = Vec::new();
        loop {
            match self.tokens.get(self.position) {
                Some(Token::RightParen) => {
                    self.position += 1;
                    return Ok(values);
                }
                Some(_) => values.push(self.parse_value()?),
                None => return Err(ParseError::InvalidSyntax),
            }
        }
    }

    fn parse_value(&mut self) -> Result<Value, ParseError> {
        match self.next() {
            Some(Token::Atom(value)) if value == "NR" => Ok(Value::RecordNumber),
            Some(Token::Atom(value)) if value == "NF" => Ok(Value::FieldCount),
            Some(Token::Atom(value)) if value.parse::<f64>().is_ok() => {
                Ok(Value::Number(value.parse().expect("number was validated")))
            }
            Some(Token::Atom(field)) => {
                let number = field
                    .strip_prefix('$')
                    .ok_or(ParseError::InvalidSyntax)?
                    .parse::<usize>()
                    .map_err(|_| ParseError::InvalidField)?;
                Ok(Value::Field(number))
            }
            Some(Token::String(value)) => Ok(Value::String(value)),
            Some(Token::LeftParen) => match self.next() {
                Some(Token::Atom(operator)) if operator == "->" => self.parse_threading(true),
                Some(Token::Atom(operator)) if operator == "->>" => self.parse_threading(false),
                Some(Token::Atom(operator)) if operator == "str" => {
                    Ok(Value::Concat(self.parse_values_until_right_paren()?))
                }
                Some(Token::Atom(operator))
                    if matches!(operator.as_str(), "+" | "-" | "*" | "/") =>
                {
                    let left = self.parse_value()?;
                    let right = self.parse_value()?;
                    self.expect_right_paren()?;
                    Ok(Value::Arithmetic {
                        operator: arithmetic_operator(&operator).expect("operator was matched"),
                        left: Box::new(left),
                        right: Box::new(right),
                    })
                }
                Some(Token::Atom(operator)) if operator == "n/fixed" => {
                    let digits = self.parse_value()?;
                    let value = self.parse_value()?;
                    self.expect_right_paren()?;
                    Ok(Value::FormatNumberFixed {
                        digits: Box::new(digits),
                        value: Box::new(value),
                    })
                }
                Some(Token::Atom(operator)) if url_part(&operator).is_some() => {
                    let value = self.parse_value()?;
                    self.expect_right_paren()?;
                    Ok(Value::UrlPart {
                        part: url_part(&operator).expect("operator was matched"),
                        value: Box::new(value),
                    })
                }
                Some(Token::Atom(operator)) if url_encoding(&operator).is_some() => {
                    let value = self.parse_value()?;
                    self.expect_right_paren()?;
                    Ok(Value::UrlEncoding {
                        operation: url_encoding(&operator).expect("operator was matched"),
                        value: Box::new(value),
                    })
                }
                Some(Token::Atom(operator)) if operator == "s/join" => {
                    let separator = self.parse_value()?;
                    Ok(Value::Join {
                        separator: Box::new(separator),
                        values: self.parse_values_until_right_paren()?,
                    })
                }
                Some(Token::Atom(operator)) if operator == "s/part" => {
                    let delimiter = self.parse_value()?;
                    let position = self.parse_value()?;
                    let value = self.parse_value()?;
                    self.expect_right_paren()?;
                    Ok(Value::Part {
                        delimiter: Box::new(delimiter),
                        position: Box::new(position),
                        value: Box::new(value),
                    })
                }
                Some(Token::Atom(operator)) if operator == "s/count" => {
                    let value = self.parse_value()?;
                    if self.next() != Some(Token::RightParen) {
                        return Err(ParseError::InvalidSyntax);
                    }
                    Ok(Value::Count(Box::new(value)))
                }
                Some(Token::Atom(operator)) if operator == "s/escape" => {
                    let value = self.parse_value()?;
                    if self.next() != Some(Token::RightParen) {
                        return Err(ParseError::InvalidSyntax);
                    }
                    Ok(Value::Escape(Box::new(value)))
                }
                Some(Token::Atom(operator)) if operator == "if" => {
                    let predicate = self.parse_predicate()?;
                    let then_value = self.parse_value()?;
                    let else_value = self.parse_value()?;
                    if self.next() != Some(Token::RightParen) {
                        return Err(ParseError::InvalidSyntax);
                    }
                    Ok(Value::If {
                        predicate: Box::new(predicate),
                        then_value: Box::new(then_value),
                        else_value: Box::new(else_value),
                    })
                }
                Some(Token::Atom(operator)) if operator == "s/lower" => {
                    let value = self.parse_value()?;
                    if self.next() != Some(Token::RightParen) {
                        return Err(ParseError::InvalidSyntax);
                    }
                    Ok(Value::Lower(Box::new(value)))
                }
                Some(Token::Atom(operator)) if operator == "s/upper" => {
                    let value = self.parse_value()?;
                    if self.next() != Some(Token::RightParen) {
                        return Err(ParseError::InvalidSyntax);
                    }
                    Ok(Value::Upper(Box::new(value)))
                }
                Some(Token::Atom(operator)) if operator == "default" => {
                    let value = self.parse_value()?;
                    let fallback = self.parse_value()?;
                    if self.next() != Some(Token::RightParen) {
                        return Err(ParseError::InvalidSyntax);
                    }
                    Ok(Value::Default {
                        value: Box::new(value),
                        fallback: Box::new(fallback),
                    })
                }
                Some(Token::Atom(operator)) if operator == "dt/unix" => {
                    let value = self.parse_value()?;
                    self.expect_right_paren()?;
                    Ok(Value::DateTimeFromUnix(Box::new(value)))
                }
                Some(Token::Atom(operator)) if operator == "dt/fmt" => {
                    let format = self.parse_value()?;
                    let value = self.parse_value()?;
                    self.expect_right_paren()?;
                    Ok(Value::FormatDateTime {
                        format: Box::new(format),
                        value: Box::new(value),
                    })
                }
                Some(Token::Atom(operator)) if operator == "du/s" => {
                    let value = self.parse_value()?;
                    self.expect_right_paren()?;
                    Ok(Value::DurationSeconds(Box::new(value)))
                }
                Some(Token::Atom(operator)) if operator == "du/ms" => {
                    let value = self.parse_value()?;
                    self.expect_right_paren()?;
                    Ok(Value::DurationMilliseconds(Box::new(value)))
                }
                Some(Token::Atom(operator)) if operator == "du/m" => {
                    let value = self.parse_value()?;
                    self.expect_right_paren()?;
                    Ok(Value::DurationMinutes(Box::new(value)))
                }
                Some(Token::Atom(operator)) if operator == "du/h" => {
                    let value = self.parse_value()?;
                    self.expect_right_paren()?;
                    Ok(Value::DurationHours(Box::new(value)))
                }
                Some(Token::Atom(operator)) if operator == "du/d" => {
                    let value = self.parse_value()?;
                    self.expect_right_paren()?;
                    Ok(Value::DurationDays(Box::new(value)))
                }
                Some(Token::Atom(operator)) if operator == "dt/now" => {
                    self.expect_right_paren()?;
                    Ok(Value::DateTimeNow)
                }
                Some(Token::Atom(operator)) if operator == "dt/floor-s" => {
                    self.parse_datetime_floor(DateTimeFloorUnit::Second)
                }
                Some(Token::Atom(operator)) if operator == "dt/floor-m" => {
                    self.parse_datetime_floor(DateTimeFloorUnit::Minute)
                }
                Some(Token::Atom(operator)) if operator == "dt/floor-h" => {
                    self.parse_datetime_floor(DateTimeFloorUnit::Hour)
                }
                Some(Token::Atom(operator)) if operator == "dt/floor-d" => {
                    self.parse_datetime_floor(DateTimeFloorUnit::Day)
                }
                Some(Token::Atom(operator)) if operator == "dt/add" => {
                    let datetime = self.parse_value()?;
                    let duration = self.parse_value()?;
                    self.expect_right_paren()?;
                    Ok(Value::AddDateTime {
                        datetime: Box::new(datetime),
                        duration: Box::new(duration),
                    })
                }
                Some(Token::Atom(operator)) if operator == "dt/sub" => {
                    let datetime = self.parse_value()?;
                    let duration = self.parse_value()?;
                    self.expect_right_paren()?;
                    Ok(Value::SubtractDateTime {
                        datetime: Box::new(datetime),
                        duration: Box::new(duration),
                    })
                }
                Some(Token::Atom(operator)) if operator == "dt/diff" => {
                    let left = self.parse_value()?;
                    let right = self.parse_value()?;
                    self.expect_right_paren()?;
                    Ok(Value::DifferenceDateTime {
                        left: Box::new(left),
                        right: Box::new(right),
                    })
                }
                operator @ Some(Token::Atom(_)) => Ok(Value::Predicate(Box::new(
                    self.parse_predicate_after_operator(operator)?,
                ))),
                _ => Err(ParseError::InvalidSyntax),
            },
            _ => Err(ParseError::InvalidSyntax),
        }
    }

    fn parse_threading(&mut self, first: bool) -> Result<Value, ParseError> {
        let mut value = self.parse_value()?;
        while self.tokens.get(self.position) != Some(&Token::RightParen) {
            if self.next() != Some(Token::LeftParen) {
                return Err(ParseError::InvalidSyntax);
            }
            let Some(Token::Atom(operator)) = self.next() else {
                return Err(ParseError::InvalidSyntax);
            };
            let mut arguments = self.parse_values_until_right_paren()?;
            if first {
                arguments.insert(0, value);
            } else {
                arguments.push(value);
            }
            value = build_value_application(&operator, arguments)?;
        }
        self.expect_right_paren()?;
        Ok(value)
    }

    fn parse_datetime_floor(&mut self, unit: DateTimeFloorUnit) -> Result<Value, ParseError> {
        let value = self.parse_value()?;
        self.expect_right_paren()?;
        Ok(Value::FloorDateTime {
            unit,
            value: Box::new(value),
        })
    }
}

fn build_value_application(operator: &str, mut arguments: Vec<Value>) -> Result<Value, ParseError> {
    let one = |mut arguments: Vec<Value>| {
        if arguments.len() == 1 {
            Ok(arguments.remove(0))
        } else {
            Err(ParseError::InvalidSyntax)
        }
    };
    let two = |mut arguments: Vec<Value>| {
        if arguments.len() == 2 {
            let right = arguments.pop().expect("length checked");
            let left = arguments.pop().expect("length checked");
            Ok((left, right))
        } else {
            Err(ParseError::InvalidSyntax)
        }
    };
    let three = |mut arguments: Vec<Value>| {
        if arguments.len() == 3 {
            let third = arguments.pop().expect("length checked");
            let second = arguments.pop().expect("length checked");
            let first = arguments.pop().expect("length checked");
            Ok((first, second, third))
        } else {
            Err(ParseError::InvalidSyntax)
        }
    };
    match operator {
        "str" => Ok(Value::Concat(arguments)),
        "+" | "-" | "*" | "/" => {
            let (left, right) = two(arguments)?;
            Ok(Value::Arithmetic {
                operator: arithmetic_operator(operator).expect("operator was matched"),
                left: Box::new(left),
                right: Box::new(right),
            })
        }
        "n/fixed" => {
            let (digits, value) = two(arguments)?;
            Ok(Value::FormatNumberFixed {
                digits: Box::new(digits),
                value: Box::new(value),
            })
        }
        operator if url_part(operator).is_some() => Ok(Value::UrlPart {
            part: url_part(operator).expect("operator was matched"),
            value: Box::new(one(arguments)?),
        }),
        operator if url_encoding(operator).is_some() => Ok(Value::UrlEncoding {
            operation: url_encoding(operator).expect("operator was matched"),
            value: Box::new(one(arguments)?),
        }),
        "s/join" if !arguments.is_empty() => {
            let separator = arguments.remove(0);
            Ok(Value::Join {
                separator: Box::new(separator),
                values: arguments,
            })
        }
        "s/part" => {
            let (delimiter, position, value) = three(arguments)?;
            Ok(Value::Part {
                delimiter: Box::new(delimiter),
                position: Box::new(position),
                value: Box::new(value),
            })
        }
        "s/count" => Ok(Value::Count(Box::new(one(arguments)?))),
        "s/escape" => Ok(Value::Escape(Box::new(one(arguments)?))),
        "s/lower" => Ok(Value::Lower(Box::new(one(arguments)?))),
        "s/upper" => Ok(Value::Upper(Box::new(one(arguments)?))),
        "default" => {
            let (value, fallback) = two(arguments)?;
            Ok(Value::Default {
                value: Box::new(value),
                fallback: Box::new(fallback),
            })
        }
        "dt/unix" => Ok(Value::DateTimeFromUnix(Box::new(one(arguments)?))),
        "dt/floor-s" => Ok(Value::FloorDateTime {
            unit: DateTimeFloorUnit::Second,
            value: Box::new(one(arguments)?),
        }),
        "dt/floor-m" => Ok(Value::FloorDateTime {
            unit: DateTimeFloorUnit::Minute,
            value: Box::new(one(arguments)?),
        }),
        "dt/floor-h" => Ok(Value::FloorDateTime {
            unit: DateTimeFloorUnit::Hour,
            value: Box::new(one(arguments)?),
        }),
        "dt/floor-d" => Ok(Value::FloorDateTime {
            unit: DateTimeFloorUnit::Day,
            value: Box::new(one(arguments)?),
        }),
        "dt/fmt" => {
            let (format, value) = two(arguments)?;
            Ok(Value::FormatDateTime {
                format: Box::new(format),
                value: Box::new(value),
            })
        }
        "du/s" => Ok(Value::DurationSeconds(Box::new(one(arguments)?))),
        "du/ms" => Ok(Value::DurationMilliseconds(Box::new(one(arguments)?))),
        "du/m" => Ok(Value::DurationMinutes(Box::new(one(arguments)?))),
        "du/h" => Ok(Value::DurationHours(Box::new(one(arguments)?))),
        "du/d" => Ok(Value::DurationDays(Box::new(one(arguments)?))),
        "dt/add" => {
            let (datetime, duration) = two(arguments)?;
            Ok(Value::AddDateTime {
                datetime: Box::new(datetime),
                duration: Box::new(duration),
            })
        }
        "dt/sub" => {
            let (datetime, duration) = two(arguments)?;
            Ok(Value::SubtractDateTime {
                datetime: Box::new(datetime),
                duration: Box::new(duration),
            })
        }
        "dt/diff" => {
            let (left, right) = two(arguments)?;
            Ok(Value::DifferenceDateTime {
                left: Box::new(left),
                right: Box::new(right),
            })
        }
        operator if parse_comparison_operator(operator).is_some() => {
            let (left, right) = two(arguments)?;
            let (kind, operator) =
                parse_comparison_operator(operator).expect("operator was matched");
            Ok(Value::Predicate(Box::new(Predicate::Compare {
                kind,
                operator,
                left,
                right,
            })))
        }
        operator if ip_class(operator).is_some() => {
            Ok(Value::Predicate(Box::new(Predicate::IpClass {
                kind: ip_class(operator).expect("operator was matched"),
                value: one(arguments)?,
            })))
        }
        "cidr/contains?" => {
            let (cidr, ip) = two(arguments)?;
            Ok(Value::Predicate(Box::new(Predicate::CidrContains {
                cidr,
                ip,
            })))
        }
        _ => Err(ParseError::InvalidSyntax),
    }
}

fn arithmetic_operator(value: &str) -> Option<ArithmeticOperator> {
    match value {
        "+" => Some(ArithmeticOperator::Add),
        "-" => Some(ArithmeticOperator::Subtract),
        "*" => Some(ArithmeticOperator::Multiply),
        "/" => Some(ArithmeticOperator::Divide),
        _ => None,
    }
}

fn ip_class(value: &str) -> Option<IpClass> {
    match value {
        "ip/private?" => Some(IpClass::Private),
        "ip/loopback?" => Some(IpClass::Loopback),
        "ip/link-local?" => Some(IpClass::LinkLocal),
        "ip/multicast?" => Some(IpClass::Multicast),
        _ => None,
    }
}

fn url_part(value: &str) -> Option<UrlPart> {
    match value {
        "url/scheme" => Some(UrlPart::Scheme),
        "url/host" => Some(UrlPart::Host),
        "url/port" => Some(UrlPart::Port),
        "url/path" => Some(UrlPart::Path),
        "url/query" => Some(UrlPart::Query),
        "url/fragment" => Some(UrlPart::Fragment),
        _ => None,
    }
}

fn url_encoding(value: &str) -> Option<UrlEncoding> {
    match value {
        "url/encode" => Some(UrlEncoding::Encode),
        "url/decode" => Some(UrlEncoding::Decode),
        _ => None,
    }
}

fn parse_comparison_operator(value: &str) -> Option<(ComparisonType, ComparisonOperator)> {
    let (kind, operator) = if let Some(operator) = value.strip_prefix("s/") {
        (ComparisonType::String, operator)
    } else if let Some(operator) = value.strip_prefix("dt/") {
        (ComparisonType::DateTime, operator)
    } else if let Some(operator) = value.strip_prefix("ip/") {
        (ComparisonType::IpAddr, operator)
    } else if let Some(operator) = value.strip_prefix("semver/") {
        (ComparisonType::SemVer, operator)
    } else {
        (ComparisonType::Number, value)
    };
    let operator = match operator {
        ">" => ComparisonOperator::GreaterThan,
        ">=" => ComparisonOperator::GreaterThanOrEqual,
        "<" => ComparisonOperator::LessThan,
        "<=" => ComparisonOperator::LessThanOrEqual,
        "=" => ComparisonOperator::Equal,
        "!=" => ComparisonOperator::NotEqual,
        _ => return None,
    };
    if kind == ComparisonType::IpAddr
        && !matches!(
            operator,
            ComparisonOperator::Equal | ComparisonOperator::NotEqual
        )
    {
        return None;
    }
    Some((kind, operator))
}

pub fn parse(program: &str) -> Result<Program, ParseError> {
    Parser::new(tokenize(program)?).parse_program()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_complete_program() {
        assert_eq!(
            parse(r#"(filter (> (s/count $1) 3)) (print (str NR ":" $1))"#),
            Ok(Program {
                expressions: vec![
                    Expr::Filter(Predicate::Compare {
                        kind: ComparisonType::Number,
                        operator: ComparisonOperator::GreaterThan,
                        left: Value::Count(Box::new(Value::Field(1))),
                        right: Value::Number(3.0),
                    }),
                    Expr::Print(vec![Value::Concat(vec![
                        Value::RecordNumber,
                        Value::String(":".into()),
                        Value::Field(1),
                    ])]),
                ],
                regex_patterns: vec![],
            })
        );
    }

    #[test]
    fn parses_short_top_level_aliases() {
        assert_eq!(
            parse("(f (> $2 20)) (p $1)"),
            parse("(filter (> $2 20)) (print $1)")
        );
        assert_eq!(parse("(p)"), parse("(print)"));
    }

    #[test]
    fn parses_regex_filters() {
        assert!(parse(r#"(filter (reg "error"))"#).is_ok());
        assert!(parse(r#"(filter (reg $1 "^[A-Z]"))"#).is_ok());
        assert!(parse(r#"(filter (reg /^error/))"#).is_ok());
        assert!(parse(r#"(filter (~ $1 /^\d+$/))"#).is_ok());
    }

    #[test]
    fn assigns_regex_ids_in_source_order() {
        let program = parse(r#"(filter (and (reg /error/) (~ $1 "^warn")))"#).unwrap();
        assert_eq!(program.regex_patterns, vec!["error", "^warn"]);
        let Expr::Filter(Predicate::And(predicates)) = &program.expressions[0] else {
            panic!("expected an and filter");
        };
        assert!(matches!(
            predicates.as_slice(),
            [
                Predicate::Regex {
                    regex: RegexId(0),
                    ..
                },
                Predicate::Regex {
                    regex: RegexId(1),
                    ..
                }
            ]
        ));
    }

    #[test]
    fn parses_boolean_predicates() {
        assert!(
            parse(r#"(filter (and (not (reg "debug")) (or (s/= $1 "info") (s/= $1 "warn"))))"#)
                .is_ok()
        );
    }

    #[test]
    fn parses_predicates_as_boolean_values() {
        assert!(parse("(print (> $1 $2) (semver/> $1 $2) (ip/loopback? $1))").is_ok());
        assert!(parse("(print (str \"match=\" (and (> $1 0) (< $1 10))))").is_ok());
        assert_eq!(
            parse("(print (-> $1 (semver/> \"1.0.0\")))"),
            parse("(print (semver/> $1 \"1.0.0\"))")
        );
    }

    #[test]
    fn parses_join_values() {
        assert_eq!(
            parse(r#"(print (s/join "," $1 $2))"#),
            Ok(Program {
                expressions: vec![Expr::Print(vec![Value::Join {
                    separator: Box::new(Value::String(",".into())),
                    values: vec![Value::Field(1), Value::Field(2)],
                }])],
                regex_patterns: vec![],
            })
        );
    }

    #[test]
    fn parses_part_values() {
        assert_eq!(
            parse(r#"(print (s/part (str "]" ":") (s/count "x") $1))"#),
            Ok(Program {
                expressions: vec![Expr::Print(vec![Value::Part {
                    delimiter: Box::new(Value::Concat(vec![
                        Value::String("]".into()),
                        Value::String(":".into()),
                    ])),
                    position: Box::new(Value::Count(Box::new(Value::String("x".into())))),
                    value: Box::new(Value::Field(1)),
                }])],
                regex_patterns: vec![],
            })
        );
        assert_eq!(
            parse(r#"(print (->> $1 (s/part "=" 2)))"#),
            parse(r#"(print (s/part "=" 2 $1))"#)
        );
    }

    #[test]
    fn parses_conditional_and_string_values() {
        assert!(
            parse(r#"(print (if (s/= (s/lower $1) "alice") (s/upper $2) (default $3 "unknown")))"#)
                .is_ok()
        );
    }

    #[test]
    fn parses_binary_arithmetic_values() {
        assert!(parse("(print (+ $1 2.0) (- $1 $2) (* 3 4) (/ 10 2))").is_ok());
        assert!(parse("(print (+ (* $1 2) (/ $2 4)))").is_ok());
    }

    #[test]
    fn parses_fixed_number_formatting() {
        assert!(parse("(print (n/fixed 2 $1))").is_ok());
        assert!(parse("(print (str \"$\" (n/fixed (+ 1 1) (* $1 $2))))").is_ok());
    }

    #[test]
    fn parses_url_component_extraction() {
        assert!(parse("(print (url/scheme $1) (url/host $1) (url/port $1))").is_ok());
        assert!(parse("(print (url/path $1) (url/query $1) (url/fragment $1))").is_ok());
        assert!(parse("(print (-> $1 (url/path) (s/upper)))").is_ok());
    }

    #[test]
    fn parses_url_component_encoding() {
        assert!(parse("(print (url/encode $1) (url/decode $2))").is_ok());
        assert!(parse("(print (-> $1 (url/encode) (url/decode)))").is_ok());
    }

    #[test]
    fn parses_typed_values_and_predicates() {
        assert!(parse(r#"(filter (dt/>= $1 "2026-08-01T00:00:00Z"))"#).is_ok());
        assert!(parse(r#"(filter (s/= $1 "Alice"))"#).is_ok());
        assert!(parse(r#"(filter (ip/private? $1))"#).is_ok());
        assert!(parse(r#"(filter (ip/loopback? $1))"#).is_ok());
        assert!(parse(r#"(filter (ip/link-local? $1))"#).is_ok());
        assert!(parse(r#"(filter (ip/multicast? $1))"#).is_ok());
        assert!(parse(r#"(filter (semver/>= $1 "2.4.0"))"#).is_ok());
        assert!(parse(r#"(filter (cidr/contains? "10.0.0.0/8" $1))"#).is_ok());
        assert!(parse(r#"(print (dt/add (dt/unix $1) (du/m 2)))"#).is_ok());
        assert!(parse(r#"(print (du/ms 250) (du/d -1.5))"#).is_ok());
        assert!(parse(r#"(print (dt/fmt "%Y-%m-%d" $1))"#).is_ok());
        assert!(parse(r#"(print (dt/floor-m (dt/now)))"#).is_ok());
    }

    #[test]
    fn threading_expands_to_existing_value_ast() {
        assert_eq!(
            parse(r#"(print (-> $1 (dt/add (du/s 10))))"#),
            parse(r#"(print (dt/add $1 (du/s 10)))"#)
        );
        assert_eq!(
            parse(r#"(print (->> $1 (dt/fmt "%Y/%m/%d") (str "date: ")))"#),
            parse(r#"(print (str "date: " (dt/fmt "%Y/%m/%d" $1)))"#)
        );
        assert!(parse("(print (-> $1 (s/lower) (s/count)))").is_ok());
        assert_eq!(
            parse(r#"(print (-> $1 (dt/floor-h)))"#),
            parse(r#"(print (dt/floor-h $1))"#)
        );
    }

    #[test]
    fn rejects_invalid_programs() {
        assert_eq!(parse("(print $x)"), Err(ParseError::InvalidField));
        assert_eq!(parse("print $1"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(filter (> $1))"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(f)"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(f (> $1 0) $2)"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(filter (reg $1))"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(filter (not))"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(filter (and))"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(filter (or))"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(print (fmt $1))"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(print (s/join))"), Err(ParseError::InvalidSyntax));
        for program in [
            "(print (+))",
            "(print (+ $1))",
            "(print (+ $1 $2 $3))",
            "(print (-))",
            "(print (* $1))",
            "(print (/ $1 $2 $3))",
            "(print (n/fixed))",
            "(print (n/fixed 2))",
            "(print (n/fixed 2 $1 $2))",
            "(print (url/host))",
            "(print (url/path $1 $2))",
            "(print (url/domain $1))",
            "(print (url/encode))",
            "(print (url/decode $1 $2))",
            "(print (s/part))",
            r#"(print (s/part ":"))"#,
            r#"(print (s/part ":" 1))"#,
            r#"(print (s/part ":" 1 $1 $2))"#,
        ] {
            assert_eq!(parse(program), Err(ParseError::InvalidSyntax), "{program}");
        }
        assert_eq!(parse("(print (s/count))"), Err(ParseError::InvalidSyntax));
        assert_eq!(
            parse("(print (s/count $1 $2))"),
            Err(ParseError::InvalidSyntax)
        );
        assert_eq!(parse("(print (s/escape))"), Err(ParseError::InvalidSyntax));
        assert_eq!(
            parse("(print (s/escape $1 $2))"),
            Err(ParseError::InvalidSyntax)
        );
        for program in [
            "(print (if))",
            "(print (if (= $1 $2) $1))",
            "(print (if (= $1 $2) $1 $2 $3))",
            "(print (s/lower))",
            "(print (s/lower $1 $2))",
            "(print (s/upper))",
            "(print (s/upper $1 $2))",
            "(print (default))",
            "(print (default $1))",
            "(print (default $1 $2 $3))",
            "(print (dt/unix))",
            "(print (dt/unix $1 $2))",
            "(print (dt/fmt $1))",
            "(print (du/s))",
            "(print (du/ms))",
            "(print (du/ms $1 $2))",
            "(print (du/d))",
            "(print (du/d $1 $2))",
            "(print (dur/s 1))",
            "(print (dt/now $1))",
            "(print (dt/floor-s))",
            "(print (dt/floor-m $1 $2))",
            r#"(print (join "," $1))"#,
            "(print (count $1))",
            "(print (escape $1))",
            "(print (lower $1))",
            "(print (upper $1))",
            "(print (dt/add $1))",
            "(print (-> $1 (unknown)))",
            "(filter (ip/loopback?))",
            "(filter (ip/link-local? $1 $2))",
            "(filter (ip/multicast?))",
            "(filter (semver/> $1))",
            "(filter (semver/= $1 $2 $3))",
        ] {
            assert_eq!(parse(program), Err(ParseError::InvalidSyntax), "{program}");
        }
        assert_eq!(
            parse("(print \"unfinished)"),
            Err(ParseError::UnterminatedString)
        );
        assert_eq!(
            parse("(filter (~ /unfinished))"),
            Err(ParseError::UnterminatedRegex)
        );
    }
}
