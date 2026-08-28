use crate::ast::*;

use super::*;

define_callable!(
    Regex,
    CallableDefinition {
        name: "reg",
        aliases: &["~"],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("pattern", Regex, Required)] => Some(ValueType::Boolean)),
            sig!([p!("value", Value, Required), p!("pattern", Regex, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let args = arguments.0;
        let (target, regex) = match args.len() {
            1 => {
                let [regex] = args.try_into().expect("length was checked");
                (Value::Field(0), expect_regex(regex)?)
            }
            2 => {
                let [target, regex] = args.try_into().expect("length was checked");
                (expect_value(target)?, expect_regex(regex)?)
            }
            _ => return Err(ParseError::InvalidSyntax),
        };
        value(Value::Predicate(Box::new(Predicate::Regex {
            target,
            regex,
        })))
    },
    RegularExpression,
    "match a regular expression",
    ["Regex literals preserve backslashes; quoted patterns require string escaping."],
    [(None, "(reg /ERROR|WARN/)"), (None, "(reg $1 /^api-/)")]
);

define_callable!(
    Replace,
    CallableDefinition {
        name: "re/replace",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("value", Value, Required), p!("pattern", Regex, Required), p!("replacement", Value, Required)] => Some(ValueType::String))
        ]
    },
    |_context, arguments| {
        let [value_arg, regex, replacement] = arguments
            .0
            .try_into()
            .map_err(|_| ParseError::InvalidSyntax)?;
        value(Value::RegexReplace {
            mode: ReplaceMode::First,
            value: Box::new(expect_value(value_arg)?),
            regex: expect_regex(regex)?,
            replacement: Box::new(expect_value(replacement)?),
        })
    },
    RegularExpression,
    "replace the first regular-expression match",
    [],
    [(None, "(re/replace $1 /[0-9]+/ \"N\")")]
);

define_callable!(
    ReplaceAll,
    CallableDefinition {
        name: "re/replace-all",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("value", Value, Required), p!("pattern", Regex, Required), p!("replacement", Value, Required)] => Some(ValueType::String))
        ]
    },
    |_context, arguments| {
        let [value_arg, regex, replacement] = arguments
            .0
            .try_into()
            .map_err(|_| ParseError::InvalidSyntax)?;
        value(Value::RegexReplace {
            mode: ReplaceMode::All,
            value: Box::new(expect_value(value_arg)?),
            regex: expect_regex(regex)?,
            replacement: Box::new(expect_value(replacement)?),
        })
    },
    RegularExpression,
    "replace all regular-expression matches",
    [],
    [(None, "(re/replace-all $1 /[0-9]+/ \"N\")")]
);

define_callable!(
    Part,
    CallableDefinition {
        name: "re/part",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("value", Value, Required), p!("pattern", Regex, Required), p!("position", Number, Required)] => Some(ValueType::String))
        ]
    },
    |_context, arguments| {
        let [value_arg, regex, position] = arguments
            .0
            .try_into()
            .map_err(|_| ParseError::InvalidSyntax)?;
        value(Value::RegexPart {
            value: Box::new(expect_value(value_arg)?),
            regex: expect_regex(regex)?,
            position: Box::new(expect_value(position)?),
        })
    },
    RegularExpression,
    "take a 1-based regular-expression-delimited part",
    ["POSITION must be a positive whole number. Missing parts are empty strings."],
    [(None, "(re/part $1 /[,:]+/ 2)")]
);
