use crate::ast::*;

use super::*;

define_callable!(
    Concat,
    CallableDefinition {
        name: "str",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Value, ZeroOrMore)] => Some(ValueType::String))]
    },
    |_context, arguments| { value(Value::Concat(values(arguments)?)) },
    String,
    "concatenate values",
    [],
    [(None, "(str $1 \":\" $2)")]
);

define_callable!(
    Join,
    CallableDefinition {
        name: "s/join",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("separator", Value, Required), p!("value", Value, ZeroOrMore)] => Some(ValueType::String))
        ]
    },
    |_context, arguments| {
        let mut args = values(arguments)?;
        let separator = args.remove(0);
        value(Value::Join {
            separator: Box::new(separator),
            values: args,
        })
    },
    String,
    "join values",
    [],
    [(None, "(s/join \",\" $1 $2)")]
);

define_callable!(
    Replace,
    CallableDefinition {
        name: "s/replace",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("value", Value, Required), p!("from", Value, Required), p!("to", Value, Required)] => Some(ValueType::String))
        ]
    },
    |_context, arguments| {
        let [value_arg, from, to] = value_array(arguments)?;
        value(Value::Replace {
            mode: ReplaceMode::First,
            value: Box::new(value_arg),
            from: Box::new(from),
            to: Box::new(to),
        })
    },
    String,
    "replace the first literal match",
    ["An empty FROM inserts TO at the start."],
    [(None, "(s/replace $1 \"-\" \"_\")")]
);

define_callable!(
    ReplaceAll,
    CallableDefinition {
        name: "s/replace-all",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("value", Value, Required), p!("from", Value, Required), p!("to", Value, Required)] => Some(ValueType::String))
        ]
    },
    |_context, arguments| {
        let [value_arg, from, to] = value_array(arguments)?;
        value(Value::Replace {
            mode: ReplaceMode::All,
            value: Box::new(value_arg),
            from: Box::new(from),
            to: Box::new(to),
        })
    },
    String,
    "replace all literal matches",
    ["An empty FROM inserts TO at every character boundary."],
    [(None, "(s/replace-all $1 \"-\" \"_\")")]
);

define_callable!(
    Part,
    CallableDefinition {
        name: "s/part",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("value", Value, Required), p!("delimiter", Value, Required), p!("position", Number, Required)] => Some(ValueType::String))
        ]
    },
    |_context, arguments| {
        let [value_arg, delimiter, position] = value_array(arguments)?;
        value(Value::Part {
            value: Box::new(value_arg),
            delimiter: Box::new(delimiter),
            position: Box::new(position),
        })
    },
    String,
    "take a 1-based literal-delimited part",
    [
        "DELIMITER must not be empty; POSITION must be a positive whole number. Missing parts are empty strings."
    ],
    [(None, "(s/part $1 \":\" 2)")]
);

define_callable!(
    Slice,
    CallableDefinition {
        name: "s/slice",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("value", Value, Required), p!("start", Number, Required), p!("length", Number, Optional)] => Some(ValueType::String))
        ]
    },
    |_context, arguments| {
        let mut args = values(arguments)?.into_iter();
        let value_arg = args.next().expect("signature requires value");
        let start = args.next().expect("signature requires start");
        value(Value::Slice {
            value: Box::new(value_arg),
            start: Box::new(start),
            length: args.next().map(Box::new),
        })
    },
    String,
    "take Unicode characters from a 1-based start",
    ["START must be positive; LENGTH must be a non-negative whole number."],
    [(None, "(s/slice $1 3 5)")]
);

define_callable!(
    Count,
    CallableDefinition {
        name: "s/count",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Value, Required)] => Some(ValueType::Number))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::Count(Box::new(value_arg)))
    },
    String,
    "count Unicode characters",
    [],
    [(None, "(s/count $1)")]
);

define_callable!(
    Escape,
    CallableDefinition {
        name: "s/escape",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Value, Required)] => Some(ValueType::String))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::Escape(Box::new(value_arg)))
    },
    String,
    "escape tabs, newlines, and backslashes",
    [],
    [(None, "(s/escape $1)")]
);

define_callable!(
    DoubleQuote,
    CallableDefinition {
        name: "s/dquote",
        aliases: &["dq"],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Value, Required)] => Some(ValueType::String))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::Quote {
            kind: StringQuote::Double,
            value: Box::new(value_arg),
        })
    },
    String,
    "stringify and wrap in escaped double quotes",
    [],
    [(None, "(s/dquote $1)")]
);

define_callable!(
    SingleQuote,
    CallableDefinition {
        name: "s/squote",
        aliases: &["sq"],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Value, Required)] => Some(ValueType::String))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::Quote {
            kind: StringQuote::Single,
            value: Box::new(value_arg),
        })
    },
    String,
    "stringify and wrap in escaped single quotes",
    [],
    [(None, "(s/squote $1)")]
);

define_callable!(
    Lower,
    CallableDefinition {
        name: "s/lower",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Value, Required)] => Some(ValueType::String))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::Lower(Box::new(value_arg)))
    },
    String,
    "lowercase",
    [],
    [(None, "(s/lower $1)")]
);

define_callable!(
    Upper,
    CallableDefinition {
        name: "s/upper",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Value, Required)] => Some(ValueType::String))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::Upper(Box::new(value_arg)))
    },
    String,
    "uppercase",
    [],
    [(None, "(s/upper $1)")]
);

define_callable!(
    Reverse,
    CallableDefinition {
        name: "s/reverse",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Value, Required)] => Some(ValueType::String))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::Reverse(Box::new(value_arg)))
    },
    String,
    "reverse Unicode characters",
    [],
    [(None, "(s/reverse $1)")]
);

define_callable!(
    Trim,
    CallableDefinition {
        name: "s/trim",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Value, Required)] => Some(ValueType::String))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::Trim {
            kind: StringTrim::Both,
            value: Box::new(value_arg),
        })
    },
    String,
    "remove Unicode whitespace from both ends",
    [],
    [(None, "(s/trim $1)")]
);

define_callable!(
    LeftTrim,
    CallableDefinition {
        name: "s/ltrim",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Value, Required)] => Some(ValueType::String))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::Trim {
            kind: StringTrim::Left,
            value: Box::new(value_arg),
        })
    },
    String,
    "remove Unicode whitespace from the left",
    [],
    [(None, "(s/ltrim $1)")]
);

define_callable!(
    RightTrim,
    CallableDefinition {
        name: "s/rtrim",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Value, Required)] => Some(ValueType::String))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::Trim {
            kind: StringTrim::Right,
            value: Box::new(value_arg),
        })
    },
    String,
    "remove Unicode whitespace from the right",
    [],
    [(None, "(s/rtrim $1)")]
);

define_callable!(
    StartsWith,
    CallableDefinition {
        name: "s/starts-with?",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("string", String, Required), p!("prefix", String, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [value_arg, pattern] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::StringTest {
            kind: StringTest::StartsWith,
            value: value_arg,
            pattern,
        })))
    },
    String,
    "test a prefix",
    [],
    [(None, "(s/starts-with? $1 \"api-\")")]
);

define_callable!(
    EndsWith,
    CallableDefinition {
        name: "s/ends-with?",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("string", String, Required), p!("suffix", String, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [value_arg, pattern] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::StringTest {
            kind: StringTest::EndsWith,
            value: value_arg,
            pattern,
        })))
    },
    String,
    "test a suffix",
    [],
    [(None, "(s/ends-with? $1 \".log\")")]
);

define_callable!(
    Contains,
    CallableDefinition {
        name: "s/contains?",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("string", String, Required), p!("needle", String, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [value_arg, pattern] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::StringTest {
            kind: StringTest::Contains,
            value: value_arg,
            pattern,
        })))
    },
    String,
    "test a substring",
    [],
    [(None, "(s/contains? $1 \"error\")")]
);

define_callable!(
    GreaterThan,
    CallableDefinition {
        name: "s/>",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", String, Required), p!("right", String, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::Compare {
            kind: ComparisonType::String,
            operator: ComparisonOperator::GreaterThan,
            left,
            right,
        })))
    },
    String,
    "greater than",
    [],
    [(None, "(s/> \"b\" \"a\")")]
);

define_callable!(
    GreaterThanOrEqual,
    CallableDefinition {
        name: "s/>=",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", String, Required), p!("right", String, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::Compare {
            kind: ComparisonType::String,
            operator: ComparisonOperator::GreaterThanOrEqual,
            left,
            right,
        })))
    },
    String,
    "greater than or equal",
    [],
    [(None, "(s/>= \"b\" \"a\")")]
);

define_callable!(
    LessThan,
    CallableDefinition {
        name: "s/<",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", String, Required), p!("right", String, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::Compare {
            kind: ComparisonType::String,
            operator: ComparisonOperator::LessThan,
            left,
            right,
        })))
    },
    String,
    "less than",
    [],
    [(None, "(s/< \"b\" \"a\")")]
);

define_callable!(
    LessThanOrEqual,
    CallableDefinition {
        name: "s/<=",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", String, Required), p!("right", String, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::Compare {
            kind: ComparisonType::String,
            operator: ComparisonOperator::LessThanOrEqual,
            left,
            right,
        })))
    },
    String,
    "less than or equal",
    [],
    [(None, "(s/<= \"b\" \"a\")")]
);

define_callable!(
    Equal,
    CallableDefinition {
        name: "s/=",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", String, Required), p!("right", String, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::Compare {
            kind: ComparisonType::String,
            operator: ComparisonOperator::Equal,
            left,
            right,
        })))
    },
    String,
    "equal",
    [],
    [(None, "(s/= \"b\" \"a\")")]
);

define_callable!(
    NotEqual,
    CallableDefinition {
        name: "s/!=",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", String, Required), p!("right", String, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::Compare {
            kind: ComparisonType::String,
            operator: ComparisonOperator::NotEqual,
            left,
            right,
        })))
    },
    String,
    "not equal",
    [],
    [(None, "(s/!= \"b\" \"a\")")]
);
