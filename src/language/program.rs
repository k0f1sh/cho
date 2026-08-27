use crate::ast::*;

use super::*;

define_callable!(
    Print,
    CallableDefinition {
        name: "print",
        aliases: &["p"],
        kind: CallableKind::ProgramForm,
        signatures: &[sig!([p!("value", Value, ZeroOrMore)] => None)]
    },
    |_context, arguments| { form(Form::Print(values(arguments)?)) },
    Program,
    "print values separated by spaces",
    [],
    [(None, "(print $1 $3)")]
);

define_callable!(
    Filter,
    CallableDefinition {
        name: "filter",
        aliases: &["f"],
        kind: CallableKind::ProgramForm,
        signatures: &[sig!([p!("condition", Boolean, Required)] => None)]
    },
    |_context, arguments| {
        let [condition] = value_array(arguments)?;
        form(Form::Filter(condition))
    },
    Program,
    "continue only when true",
    [],
    [(None, "(filter (> $2 20))")]
);
