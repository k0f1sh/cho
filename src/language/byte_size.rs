use crate::ast::*;

use super::*;

define_callable!(
    Normalize,
    CallableDefinition {
        name: "bs",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", ByteSize, Required)] => Some(ValueType::ByteSize))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::NormalizeByteSize(Box::new(value_arg)))
    },
    ByteSize,
    "validate and normalize a byte size",
    [],
    [(None, "(bs $1)")]
);

define_callable!(
    ToBytes,
    CallableDefinition {
        name: "bs/to-b",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", ByteSize, Required)] => Some(ValueType::Number))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::ByteSizeToBytes(Box::new(value_arg)))
    },
    ByteSize,
    "convert a byte size to bytes",
    ["Values greater than 2^53 - 1 B are outside Number's safe integer range and are errors."],
    [(None, "(bs/to-b $1)")]
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
                    sig!([p!("left", ByteSize, Required), p!("right", ByteSize, Required)] => Some(ValueType::Boolean))
                ]
            },
            |_context, arguments| {
                let [left, right] = value_array(arguments)?;
                value(Value::Predicate(Box::new(Predicate::Compare {
                    kind: ComparisonType::ByteSize,
                    operator: ComparisonOperator::$operator,
                    left,
                    right,
                })))
            },
            ByteSize,
            $summary,
            [],
            [(None, $example)]
        );
    };
}

comparison!(
    GreaterThan,
    "bs/>",
    GreaterThan,
    "greater than",
    "(bs/> $1 $2)"
);
comparison!(
    GreaterThanOrEqual,
    "bs/>=",
    GreaterThanOrEqual,
    "greater than or equal",
    "(bs/>= $1 $2)"
);
comparison!(LessThan, "bs/<", LessThan, "less than", "(bs/< $1 $2)");
comparison!(
    LessThanOrEqual,
    "bs/<=",
    LessThanOrEqual,
    "less than or equal",
    "(bs/<= $1 $2)"
);
comparison!(Equal, "bs/=", Equal, "equal", "(bs/= $1 $2)");
comparison!(NotEqual, "bs/!=", NotEqual, "not equal", "(bs/!= $1 $2)");
