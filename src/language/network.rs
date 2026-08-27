use crate::ast::*;

use super::*;

define_callable!(
    IpVersion,
    CallableDefinition {
        name: "ip/version",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", IpAddr, Required)] => Some(ValueType::Number))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::IpVersion(Box::new(value_arg)))
    },
    Network,
    "return 4 or 6",
    [],
    [(None, "(ip/version $1)")]
);

define_callable!(
    IpEqual,
    CallableDefinition {
        name: "ip/=",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", IpAddr, Required), p!("right", IpAddr, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::Compare {
            kind: ComparisonType::IpAddr,
            operator: ComparisonOperator::Equal,
            left,
            right,
        })))
    },
    Network,
    "test IP address equality",
    [],
    [(None, "(ip/= $1 \"127.0.0.1\")")]
);

define_callable!(
    IpNotEqual,
    CallableDefinition {
        name: "ip/!=",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", IpAddr, Required), p!("right", IpAddr, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::Compare {
            kind: ComparisonType::IpAddr,
            operator: ComparisonOperator::NotEqual,
            left,
            right,
        })))
    },
    Network,
    "test IP address inequality",
    [],
    [(None, "(ip/!= $1 \"127.0.0.1\")")]
);

define_callable!(
    IpPrivate,
    CallableDefinition {
        name: "ip/private?",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", IpAddr, Required)] => Some(ValueType::Boolean))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::IpClass {
            kind: IpClass::Private,
            value: value_arg,
        })))
    },
    Network,
    "test RFC 1918 IPv4 or fc00::/7 IPv6 ULA",
    [],
    [(None, "(ip/private? $1)")]
);

define_callable!(
    IpLoopback,
    CallableDefinition {
        name: "ip/loopback?",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", IpAddr, Required)] => Some(ValueType::Boolean))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::IpClass {
            kind: IpClass::Loopback,
            value: value_arg,
        })))
    },
    Network,
    "test a loopback address",
    [],
    [(None, "(ip/loopback? $1)")]
);

define_callable!(
    IpLinkLocal,
    CallableDefinition {
        name: "ip/link-local?",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", IpAddr, Required)] => Some(ValueType::Boolean))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::IpClass {
            kind: IpClass::LinkLocal,
            value: value_arg,
        })))
    },
    Network,
    "test a link-local address",
    [],
    [(None, "(ip/link-local? $1)")]
);

define_callable!(
    IpMulticast,
    CallableDefinition {
        name: "ip/multicast?",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", IpAddr, Required)] => Some(ValueType::Boolean))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::IpClass {
            kind: IpClass::Multicast,
            value: value_arg,
        })))
    },
    Network,
    "test a multicast address",
    [],
    [(None, "(ip/multicast? $1)")]
);

define_callable!(
    CidrContains,
    CallableDefinition {
        name: "cidr/contains?",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("cidr", Cidr, Required), p!("ip", IpAddr, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [cidr, ip] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::CidrContains {
            cidr,
            ip,
        })))
    },
    Network,
    "test CIDR membership",
    [],
    [(None, "(cidr/contains? \"10.0.0.0/8\" $1)")]
);

define_callable!(
    CidrNetwork,
    CallableDefinition {
        name: "cidr/network",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Cidr, Required)] => Some(ValueType::IpAddr))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::CidrPart {
            part: CidrPart::Network,
            value: Box::new(value_arg),
        })
    },
    Network,
    "return the network address",
    [],
    [(None, "(cidr/network $1)")]
);

define_callable!(
    CidrPrefix,
    CallableDefinition {
        name: "cidr/prefix",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Cidr, Required)] => Some(ValueType::Number))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::CidrPart {
            part: CidrPart::Prefix,
            value: Box::new(value_arg),
        })
    },
    Network,
    "return the prefix length",
    [],
    [(None, "(cidr/prefix $1)")]
);

define_callable!(
    CidrFirst,
    CallableDefinition {
        name: "cidr/first",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Cidr, Required)] => Some(ValueType::IpAddr))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::CidrPart {
            part: CidrPart::First,
            value: Box::new(value_arg),
        })
    },
    Network,
    "return the lowest address",
    [],
    [(None, "(cidr/first $1)")]
);

define_callable!(
    CidrLast,
    CallableDefinition {
        name: "cidr/last",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Cidr, Required)] => Some(ValueType::IpAddr))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::CidrPart {
            part: CidrPart::Last,
            value: Box::new(value_arg),
        })
    },
    Network,
    "return the highest address",
    [],
    [(None, "(cidr/last $1)")]
);

define_callable!(
    CidrSize,
    CallableDefinition {
        name: "cidr/size",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", Cidr, Required)] => Some(ValueType::Number))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::CidrPart {
            part: CidrPart::Size,
            value: Box::new(value_arg),
        })
    },
    Network,
    "return the address count up to 2^53 - 1",
    [],
    [(None, "(cidr/size $1)")]
);
