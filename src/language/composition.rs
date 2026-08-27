use super::*;

define_callable!(
    ThreadFirst,
    CallableDefinition {
        name: "->",
        aliases: &[],
        kind: CallableKind::ThreadingForm,
        signatures: &[
            sig!([p!("value", Value, Required), p!("step", Step, ZeroOrMore)] => Some(ValueType::Value))
        ]
    },
    |_context, arguments| {
        _context
            .compile_threading(ThreadDirection::First, arguments)
            .and_then(value)
    },
    Composition,
    "insert a value as each step's first argument",
    [],
    [(None, "(-> $1 s/trim s/upper)")]
);

define_callable!(
    ThreadLast,
    CallableDefinition {
        name: "->>",
        aliases: &[],
        kind: CallableKind::ThreadingForm,
        signatures: &[
            sig!([p!("value", Value, Required), p!("step", Step, ZeroOrMore)] => Some(ValueType::Value))
        ]
    },
    |_context, arguments| {
        _context
            .compile_threading(ThreadDirection::Last, arguments)
            .and_then(value)
    },
    Composition,
    "insert a value as each step's last argument",
    [],
    [(None, "(->> $1 (str \"value=\"))")]
);
