use crate::ast::Value;

use super::*;

define_callable!(
    Field,
    CallableDefinition {
        name: "field",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("number", Number, Required)] => Some(ValueType::String))]
    },
    |_context, arguments| {
        let [number] = value_array(arguments)?;
        value(Value::DynamicField(Box::new(number)))
    },
    Field,
    "get a field by its computed number",
    [
        "NUMBER must be a non-negative whole number. Field 0 is the complete record; a missing field is an empty string."
    ],
    [(None, "(field (- NF 1))")]
);

define_callable!(
    Fields,
    CallableDefinition {
        name: "fields",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([
            p!("start", Number, Required, "START"),
            p!("end", Number, Required, "END")
        ] => Some(ValueType::String))]
    },
    |context, arguments| {
        let [start, end] = value_array(arguments)?;
        context.mark_field_range();
        value(Value::DynamicFieldRange {
            start: Some(Box::new(start)),
            end: Some(Box::new(end)),
        })
    },
    Field,
    "get a computed inclusive field range",
    [
        "START and END must be positive whole numbers. Original separators between the fields are preserved."
    ],
    [(None, "(fields 2 (- NF 1))")]
);

define_callable!(
    FieldsFrom,
    CallableDefinition {
        name: "fields-from",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("start", Number, Required, "START")] => Some(ValueType::String))]
    },
    |context, arguments| {
        let [start] = value_array(arguments)?;
        context.mark_field_range();
        value(Value::DynamicFieldRange {
            start: Some(Box::new(start)),
            end: None,
        })
    },
    Field,
    "get fields from a computed position through the record end",
    ["START must be a positive whole number. Trailing separators are preserved."],
    [(None, "(fields-from (- NF 2))")]
);

define_callable!(
    FieldsTo,
    CallableDefinition {
        name: "fields-to",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("end", Number, Required, "END")] => Some(ValueType::String))]
    },
    |context, arguments| {
        let [end] = value_array(arguments)?;
        context.mark_field_range();
        value(Value::DynamicFieldRange {
            start: None,
            end: Some(Box::new(end)),
        })
    },
    Field,
    "get fields from the record start through a computed position",
    ["END must be a positive whole number. Leading separators are preserved."],
    [(None, "(fields-to (- NF 1))")]
);
