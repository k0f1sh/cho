use crate::ast::*;

use super::*;

define_callable!(
    Not,
    CallableDefinition {
        name: "not",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Boolean, Required)] => Some(ValueType::Boolean))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::Not(Box::new(value_arg)))
    },
    Boolean,
    "negate",
    [],
    [(None, "(not true)")]
);

define_callable!(
    If,
    CallableDefinition {
        name: "if",
        aliases: &[],
        kind: CallableKind::SpecialForm,
        signatures: &[
            sig!([p!("condition", Boolean, Required), p!("then", Value, Required), p!("else", Value, Required)] => Some(ValueType::Value))
        ]
    },
    |_context, arguments| {
        let [condition, then_value, else_value] = value_array(arguments)?;
        value(Value::If {
            condition: Box::new(condition),
            then_value: Box::new(then_value),
            else_value: Box::new(else_value),
        })
    },
    SpecialForm,
    "select one value lazily",
    [],
    [(None, "(if (> $1 0) \"positive\" \"other\")")]
);

define_callable!(
    Default,
    CallableDefinition {
        name: "default",
        aliases: &[],
        kind: CallableKind::SpecialForm,
        signatures: &[
            sig!([p!("value", Value, Required), p!("fallback", Value, Required, "FALLBACK")] => Some(ValueType::Value))
        ]
    },
    |_context, arguments| {
        let [value_arg, fallback] = value_array(arguments)?;
        value(Value::Default {
            value: Box::new(value_arg),
            fallback: Box::new(fallback),
        })
    },
    SpecialForm,
    "use a fallback when the value is empty or errors",
    [],
    [(None, "(default $3 \"unknown\")")]
);

define_callable!(
    And,
    CallableDefinition {
        name: "and",
        aliases: &[],
        kind: CallableKind::SpecialForm,
        signatures: &[sig!([p!("value", Boolean, OneOrMore)] => Some(ValueType::Boolean))]
    },
    |_context, arguments| { value(Value::And(values(arguments)?)) },
    SpecialForm,
    "stop at the first false value",
    [],
    [(None, "(and (> $1 0) (< $1 10))")]
);

define_callable!(
    Or,
    CallableDefinition {
        name: "or",
        aliases: &[],
        kind: CallableKind::SpecialForm,
        signatures: &[sig!([p!("value", Boolean, OneOrMore)] => Some(ValueType::Boolean))]
    },
    |_context, arguments| { value(Value::Or(values(arguments)?)) },
    SpecialForm,
    "stop at the first true value",
    [],
    [(None, "(or (= $1 0) (= $1 1))")]
);
