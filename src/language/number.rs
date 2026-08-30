use crate::ast::*;

use super::*;

define_callable!(
    Add,
    CallableDefinition {
        name: "+",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", Number, Required), p!("right", Number, Required)] => Some(ValueType::Number))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Arithmetic {
            operator: ArithmeticOperator::Add,
            left: Box::new(left),
            right: Box::new(right),
        })
    },
    Number,
    "add",
    [],
    [(None, "(+ $1 2)")]
);

define_callable!(
    Subtract,
    CallableDefinition {
        name: "-",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", Number, Required), p!("right", Number, Required)] => Some(ValueType::Number))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Arithmetic {
            operator: ArithmeticOperator::Subtract,
            left: Box::new(left),
            right: Box::new(right),
        })
    },
    Number,
    "subtract",
    [],
    [(None, "(- $1 2)")]
);

define_callable!(
    Multiply,
    CallableDefinition {
        name: "*",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", Number, Required), p!("right", Number, Required)] => Some(ValueType::Number))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Arithmetic {
            operator: ArithmeticOperator::Multiply,
            left: Box::new(left),
            right: Box::new(right),
        })
    },
    Number,
    "multiply",
    [],
    [(None, "(* $1 2)")]
);

define_callable!(
    Divide,
    CallableDefinition {
        name: "/",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", Number, Required), p!("right", Number, Required)] => Some(ValueType::Number))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Arithmetic {
            operator: ArithmeticOperator::Divide,
            left: Box::new(left),
            right: Box::new(right),
        })
    },
    Number,
    "divide",
    ["Division by zero and non-finite results are errors."],
    [(None, "(/ $1 2)")]
);

define_callable!(
    Remainder,
    CallableDefinition {
        name: "%",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", Number, Required), p!("right", Number, Required)] => Some(ValueType::Number))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Arithmetic {
            operator: ArithmeticOperator::Remainder,
            left: Box::new(left),
            right: Box::new(right),
        })
    },
    Number,
    "remainder",
    ["A zero divisor is an error. The result has the sign of the dividend."],
    [(None, "(% $1 2)")]
);

define_callable!(
    Truncate,
    CallableDefinition {
        name: "n/trunc",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Number, Required)] => Some(ValueType::Number))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::NumberOperation {
            operator: NumberOperator::Truncate,
            value: Box::new(value_arg),
        })
    },
    Number,
    "discard fractional digits toward zero",
    [],
    [(None, "(n/trunc -2.7)")]
);

define_callable!(
    Floor,
    CallableDefinition {
        name: "n/floor",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Number, Required)] => Some(ValueType::Number))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::NumberOperation {
            operator: NumberOperator::Floor,
            value: Box::new(value_arg),
        })
    },
    Number,
    "round down toward negative infinity",
    [],
    [(None, "(n/floor -2.7)")]
);

define_callable!(
    Ceil,
    CallableDefinition {
        name: "n/ceil",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Number, Required)] => Some(ValueType::Number))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::NumberOperation {
            operator: NumberOperator::Ceil,
            value: Box::new(value_arg),
        })
    },
    Number,
    "round up toward positive infinity",
    [],
    [(None, "(n/ceil 2.1)")]
);

define_callable!(
    Round,
    CallableDefinition {
        name: "n/round",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Number, Required)] => Some(ValueType::Number))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::NumberOperation {
            operator: NumberOperator::Round,
            value: Box::new(value_arg),
        })
    },
    Number,
    "round to nearest, halves away from zero",
    [],
    [(None, "(n/round 2.5)")]
);

define_callable!(
    Absolute,
    CallableDefinition {
        name: "n/abs",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Number, Required)] => Some(ValueType::Number))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::NumberOperation {
            operator: NumberOperator::Absolute,
            value: Box::new(value_arg),
        })
    },
    Number,
    "absolute value",
    [],
    [(None, "(n/abs -2.5)")]
);

define_callable!(
    Fixed,
    CallableDefinition {
        name: "n/fixed",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("value", Number, Required), p!("digits", Number, Required)] => Some(ValueType::String))
        ]
    },
    |_context, arguments| {
        let [value_arg, digits] = value_array(arguments)?;
        value(Value::FormatNumberFixed {
            value: Box::new(value_arg),
            digits: Box::new(digits),
        })
    },
    Number,
    "format with 0 to 100 fractional digits",
    ["DIGITS must be a whole number from 0 through 100."],
    [(None, "(n/fixed 12.5 2)")]
);

define_callable!(
    GreaterThan,
    CallableDefinition {
        name: ">",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", Number, Required), p!("right", Number, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::Compare {
            kind: ComparisonType::Number,
            operator: ComparisonOperator::GreaterThan,
            left,
            right,
        })))
    },
    Number,
    "greater than",
    [],
    [(None, "(> 12 3)")]
);

define_callable!(
    GreaterThanOrEqual,
    CallableDefinition {
        name: ">=",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", Number, Required), p!("right", Number, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::Compare {
            kind: ComparisonType::Number,
            operator: ComparisonOperator::GreaterThanOrEqual,
            left,
            right,
        })))
    },
    Number,
    "greater than or equal",
    [],
    [(None, "(>= 12 3)")]
);

define_callable!(
    LessThan,
    CallableDefinition {
        name: "<",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", Number, Required), p!("right", Number, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::Compare {
            kind: ComparisonType::Number,
            operator: ComparisonOperator::LessThan,
            left,
            right,
        })))
    },
    Number,
    "less than",
    [],
    [(None, "(< 12 3)")]
);

define_callable!(
    LessThanOrEqual,
    CallableDefinition {
        name: "<=",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", Number, Required), p!("right", Number, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::Compare {
            kind: ComparisonType::Number,
            operator: ComparisonOperator::LessThanOrEqual,
            left,
            right,
        })))
    },
    Number,
    "less than or equal",
    [],
    [(None, "(<= 12 3)")]
);

define_callable!(
    Equal,
    CallableDefinition {
        name: "=",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", Number, Required), p!("right", Number, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::Compare {
            kind: ComparisonType::Number,
            operator: ComparisonOperator::Equal,
            left,
            right,
        })))
    },
    Number,
    "equal",
    [],
    [(None, "(= 12 3)")]
);

define_callable!(
    NotEqual,
    CallableDefinition {
        name: "!=",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", Number, Required), p!("right", Number, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::Compare {
            kind: ComparisonType::Number,
            operator: ComparisonOperator::NotEqual,
            left,
            right,
        })))
    },
    Number,
    "not equal",
    [],
    [(None, "(!= 12 3)")]
);
