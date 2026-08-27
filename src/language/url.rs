use crate::ast::*;

use super::*;

define_callable!(
    Scheme,
    CallableDefinition {
        name: "url/scheme",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Url, Required)] => Some(ValueType::String))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::UrlPart {
            part: UrlPart::Scheme,
            value: Box::new(value_arg),
        })
    },
    Url,
    "extract the scheme",
    [],
    [(None, "(url/scheme $1)")]
);

define_callable!(
    Host,
    CallableDefinition {
        name: "url/host",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Url, Required)] => Some(ValueType::String))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::UrlPart {
            part: UrlPart::Host,
            value: Box::new(value_arg),
        })
    },
    Url,
    "extract the host",
    [],
    [(None, "(url/host $1)")]
);

define_callable!(
    Port,
    CallableDefinition {
        name: "url/port",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Url, Required)] => Some(ValueType::String))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::UrlPart {
            part: UrlPart::Port,
            value: Box::new(value_arg),
        })
    },
    Url,
    "extract the port or an empty string",
    [],
    [(None, "(url/port $1)")]
);

define_callable!(
    Path,
    CallableDefinition {
        name: "url/path",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Url, Required)] => Some(ValueType::String))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::UrlPart {
            part: UrlPart::Path,
            value: Box::new(value_arg),
        })
    },
    Url,
    "extract the path",
    [],
    [(None, "(url/path $1)")]
);

define_callable!(
    Query,
    CallableDefinition {
        name: "url/query",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Url, Required)] => Some(ValueType::String))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::UrlPart {
            part: UrlPart::Query,
            value: Box::new(value_arg),
        })
    },
    Url,
    "extract the query or an empty string",
    [],
    [(None, "(url/query $1)")]
);

define_callable!(
    Fragment,
    CallableDefinition {
        name: "url/fragment",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Url, Required)] => Some(ValueType::String))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::UrlPart {
            part: UrlPart::Fragment,
            value: Box::new(value_arg),
        })
    },
    Url,
    "extract the fragment or an empty string",
    [],
    [(None, "(url/fragment $1)")]
);

define_callable!(
    QueryGet,
    CallableDefinition {
        name: "url/query-get",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("url", Url, Required), p!("name", String, Required)] => Some(ValueType::String))
        ]
    },
    |_context, arguments| {
        let [url, name] = value_array(arguments)?;
        value(Value::UrlQueryGet {
            url: Box::new(url),
            name: Box::new(name),
        })
    },
    Url,
    "return the first decoded value or an empty string",
    [],
    [(None, "(url/query-get $1 \"page\")")]
);

define_callable!(
    QueryHas,
    CallableDefinition {
        name: "url/query-has?",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("url", Url, Required), p!("name", String, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [url, name] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::UrlQueryHas {
            url,
            name,
        })))
    },
    Url,
    "test whether a decoded query name exists",
    [],
    [(None, "(url/query-has? $1 \"page\")")]
);

define_callable!(
    Encode,
    CallableDefinition {
        name: "url/encode",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", String, Required)] => Some(ValueType::String))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::UrlEncoding {
            operation: UrlEncoding::Encode,
            value: Box::new(value_arg),
        })
    },
    Url,
    "encode an RFC 3986 component",
    [],
    [(None, "(url/encode $1)")]
);

define_callable!(
    Decode,
    CallableDefinition {
        name: "url/decode",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", String, Required)] => Some(ValueType::String))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::UrlEncoding {
            operation: UrlEncoding::Decode,
            value: Box::new(value_arg),
        })
    },
    Url,
    "percent-decode a component",
    [],
    [(None, "(url/decode $1)")]
);
