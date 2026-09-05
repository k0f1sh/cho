use crate::lexer::{Token, tokenize};
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    InvalidSyntax,
    AutomaticValueWithPrint,
    MultipleAutomaticValues,
    FilterAfterAutomaticValue,
    NonFiniteNumberLiteral(String),
    UnknownFunction(String),
    InvalidField,
    UnterminatedString,
    UnsupportedStringEscape(char),
    UnterminatedRegex,
    MissingClosingParenthesis,
    UnexpectedClosingParenthesis,
    InvalidArity {
        expression: String,
        expected: String,
        actual: usize,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidSyntax => "invalid syntax",
            Self::AutomaticValueWithPrint => {
                "cannot combine an automatic top-level value with print"
            }
            Self::MultipleAutomaticValues => "only one automatic top-level value is allowed",
            Self::FilterAfterAutomaticValue => {
                "an automatic top-level value must follow all filters"
            }
            Self::NonFiniteNumberLiteral(literal) => {
                return write!(formatter, "non-finite number literal: {literal}");
            }
            Self::UnknownFunction(function) => {
                return write!(formatter, "no such function: {function}");
            }
            Self::InvalidField => "invalid field reference",
            Self::UnterminatedString => "unterminated string literal",
            Self::UnsupportedStringEscape(character) => {
                return write!(
                    formatter,
                    "unsupported escape in string literal: \\{}",
                    character.escape_default()
                );
            }
            Self::UnterminatedRegex => "unterminated regex literal",
            Self::MissingClosingParenthesis => "missing closing parenthesis",
            Self::UnexpectedClosingParenthesis => "unexpected closing parenthesis",
            Self::InvalidArity {
                expression,
                expected,
                actual,
            } => {
                return write!(
                    formatter,
                    "{expression}: expected {expected}, but got {actual}"
                );
            }
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ParseError {}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SExpr {
    List(Vec<SExpr>),
    Atom(Atom),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Atom {
    Symbol(String),
    String(String),
    Regex(String),
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

    fn parse_program(&mut self) -> Result<Vec<SExpr>, ParseError> {
        let mut expressions = Vec::new();
        let mut lists: Vec<Vec<SExpr>> = Vec::new();
        while self.position < self.tokens.len() {
            match self.next() {
                Some(Token::Atom(symbol)) => push_expression(
                    &mut expressions,
                    &mut lists,
                    SExpr::Atom(Atom::Symbol(symbol)),
                ),
                Some(Token::String(value)) => push_expression(
                    &mut expressions,
                    &mut lists,
                    SExpr::Atom(Atom::String(value)),
                ),
                Some(Token::Regex(pattern)) => push_expression(
                    &mut expressions,
                    &mut lists,
                    SExpr::Atom(Atom::Regex(pattern)),
                ),
                Some(Token::LeftParen) => lists.push(Vec::new()),
                Some(Token::RightParen) => {
                    let items = lists
                        .pop()
                        .ok_or(ParseError::UnexpectedClosingParenthesis)?;
                    push_expression(&mut expressions, &mut lists, SExpr::List(items));
                }
                None => unreachable!("the loop checks that a token remains"),
            }
        }
        if lists.is_empty() {
            Ok(expressions)
        } else {
            Err(ParseError::MissingClosingParenthesis)
        }
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.position).cloned();
        self.position += usize::from(token.is_some());
        token
    }
}

fn push_expression(expressions: &mut Vec<SExpr>, lists: &mut [Vec<SExpr>], expression: SExpr) {
    match lists.last_mut() {
        Some(list) => list.push(expression),
        None => expressions.push(expression),
    }
}

pub(crate) fn parse(program: &str) -> Result<Vec<SExpr>, ParseError> {
    let tokens = tokenize(program)?;
    stacker::grow(32 * 1024 * 1024, || Parser::new(tokens).parse_program())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_lists_and_atoms_without_language_knowledge() {
        assert_eq!(
            parse(r#"(unknown $1 (anything 42 "text" /a+/) ())"#),
            Ok(vec![SExpr::List(vec![
                SExpr::Atom(Atom::Symbol("unknown".into())),
                SExpr::Atom(Atom::Symbol("$1".into())),
                SExpr::List(vec![
                    SExpr::Atom(Atom::Symbol("anything".into())),
                    SExpr::Atom(Atom::Symbol("42".into())),
                    SExpr::Atom(Atom::String("text".into())),
                    SExpr::Atom(Atom::Regex("a+".into())),
                ]),
                SExpr::List(vec![]),
            ])])
        );
    }

    #[test]
    fn parses_multiple_top_level_expressions() {
        assert_eq!(
            parse("first (second)"),
            Ok(vec![
                SExpr::Atom(Atom::Symbol("first".into())),
                SExpr::List(vec![SExpr::Atom(Atom::Symbol("second".into()))]),
            ])
        );
    }

    #[test]
    fn rejects_unbalanced_parentheses() {
        assert_eq!(
            parse("(print $1"),
            Err(ParseError::MissingClosingParenthesis)
        );
        assert_eq!(
            parse("print $1)"),
            Err(ParseError::UnexpectedClosingParenthesis)
        );
    }

    #[test]
    fn parses_deeply_nested_expressions() {
        let program = format!("{}1{}", "(".repeat(20_000), ")".repeat(20_000));
        let expressions = parse(&program).unwrap();
        stacker::grow(32 * 1024 * 1024, || drop(expressions));
    }

    #[test]
    fn describes_parse_errors() {
        assert_eq!(ParseError::InvalidSyntax.to_string(), "invalid syntax");
        assert_eq!(
            ParseError::UnknownFunction("ip/v6".to_owned()).to_string(),
            "no such function: ip/v6"
        );
        assert_eq!(
            ParseError::MissingClosingParenthesis.to_string(),
            "missing closing parenthesis"
        );
    }
}
