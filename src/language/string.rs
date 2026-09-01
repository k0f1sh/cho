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
            sig!([p!("separator", Value, Required, "SEPARATOR"), p!("value", Value, ZeroOrMore)] => Some(ValueType::String))
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
            sig!([p!("value", Value, Required), p!("from", Value, Required, "FROM"), p!("to", Value, Required, "TO")] => Some(ValueType::String))
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
            sig!([p!("value", Value, Required), p!("from", Value, Required, "FROM"), p!("to", Value, Required, "TO")] => Some(ValueType::String))
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
            sig!([p!("value", Value, Required), p!("delimiter", Value, Required, "DELIMITER"), p!("position", Number, Required, "POSITION")] => Some(ValueType::String))
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

macro_rules! define_boundary {
    ($type:ident, $name:literal, $kind:ident, $summary:literal, $example:literal) => {
        define_callable!(
            $type,
            CallableDefinition {
                name: $name,
                aliases: &[],
                kind: CallableKind::Function,
                signatures: &[
                    sig!([p!("value", Value, Required), p!("delimiter", Value, Required, "DELIMITER")] => Some(ValueType::String))
                ]
            },
            |_context, arguments| {
                let [value_arg, delimiter] = value_array(arguments)?;
                value(Value::Boundary {
                    kind: StringBoundary::$kind,
                    value: Box::new(value_arg),
                    delimiter: Box::new(delimiter),
                })
            },
            String,
            $summary,
            ["DELIMITER must not be empty."],
            [(None, $example)]
        );
    };
}

define_boundary!(
    Before,
    "s/before",
    Before,
    "take text before the first literal delimiter",
    "(s/before $1 \"=\")"
);
define_boundary!(
    After,
    "s/after",
    After,
    "take text after the first literal delimiter",
    "(s/after $1 \"=\")"
);

define_callable!(
    Slice,
    CallableDefinition {
        name: "s/slice",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("value", Value, Required), p!("start", Number, Required, "START"), p!("length", Number, Optional, "LENGTH")] => Some(ValueType::String))
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

macro_rules! define_padding {
    ($type:ident, $name:literal, $kind:ident, $summary:literal, $example:literal) => {
        define_callable!(
            $type,
            CallableDefinition {
                name: $name,
                aliases: &[],
                kind: CallableKind::Function,
                signatures: &[
                    sig!([p!("value", Value, Required), p!("width", Number, Required, "WIDTH"), p!("fill", Value, Optional, "FILL")] => Some(ValueType::String))
                ]
            },
            |_context, arguments| {
                let mut args = values(arguments)?.into_iter();
                let value_arg = args.next().expect("signature requires value");
                let width = args.next().expect("signature requires width");
                value(Value::Pad {
                    kind: StringPadding::$kind,
                    value: Box::new(value_arg),
                    width: Box::new(width),
                    fill: args.next().map(Box::new),
                })
            },
            String,
            $summary,
            ["WIDTH counts Unicode characters and must be a non-negative whole number. FILL defaults to one space and must be exactly one Unicode character."],
            [(None, $example)]
        );
    };
}

define_padding!(
    LeftPad,
    "s/lpad",
    Left,
    "pad the left side to a Unicode character width",
    "(s/lpad $1 5 \"0\")"
);
define_padding!(
    RightPad,
    "s/rpad",
    Right,
    "pad the right side to a Unicode character width",
    "(s/rpad $1 10)"
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
    Empty,
    CallableDefinition {
        name: "s/empty?",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Value, Required)] => Some(ValueType::Boolean))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::StringEmpty(Box::new(value_arg)))
    },
    String,
    "test whether a string is empty",
    [],
    [(None, "(s/empty? $1)")]
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
    Unquote,
    CallableDefinition {
        name: "s/unquote",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Value, Required)] => Some(ValueType::String))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::Unquote(Box::new(value_arg)))
    },
    String,
    "remove matching quotes and decode backslash escapes",
    [
        "Unquoted values are unchanged. Quoted values accept \\\\, \\n, \\r, \\t, and an escaped enclosing quote."
    ],
    [(None, r#"(s/unquote $1)"#)]
);

define_callable!(
    ShellQuote,
    CallableDefinition {
        name: "shq",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Value, Required)] => Some(ValueType::String))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::ShellQuote(Box::new(value_arg)))
    },
    String,
    "stringify and quote as one shell-safe argument",
    ["Unlike dq, prevents shell expansion of $, command substitutions, and other syntax."],
    [(None, r#"(shq "it's good")"#)]
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
        signatures: &[
            sig!([p!("value", Value, Required)] => Some(ValueType::String)),
            sig!([p!("value", Value, Required), p!("prefix", Value, Required, "PREFIX"), p!("suffix", Value, Required, "SUFFIX")] => Some(ValueType::String))
        ]
    },
    |_context, arguments| {
        let mut arguments = values(arguments)?.into_iter();
        let value_arg = arguments.next().expect("signature requires value");
        match (arguments.next(), arguments.next()) {
            (None, None) => value(Value::Trim {
                kind: StringTrim::Both,
                value: Box::new(value_arg),
            }),
            (Some(prefix), Some(suffix)) => value(Value::TrimAffixes {
                value: Box::new(value_arg),
                prefix: Some(Box::new(prefix)),
                suffix: Some(Box::new(suffix)),
            }),
            _ => Err(ParseError::InvalidSyntax),
        }
    },
    String,
    "trim whitespace or exact affixes",
    ["Exact affixes are removed at most once; empty or absent affixes leave that end unchanged."],
    [
        (
            Some("remove Unicode whitespace from both ends"),
            "(s/trim $1)"
        ),
        (
            Some("remove one exact prefix and suffix"),
            "(s/trim $1 \"[\" \"]\")"
        )
    ]
);

define_callable!(
    LeftTrim,
    CallableDefinition {
        name: "s/ltrim",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("value", Value, Required)] => Some(ValueType::String)),
            sig!([p!("value", Value, Required), p!("prefix", Value, Required, "PREFIX")] => Some(ValueType::String))
        ]
    },
    |_context, arguments| {
        let mut arguments = values(arguments)?.into_iter();
        let value_arg = arguments.next().expect("signature requires value");
        match arguments.next() {
            None => value(Value::Trim {
                kind: StringTrim::Left,
                value: Box::new(value_arg),
            }),
            Some(prefix) => value(Value::TrimAffixes {
                value: Box::new(value_arg),
                prefix: Some(Box::new(prefix)),
                suffix: None,
            }),
        }
    },
    String,
    "trim left whitespace or an exact prefix",
    [
        "An exact prefix is removed at most once; an empty or absent prefix leaves the value unchanged."
    ],
    [
        (
            Some("remove Unicode whitespace from the left"),
            "(s/ltrim $1)"
        ),
        (Some("remove one exact prefix"), "(s/ltrim $1 \"v\")")
    ]
);

define_callable!(
    RightTrim,
    CallableDefinition {
        name: "s/rtrim",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("value", Value, Required)] => Some(ValueType::String)),
            sig!([p!("value", Value, Required), p!("suffix", Value, Required, "SUFFIX")] => Some(ValueType::String))
        ]
    },
    |_context, arguments| {
        let mut arguments = values(arguments)?.into_iter();
        let value_arg = arguments.next().expect("signature requires value");
        match arguments.next() {
            None => value(Value::Trim {
                kind: StringTrim::Right,
                value: Box::new(value_arg),
            }),
            Some(suffix) => value(Value::TrimAffixes {
                value: Box::new(value_arg),
                prefix: None,
                suffix: Some(Box::new(suffix)),
            }),
        }
    },
    String,
    "trim right whitespace or an exact suffix",
    [
        "An exact suffix is removed at most once; an empty or absent suffix leaves the value unchanged."
    ],
    [
        (
            Some("remove Unicode whitespace from the right"),
            "(s/rtrim $1)"
        ),
        (Some("remove one exact suffix"), "(s/rtrim $1 \"%\")")
    ]
);

define_callable!(
    StartsWith,
    CallableDefinition {
        name: "s/starts-with?",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("value", Value, Required), p!("prefix", Value, Required, "PREFIX")] => Some(ValueType::Boolean))
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
            sig!([p!("value", Value, Required), p!("suffix", Value, Required, "SUFFIX")] => Some(ValueType::Boolean))
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
            sig!([p!("value", Value, Required), p!("needle", Value, Required, "NEEDLE")] => Some(ValueType::Boolean))
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
            sig!([p!("left", Value, Required), p!("right", Value, Required)] => Some(ValueType::Boolean))
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
            sig!([p!("left", Value, Required), p!("right", Value, Required)] => Some(ValueType::Boolean))
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
            sig!([p!("left", Value, Required), p!("right", Value, Required)] => Some(ValueType::Boolean))
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
            sig!([p!("left", Value, Required), p!("right", Value, Required)] => Some(ValueType::Boolean))
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
            sig!([p!("left", Value, Required), p!("right", Value, Required)] => Some(ValueType::Boolean))
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
            sig!([p!("left", Value, Required), p!("right", Value, Required)] => Some(ValueType::Boolean))
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
