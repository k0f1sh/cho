use crate::parser::ParseError;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Token {
    LeftParen,
    RightParen,
    Atom(String),
    String(String),
    Regex(String),
}

pub(crate) fn tokenize(program: &str) -> Result<Vec<Token>, ParseError> {
    let mut characters = program.chars().peekable();
    let mut tokens = Vec::new();

    while let Some(character) = characters.next() {
        match character {
            character if character.is_whitespace() => {}
            '(' => tokens.push(Token::LeftParen),
            ')' => tokens.push(Token::RightParen),
            '"' => tokens.push(Token::String(read_string(&mut characters)?)),
            '/' => tokens.push(Token::Regex(read_regex(&mut characters)?)),
            first => {
                let mut atom = String::from(first);
                while let Some(character) = characters.peek() {
                    if character.is_whitespace() || matches!(character, '(' | ')' | '"') {
                        break;
                    }
                    atom.push(characters.next().expect("peeked character must exist"));
                }
                tokens.push(Token::Atom(atom));
            }
        }
    }

    Ok(tokens)
}

fn read_regex<I>(characters: &mut I) -> Result<String, ParseError>
where
    I: Iterator<Item = char>,
{
    let mut pattern = String::new();
    while let Some(character) = characters.next() {
        match character {
            '/' => return Ok(pattern),
            '\\' => {
                let escaped = characters.next().ok_or(ParseError::UnterminatedRegex)?;
                if escaped != '/' {
                    pattern.push('\\');
                }
                pattern.push(escaped);
            }
            other => pattern.push(other),
        }
    }
    Err(ParseError::UnterminatedRegex)
}

fn read_string<I>(characters: &mut I) -> Result<String, ParseError>
where
    I: Iterator<Item = char>,
{
    let mut value = String::new();
    while let Some(character) = characters.next() {
        match character {
            '"' => return Ok(value),
            '\\' => {
                let escaped = characters.next().ok_or(ParseError::UnterminatedString)?;
                value.push(match escaped {
                    'n' => '\n',
                    't' => '\t',
                    other => other,
                });
            }
            other => value.push(other),
        }
    }
    Err(ParseError::UnterminatedString)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_parentheses_atoms_and_strings() {
        assert_eq!(
            tokenize(r#"(print $1 "hello world")"#),
            Ok(vec![
                Token::LeftParen,
                Token::Atom("print".into()),
                Token::Atom("$1".into()),
                Token::String("hello world".into()),
                Token::RightParen,
            ])
        );
    }

    #[test]
    fn handles_string_escapes() {
        assert_eq!(
            tokenize(r#""a\tb\n\"c""#),
            Ok(vec![Token::String("a\tb\n\"c".into())])
        );
    }

    #[test]
    fn rejects_an_unterminated_string() {
        assert_eq!(tokenize(r#""oops"#), Err(ParseError::UnterminatedString));
    }

    #[test]
    fn tokenizes_regex_literals_without_consuming_regex_escapes() {
        assert_eq!(
            tokenize(r#"/^\d+\/path$/"#),
            Ok(vec![Token::Regex(r#"^\d+/path$"#.into())])
        );
    }

    #[test]
    fn rejects_an_unterminated_regex_literal() {
        assert_eq!(tokenize(r#"/^oops"#), Err(ParseError::UnterminatedRegex));
    }
}
