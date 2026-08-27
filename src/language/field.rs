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
