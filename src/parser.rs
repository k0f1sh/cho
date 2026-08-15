use crate::ast::{ComparisonOperator, Expr, Predicate, Program, Value};
use crate::lexer::{Token, tokenize};

#[derive(Debug, PartialEq)]
pub enum ParseError {
    InvalidSyntax,
    InvalidField,
    UnterminatedString,
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
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
        Ok(Program { expressions })
    }

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        if self.next() != Some(Token::LeftParen) {
            return Err(ParseError::InvalidSyntax);
        }
        match self.next() {
            Some(Token::Atom(operator)) if operator == "print" => {
                Ok(Expr::Print(self.parse_values_until_right_paren()?))
            }
            Some(Token::Atom(operator)) if operator == "filter" => {
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
        if operator == Some(Token::Atom("reg".into())) {
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
        let operator = match operator {
            Some(Token::Atom(value)) if value == ">" => ComparisonOperator::GreaterThan,
            Some(Token::Atom(value)) if value == ">=" => ComparisonOperator::GreaterThanOrEqual,
            Some(Token::Atom(value)) if value == "<" => ComparisonOperator::LessThan,
            Some(Token::Atom(value)) if value == "<=" => ComparisonOperator::LessThanOrEqual,
            Some(Token::Atom(value)) if value == "=" => ComparisonOperator::Equal,
            Some(Token::Atom(value)) if value == "!=" => ComparisonOperator::NotEqual,
            _ => return Err(ParseError::InvalidSyntax),
        };
        let left = self.parse_value()?;
        let right = self.parse_value()?;
        if self.next() != Some(Token::RightParen) {
            return Err(ParseError::InvalidSyntax);
        }
        Ok(Predicate::Compare {
            operator,
            left,
            right,
        })
    }

    fn parse_regex_predicate(&mut self) -> Result<Predicate, ParseError> {
        let first = self.parse_value()?;
        let (target, pattern) = if self.tokens.get(self.position) == Some(&Token::RightParen) {
            (Value::Field(0), first)
        } else {
            (first, self.parse_value()?)
        };
        if self.next() != Some(Token::RightParen) {
            return Err(ParseError::InvalidSyntax);
        }
        let Value::String(pattern) = pattern else {
            return Err(ParseError::InvalidSyntax);
        };
        Ok(Predicate::Regex { target, pattern })
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
                Some(Token::Atom(operator)) if operator == "str" => {
                    Ok(Value::Concat(self.parse_values_until_right_paren()?))
                }
                Some(Token::Atom(operator)) if operator == "count" => {
                    let value = self.parse_value()?;
                    if self.next() != Some(Token::RightParen) {
                        return Err(ParseError::InvalidSyntax);
                    }
                    Ok(Value::Count(Box::new(value)))
                }
                _ => Err(ParseError::InvalidSyntax),
            },
            _ => Err(ParseError::InvalidSyntax),
        }
    }
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
            parse(r#"(filter (> (count $1) 3)) (print (str NR ":" $1))"#),
            Ok(Program {
                expressions: vec![
                    Expr::Filter(Predicate::Compare {
                        operator: ComparisonOperator::GreaterThan,
                        left: Value::Count(Box::new(Value::Field(1))),
                        right: Value::Number(3.0),
                    }),
                    Expr::Print(vec![Value::Concat(vec![
                        Value::RecordNumber,
                        Value::String(":".into()),
                        Value::Field(1),
                    ])]),
                ]
            })
        );
    }

    #[test]
    fn parses_regex_filters() {
        assert!(parse(r#"(filter (reg "error"))"#).is_ok());
        assert!(parse(r#"(filter (reg $1 "^[A-Z]"))"#).is_ok());
    }

    #[test]
    fn parses_boolean_predicates() {
        assert!(
            parse(r#"(filter (and (not (reg "debug")) (or (= $1 "info") (= $1 "warn"))))"#).is_ok()
        );
    }

    #[test]
    fn rejects_invalid_programs() {
        assert_eq!(parse("(print $x)"), Err(ParseError::InvalidField));
        assert_eq!(parse("print $1"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(filter (> $1))"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(filter (reg $1))"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(filter (not))"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(filter (and))"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(filter (or))"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(print (fmt $1))"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(print (count))"), Err(ParseError::InvalidSyntax));
        assert_eq!(
            parse("(print (count $1 $2))"),
            Err(ParseError::InvalidSyntax)
        );
        assert_eq!(
            parse("(print \"unfinished)"),
            Err(ParseError::UnterminatedString)
        );
    }
}
