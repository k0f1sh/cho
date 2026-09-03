use crate::ast::*;

use super::*;

define_callable!(
    Normalize,
    CallableDefinition {
        name: "date",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Date, Required)] => Some(ValueType::Date))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::NormalizeDate(Box::new(value_arg)))
    },
    Date,
    "validate a calendar date",
    [],
    [(None, "(date $1)")]
);

macro_rules! part {
    ($type:ident, $name:literal, $part:ident, $summary:literal, $example:literal) => {
        define_callable!(
            $type,
            CallableDefinition {
                name: $name,
                aliases: &[],
                kind: CallableKind::Function,
                signatures: &[sig!([p!("date", Date, Required)] => Some(ValueType::Number))]
            },
            |_context, arguments| {
                let [value_arg] = value_array(arguments)?;
                value(Value::DatePart {
                    part: DatePart::$part,
                    value: Box::new(value_arg),
                })
            },
            Date,
            $summary,
            [],
            [(None, $example)]
        );
    };
}

part!(Year, "d/year", Year, "extract the year", "(d/year $1)");
part!(
    Month,
    "d/month",
    Month,
    "extract the month number",
    "(d/month $1)"
);
part!(
    Day,
    "d/day",
    Day,
    "extract the day of the month",
    "(d/day $1)"
);
part!(
    Weekday,
    "d/weekday",
    Weekday,
    "extract the ISO weekday number",
    "(d/weekday $1)"
);

define_callable!(
    Add,
    CallableDefinition {
        name: "d/add",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("date", Date, Required), p!("days", Number, Required, "DAYS")] => Some(ValueType::Date))
        ]
    },
    |_context, arguments| {
        let [date, days] = value_array(arguments)?;
        value(Value::AddDate {
            date: Box::new(date),
            days: Box::new(days),
        })
    },
    Date,
    "add whole calendar days",
    ["DAYS may be negative."],
    [(None, "(d/add $1 7)")]
);

define_callable!(
    Subtract,
    CallableDefinition {
        name: "d/sub",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("date", Date, Required), p!("days", Number, Required, "DAYS")] => Some(ValueType::Date))
        ]
    },
    |_context, arguments| {
        let [date, days] = value_array(arguments)?;
        value(Value::SubtractDate {
            date: Box::new(date),
            days: Box::new(days),
        })
    },
    Date,
    "subtract whole calendar days",
    ["DAYS may be negative."],
    [(None, "(d/sub $1 7)")]
);

define_callable!(
    Difference,
    CallableDefinition {
        name: "d/diff",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", Date, Required), p!("right", Date, Required)] => Some(ValueType::Number))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::DifferenceDate {
            left: Box::new(left),
            right: Box::new(right),
        })
    },
    Date,
    "difference in calendar days",
    ["Returns LEFT minus RIGHT."],
    [(None, "(d/diff $1 $2)")]
);

macro_rules! comparison {
    ($type:ident, $name:literal, $operator:ident, $summary:literal, $example:literal) => {
        define_callable!(
            $type,
            CallableDefinition {
                name: $name,
                aliases: &[],
                kind: CallableKind::Function,
                signatures: &[
                    sig!([p!("left", Date, Required), p!("right", Date, Required)] => Some(ValueType::Boolean))
                ]
            },
            |_context, arguments| {
                let [left, right] = value_array(arguments)?;
                value(Value::Predicate(Box::new(Predicate::Compare {
                    kind: ComparisonType::Date,
                    operator: ComparisonOperator::$operator,
                    left,
                    right,
                })))
            },
            Date,
            $summary,
            [],
            [(None, $example)]
        );
    };
}

comparison!(GreaterThan, "d/>", GreaterThan, "later than", "(d/> $1 $2)");
comparison!(
    GreaterThanOrEqual,
    "d/>=",
    GreaterThanOrEqual,
    "later than or equal",
    "(d/>= $1 $2)"
);
comparison!(LessThan, "d/<", LessThan, "earlier than", "(d/< $1 $2)");
comparison!(
    LessThanOrEqual,
    "d/<=",
    LessThanOrEqual,
    "earlier than or equal",
    "(d/<= $1 $2)"
);
comparison!(Equal, "d/=", Equal, "equal", "(d/= $1 $2)");
comparison!(NotEqual, "d/!=", NotEqual, "not equal", "(d/!= $1 $2)");
