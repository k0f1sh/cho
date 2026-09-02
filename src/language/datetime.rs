use crate::ast::*;

use super::*;

define_callable!(
    Unix,
    CallableDefinition {
        name: "dt/unix",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Number, Required)] => Some(ValueType::DateTime))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::DateTimeFromUnix(Box::new(value_arg)))
    },
    DateTime,
    "convert Unix seconds",
    [],
    [(None, "(dt/unix 0)")]
);

define_callable!(
    ToUnix,
    CallableDefinition {
        name: "dt/to-unix",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("datetime", DateTime, Required)] => Some(ValueType::Number))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::DateTimeToUnix(Box::new(value_arg)))
    },
    DateTime,
    "convert a datetime to Unix seconds",
    ["Fractional seconds are preserved when representable as Number."],
    [(None, "(dt/to-unix $1)")]
);

define_callable!(
    Format,
    CallableDefinition {
        name: "dt/fmt",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("datetime", DateTime, Required), p!("format", String, Required)] => Some(ValueType::String)),
            sig!([p!("datetime", DateTime, Required), p!("format", String, Required), p!("timezone", String, Required, "TIMEZONE")] => Some(ValueType::String))
        ]
    },
    |_context, arguments| {
        let mut args = values(arguments)?.into_iter();
        let value_arg = args.next().expect("signature requires datetime");
        let format = args.next().expect("signature requires format");
        value(Value::FormatDateTime {
            value: Box::new(value_arg),
            format: Box::new(format),
            timezone: args.next().map(Box::new),
        })
    },
    DateTime,
    "format a datetime",
    [],
    [
        (Some("format in UTC"), "(dt/fmt $1 \"%Y-%m-%d\")"),
        (
            Some("format in the specified timezone"),
            "(dt/fmt $1 \"%H:%M\" \"Asia/Tokyo\")"
        )
    ]
);

define_callable!(
    Now,
    CallableDefinition {
        name: "dt/now",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([] => Some(ValueType::DateTime))]
    },
    |_context, arguments| {
        let [] = value_array(arguments)?;
        value(Value::DateTimeNow)
    },
    DateTime,
    "current UTC time at second precision",
    [],
    [(None, "(dt/now)")]
);

define_callable!(
    FloorSecond,
    CallableDefinition {
        name: "dt/floor-s",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("datetime", DateTime, Required)] => Some(ValueType::DateTime)),
            sig!([p!("datetime", DateTime, Required), p!("timezone", String, Required, "TIMEZONE")] => Some(ValueType::DateTime))
        ]
    },
    |_context, arguments| {
        let mut args = values(arguments)?.into_iter();
        let value_arg = args.next().expect("signature requires datetime");
        value(Value::FloorDateTime {
            unit: DateTimeFloorUnit::Second,
            value: Box::new(value_arg),
            timezone: args.next().map(Box::new),
        })
    },
    DateTime,
    "floor to a local or UTC second",
    [],
    [
        (None, "(dt/floor-s $1)"),
        (None, "(dt/floor-s $1 \"Asia/Tokyo\")")
    ]
);

define_callable!(
    FloorMinute,
    CallableDefinition {
        name: "dt/floor-m",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("datetime", DateTime, Required)] => Some(ValueType::DateTime)),
            sig!([p!("datetime", DateTime, Required), p!("timezone", String, Required, "TIMEZONE")] => Some(ValueType::DateTime))
        ]
    },
    |_context, arguments| {
        let mut args = values(arguments)?.into_iter();
        let value_arg = args.next().expect("signature requires datetime");
        value(Value::FloorDateTime {
            unit: DateTimeFloorUnit::Minute,
            value: Box::new(value_arg),
            timezone: args.next().map(Box::new),
        })
    },
    DateTime,
    "floor to a local or UTC minute",
    [],
    [
        (None, "(dt/floor-m $1)"),
        (None, "(dt/floor-m $1 \"Asia/Tokyo\")")
    ]
);

define_callable!(
    FloorHour,
    CallableDefinition {
        name: "dt/floor-h",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("datetime", DateTime, Required)] => Some(ValueType::DateTime)),
            sig!([p!("datetime", DateTime, Required), p!("timezone", String, Required, "TIMEZONE")] => Some(ValueType::DateTime))
        ]
    },
    |_context, arguments| {
        let mut args = values(arguments)?.into_iter();
        let value_arg = args.next().expect("signature requires datetime");
        value(Value::FloorDateTime {
            unit: DateTimeFloorUnit::Hour,
            value: Box::new(value_arg),
            timezone: args.next().map(Box::new),
        })
    },
    DateTime,
    "floor to a local or UTC hour",
    [],
    [
        (None, "(dt/floor-h $1)"),
        (None, "(dt/floor-h $1 \"Asia/Tokyo\")")
    ]
);

define_callable!(
    FloorDay,
    CallableDefinition {
        name: "dt/floor-d",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("datetime", DateTime, Required)] => Some(ValueType::DateTime)),
            sig!([p!("datetime", DateTime, Required), p!("timezone", String, Required, "TIMEZONE")] => Some(ValueType::DateTime))
        ]
    },
    |_context, arguments| {
        let mut args = values(arguments)?.into_iter();
        let value_arg = args.next().expect("signature requires datetime");
        value(Value::FloorDateTime {
            unit: DateTimeFloorUnit::Day,
            value: Box::new(value_arg),
            timezone: args.next().map(Box::new),
        })
    },
    DateTime,
    "floor to a local or UTC calendar day",
    [],
    [
        (None, "(dt/floor-d $1)"),
        (None, "(dt/floor-d $1 \"Asia/Tokyo\")")
    ]
);

define_callable!(
    Add,
    CallableDefinition {
        name: "dt/add",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("datetime", DateTime, Required), p!("duration", Duration, Required)] => Some(ValueType::DateTime))
        ]
    },
    |_context, arguments| {
        let [datetime, duration] = value_array(arguments)?;
        value(Value::AddDateTime {
            datetime: Box::new(datetime),
            duration: Box::new(duration),
        })
    },
    DateTime,
    "add a duration",
    [],
    [(None, "(dt/add $1 (du/s 10))")]
);

define_callable!(
    Subtract,
    CallableDefinition {
        name: "dt/sub",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("datetime", DateTime, Required), p!("duration", Duration, Required)] => Some(ValueType::DateTime))
        ]
    },
    |_context, arguments| {
        let [datetime, duration] = value_array(arguments)?;
        value(Value::SubtractDateTime {
            datetime: Box::new(datetime),
            duration: Box::new(duration),
        })
    },
    DateTime,
    "subtract a duration",
    [],
    [(None, "(dt/sub $1 (du/s 10))")]
);

define_callable!(
    Difference,
    CallableDefinition {
        name: "dt/diff",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", DateTime, Required), p!("right", DateTime, Required)] => Some(ValueType::Duration))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::DifferenceDateTime {
            left: Box::new(left),
            right: Box::new(right),
        })
    },
    DateTime,
    "subtract the right datetime from the left",
    [],
    [(None, "(dt/diff $1 $2)")]
);

define_callable!(
    DurationSeconds,
    CallableDefinition {
        name: "du/s",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Number, Required)] => Some(ValueType::Duration))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::DurationSeconds(Box::new(value_arg)))
    },
    DateTime,
    "convert seconds to a duration",
    [],
    [(None, "(du/s 10)")]
);

define_callable!(
    DurationMilliseconds,
    CallableDefinition {
        name: "du/ms",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Number, Required)] => Some(ValueType::Duration))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::DurationMilliseconds(Box::new(value_arg)))
    },
    DateTime,
    "convert milliseconds to a duration",
    [],
    [(None, "(du/ms 250)")]
);

define_callable!(
    DurationMinutes,
    CallableDefinition {
        name: "du/m",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Number, Required)] => Some(ValueType::Duration))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::DurationMinutes(Box::new(value_arg)))
    },
    DateTime,
    "convert minutes to a duration",
    [],
    [(None, "(du/m 5)")]
);

define_callable!(
    DurationHours,
    CallableDefinition {
        name: "du/h",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Number, Required)] => Some(ValueType::Duration))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::DurationHours(Box::new(value_arg)))
    },
    DateTime,
    "convert hours to a duration",
    [],
    [(None, "(du/h 2)")]
);

define_callable!(
    DurationDays,
    CallableDefinition {
        name: "du/d",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Number, Required)] => Some(ValueType::Duration))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::DurationDays(Box::new(value_arg)))
    },
    DateTime,
    "convert fixed 24-hour days to a duration",
    [],
    [(None, "(du/d 1)")]
);

define_callable!(
    DurationToMilliseconds,
    CallableDefinition {
        name: "du/to-ms",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Duration, Required)] => Some(ValueType::Number))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::DurationToMilliseconds(Box::new(value_arg)))
    },
    DateTime,
    "convert a duration to milliseconds",
    [],
    [(None, "(du/to-ms (du/s 1))")]
);

define_callable!(
    DurationToSeconds,
    CallableDefinition {
        name: "du/to-s",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Duration, Required)] => Some(ValueType::Number))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::DurationToSeconds(Box::new(value_arg)))
    },
    DateTime,
    "convert a duration to seconds",
    [],
    [(None, "(du/to-s (du/m 1))")]
);

define_callable!(
    DurationToMinutes,
    CallableDefinition {
        name: "du/to-m",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Duration, Required)] => Some(ValueType::Number))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::DurationToMinutes(Box::new(value_arg)))
    },
    DateTime,
    "convert a duration to minutes",
    [],
    [(None, "(du/to-m (du/h 1))")]
);

define_callable!(
    DurationToHours,
    CallableDefinition {
        name: "du/to-h",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Duration, Required)] => Some(ValueType::Number))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::DurationToHours(Box::new(value_arg)))
    },
    DateTime,
    "convert a duration to hours",
    [],
    [(None, "(du/to-h (du/d 1))")]
);

define_callable!(
    DurationToDays,
    CallableDefinition {
        name: "du/to-d",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Duration, Required)] => Some(ValueType::Number))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::DurationToDays(Box::new(value_arg)))
    },
    DateTime,
    "convert a duration to fixed 24-hour days",
    [],
    [(None, "(du/to-d (du/h 24))")]
);

define_callable!(
    GreaterThan,
    CallableDefinition {
        name: "dt/>",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", DateTime, Required), p!("right", DateTime, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::Compare {
            kind: ComparisonType::DateTime,
            operator: ComparisonOperator::GreaterThan,
            left,
            right,
        })))
    },
    DateTime,
    "greater than",
    [],
    [(None, "(dt/> $1 $2)")]
);

define_callable!(
    GreaterThanOrEqual,
    CallableDefinition {
        name: "dt/>=",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", DateTime, Required), p!("right", DateTime, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::Compare {
            kind: ComparisonType::DateTime,
            operator: ComparisonOperator::GreaterThanOrEqual,
            left,
            right,
        })))
    },
    DateTime,
    "greater than or equal",
    [],
    [(None, "(dt/>= $1 $2)")]
);

define_callable!(
    LessThan,
    CallableDefinition {
        name: "dt/<",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", DateTime, Required), p!("right", DateTime, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::Compare {
            kind: ComparisonType::DateTime,
            operator: ComparisonOperator::LessThan,
            left,
            right,
        })))
    },
    DateTime,
    "less than",
    [],
    [(None, "(dt/< $1 $2)")]
);

define_callable!(
    LessThanOrEqual,
    CallableDefinition {
        name: "dt/<=",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", DateTime, Required), p!("right", DateTime, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::Compare {
            kind: ComparisonType::DateTime,
            operator: ComparisonOperator::LessThanOrEqual,
            left,
            right,
        })))
    },
    DateTime,
    "less than or equal",
    [],
    [(None, "(dt/<= $1 $2)")]
);

define_callable!(
    Equal,
    CallableDefinition {
        name: "dt/=",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", DateTime, Required), p!("right", DateTime, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::Compare {
            kind: ComparisonType::DateTime,
            operator: ComparisonOperator::Equal,
            left,
            right,
        })))
    },
    DateTime,
    "equal",
    [],
    [(None, "(dt/= $1 $2)")]
);

define_callable!(
    NotEqual,
    CallableDefinition {
        name: "dt/!=",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", DateTime, Required), p!("right", DateTime, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::Compare {
            kind: ComparisonType::DateTime,
            operator: ComparisonOperator::NotEqual,
            left,
            right,
        })))
    },
    DateTime,
    "not equal",
    [],
    [(None, "(dt/!= $1 $2)")]
);
