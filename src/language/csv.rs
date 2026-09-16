use crate::ast::Expr;

use super::*;

define_callable!(
    Join,
    CallableDefinition {
        name: "csv/join",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Value, ZeroOrMore)] => Some(ValueType::String))]
    },
    |_context, arguments| { expr(Expr::CsvJoin(exprs(arguments)?)) },
    Csv,
    "join values as one CSV record",
    [
        "Fields containing commas, double quotes, CR, or LF are quoted; embedded double quotes are doubled.",
        "A single empty field renders as \"\". The result has no record-ending newline."
    ],
    [(None, "(csv/join $1 $2)")]
);
