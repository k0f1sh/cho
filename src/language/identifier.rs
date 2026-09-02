use crate::ast::*;

use super::*;

define_callable!(
    UuidNormalize,
    CallableDefinition {
        name: "uuid",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Uuid, Required)] => Some(ValueType::Uuid))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::NormalizeUuid(Box::new(value_arg)))
    },
    Identifier,
    "validate and normalize a UUID",
    ["Accepts simple, hyphenated, braced, and URN forms; renders lowercase hyphenated."],
    [(None, "(uuid $1)")]
);

define_callable!(
    UuidV4,
    CallableDefinition {
        name: "uuid/v4",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([] => Some(ValueType::Uuid))]
    },
    |_context, arguments| {
        let [] = value_array(arguments)?;
        value(Value::UuidV4)
    },
    Identifier,
    "generate a random UUID version 4",
    [],
    [(None, "(uuid/v4)", "<random UUID v4>")]
);

define_callable!(
    UuidV7,
    CallableDefinition {
        name: "uuid/v7",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([] => Some(ValueType::Uuid))]
    },
    |_context, arguments| {
        let [] = value_array(arguments)?;
        value(Value::UuidV7)
    },
    Identifier,
    "generate a time-ordered UUID version 7",
    ["Generation order is preserved within one cho invocation."],
    [(None, "(uuid/v7)", "<time-ordered UUID v7>")]
);

define_callable!(
    UuidVersion,
    CallableDefinition {
        name: "uuid/version",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Uuid, Required)] => Some(ValueType::Number))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::UuidVersion(Box::new(value_arg)))
    },
    Identifier,
    "return the UUID version number",
    [],
    [(None, "(uuid/version $1)")]
);

define_callable!(
    UuidTime,
    CallableDefinition {
        name: "uuid/time",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Uuid, Required)] => Some(ValueType::DateTime))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::UuidTime(Box::new(value_arg)))
    },
    Identifier,
    "extract the timestamp from a UUID",
    ["The UUID must be version 1, 6, or 7."],
    [(None, "(uuid/time $1)")]
);

define_callable!(
    UlidNormalize,
    CallableDefinition {
        name: "ulid",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Ulid, Required)] => Some(ValueType::Ulid))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::NormalizeUlid(Box::new(value_arg)))
    },
    Identifier,
    "validate and normalize a ULID",
    ["Accepts upper- or lowercase Crockford Base32; renders uppercase."],
    [(None, "(ulid $1)")]
);

define_callable!(
    UlidNew,
    CallableDefinition {
        name: "ulid/new",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([] => Some(ValueType::Ulid))]
    },
    |_context, arguments| {
        let [] = value_array(arguments)?;
        value(Value::UlidNew)
    },
    Identifier,
    "generate a monotonic ULID",
    ["Generation order is preserved within one cho invocation."],
    [(None, "(ulid/new)", "<monotonic ULID>")]
);

define_callable!(
    UlidTime,
    CallableDefinition {
        name: "ulid/time",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Ulid, Required)] => Some(ValueType::DateTime))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::UlidTime(Box::new(value_arg)))
    },
    Identifier,
    "extract the timestamp from a ULID",
    [],
    [(None, "(ulid/time $1)")]
);

macro_rules! comparison {
    ($type:ident, $name:literal, $value_type:ident, $kind:ident, $operator:ident, $summary:literal, $example:literal) => {
        define_callable!(
            $type,
            CallableDefinition {
                name: $name,
                aliases: &[],
                kind: CallableKind::Function,
                signatures: &[
                    sig!([p!("left", $value_type, Required), p!("right", $value_type, Required)] => Some(ValueType::Boolean))
                ]
            },
            |_context, arguments| {
                let [left, right] = value_array(arguments)?;
                value(Value::Predicate(Box::new(Predicate::Compare {
                    kind: ComparisonType::$kind,
                    operator: ComparisonOperator::$operator,
                    left,
                    right,
                })))
            },
            Identifier,
            $summary,
            [],
            [(None, $example)]
        );
    };
}

comparison!(
    UuidGreaterThan,
    "uuid/>",
    Uuid,
    Uuid,
    GreaterThan,
    "greater than",
    "(uuid/> $1 $2)"
);
comparison!(
    UuidGreaterThanOrEqual,
    "uuid/>=",
    Uuid,
    Uuid,
    GreaterThanOrEqual,
    "greater than or equal",
    "(uuid/>= $1 $2)"
);
comparison!(
    UuidLessThan,
    "uuid/<",
    Uuid,
    Uuid,
    LessThan,
    "less than",
    "(uuid/< $1 $2)"
);
comparison!(
    UuidLessThanOrEqual,
    "uuid/<=",
    Uuid,
    Uuid,
    LessThanOrEqual,
    "less than or equal",
    "(uuid/<= $1 $2)"
);
comparison!(
    UuidEqual,
    "uuid/=",
    Uuid,
    Uuid,
    Equal,
    "equal",
    "(uuid/= $1 $2)"
);
comparison!(
    UuidNotEqual,
    "uuid/!=",
    Uuid,
    Uuid,
    NotEqual,
    "not equal",
    "(uuid/!= $1 $2)"
);

comparison!(
    UlidGreaterThan,
    "ulid/>",
    Ulid,
    Ulid,
    GreaterThan,
    "greater than",
    "(ulid/> $1 $2)"
);
comparison!(
    UlidGreaterThanOrEqual,
    "ulid/>=",
    Ulid,
    Ulid,
    GreaterThanOrEqual,
    "greater than or equal",
    "(ulid/>= $1 $2)"
);
comparison!(
    UlidLessThan,
    "ulid/<",
    Ulid,
    Ulid,
    LessThan,
    "less than",
    "(ulid/< $1 $2)"
);
comparison!(
    UlidLessThanOrEqual,
    "ulid/<=",
    Ulid,
    Ulid,
    LessThanOrEqual,
    "less than or equal",
    "(ulid/<= $1 $2)"
);
comparison!(
    UlidEqual,
    "ulid/=",
    Ulid,
    Ulid,
    Equal,
    "equal",
    "(ulid/= $1 $2)"
);
comparison!(
    UlidNotEqual,
    "ulid/!=",
    Ulid,
    Ulid,
    NotEqual,
    "not equal",
    "(ulid/!= $1 $2)"
);
