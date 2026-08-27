use crate::lexer::{Token, tokenize};

#[derive(Debug, PartialEq)]
pub enum ParseError {
    InvalidSyntax,
    InvalidField,
    UnterminatedString,
    UnterminatedRegex,
}

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
        while self.position < self.tokens.len() {
            expressions.push(self.parse_expression()?);
        }
        Ok(expressions)
    }

    fn parse_expression(&mut self) -> Result<SExpr, ParseError> {
        match self.next() {
            Some(Token::Atom(symbol)) => Ok(SExpr::Atom(Atom::Symbol(symbol))),
            Some(Token::String(value)) => Ok(SExpr::Atom(Atom::String(value))),
            Some(Token::Regex(pattern)) => Ok(SExpr::Atom(Atom::Regex(pattern))),
            Some(Token::LeftParen) => {
                let mut expressions = Vec::new();
                loop {
                    match self.tokens.get(self.position) {
                        Some(Token::RightParen) => {
                            self.position += 1;
                            return Ok(SExpr::List(expressions));
                        }
                        Some(_) => expressions.push(self.parse_expression()?),
                        None => return Err(ParseError::InvalidSyntax),
                    }
                }
            }
            Some(Token::RightParen) | None => Err(ParseError::InvalidSyntax),
        }
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.position).cloned();
        self.position += usize::from(token.is_some());
        token
    }
}

pub(crate) fn parse(program: &str) -> Result<Vec<SExpr>, ParseError> {
    Parser::new(tokenize(program)?).parse_program()
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
        assert_eq!(parse("(print $1"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("print $1)"), Err(ParseError::InvalidSyntax));
    }
}
