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
        let [value_arg] = expr_array(arguments)?;
        expr(Expr::IpVersion(Box::new(value_arg)))
    },
    Network,
    "return 4 or 6",
    [],
    [(None, "(ip/version $1)")]
);

define_callable!(
    IpV4,
    CallableDefinition {
        name: "ip/v4?",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", IpAddr, Required)] => Some(ValueType::Boolean))]
    },
    |_context, arguments| {
        let [value_arg] = expr_array(arguments)?;
        expr(Expr::Predicate(Box::new(Predicate::IpClass {
            kind: IpClass::V4,
            value: value_arg,
        })))
    },
    Network,
    "test an IPv4 address",
    [],
    [(None, "(ip/v4? $1)")]
);

define_callable!(
    IpV6,
    CallableDefinition {
        name: "ip/v6?",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", IpAddr, Required)] => Some(ValueType::Boolean))]
    },
    |_context, arguments| {
        let [value_arg] = expr_array(arguments)?;
        expr(Expr::Predicate(Box::new(Predicate::IpClass {
            kind: IpClass::V6,
            value: value_arg,
        })))
    },
    Network,
    "test an IPv6 address",
    [],
    [(None, "(ip/v6? $1)")]
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
        let [left, right] = expr_array(arguments)?;
        expr(Expr::Predicate(Box::new(Predicate::Compare {
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
        let [left, right] = expr_array(arguments)?;
        expr(Expr::Predicate(Box::new(Predicate::Compare {
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
        let [value_arg] = expr_array(arguments)?;
        expr(Expr::Predicate(Box::new(Predicate::IpClass {
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
        let [value_arg] = expr_array(arguments)?;
        expr(Expr::Predicate(Box::new(Predicate::IpClass {
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
        let [value_arg] = expr_array(arguments)?;
        expr(Expr::Predicate(Box::new(Predicate::IpClass {
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
        let [value_arg] = expr_array(arguments)?;
        expr(Expr::Predicate(Box::new(Predicate::IpClass {
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
        let [cidr, ip] = expr_array(arguments)?;
        expr(Expr::Predicate(Box::new(Predicate::CidrContains {
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
        let [value_arg] = expr_array(arguments)?;
        expr(Expr::CidrPart {
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
        let [value_arg] = expr_array(arguments)?;
        expr(Expr::CidrPart {
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
        let [value_arg] = expr_array(arguments)?;
        expr(Expr::CidrPart {
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
        let [value_arg] = expr_array(arguments)?;
        expr(Expr::CidrPart {
            part: CidrPart::Last,
            value: Box::new(value_arg),
        })
    },
    Network,
    "return the highest address",
    [
        "For IPv4 /0 through /30 this is the broadcast address. IPv4 /31 and /32 have no ordinary subnet broadcast; IPv6 has no broadcast."
    ],
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
        let [value_arg] = expr_array(arguments)?;
        expr(Expr::CidrPart {
            part: CidrPart::Size,
            value: Box::new(value_arg),
        })
    },
    Network,
    "return the address count up to 2^53 - 1",
    [],
    [(None, "(cidr/size $1)")]
);

define_callable!(
    NormalizeIp,
    CallableDefinition {
        name: "ip",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", IpAddr, Required)] => Some(ValueType::IpAddr))]
    },
    |_context, arguments| {
        let [value] = expr_array(arguments)?;
        expr(Expr::NormalizeIp(Box::new(value)))
    },
    Network,
    "validate and normalize an IP address",
    ["IPv6 renders in compressed form; IPv4 renders in dotted decimal."],
    [(None, "(ip $1)", "2001:0DB8:0:0:0:0:0:1", "2001:db8::1")]
);

macro_rules! ip_format {
    ($type:ident, $name:literal, $format:ident, $summary:literal, $note:literal,
     $example:literal, $input:literal, $output:literal) => {
        define_callable!(
            $type,
            CallableDefinition {
                name: $name,
                aliases: &[],
                kind: CallableKind::Function,
                signatures: &[sig!([p!("value", IpAddr, Required)] => Some(ValueType::String))]
            },
            |_context, arguments| {
                let [value] = expr_array(arguments)?;
                expr(Expr::FormatIp { format: IpFormat::$format, value: Box::new(value) })
            },
            Network,
            $summary,
            [$note],
            [(None, $example, $input, $output)]
        );
    };
}

ip_format!(
    IpExpand,
    "ip/expand",
    Expanded,
    "return the full IP address as a string",
    "IPv6 uses eight lowercase, zero-padded four-digit groups; IPv4 uses dotted decimal.",
    "(ip/expand $1)",
    "2001:db8::1",
    "2001:0db8:0000:0000:0000:0000:0000:0001"
);
ip_format!(
    IpBinary,
    "ip/binary",
    Binary,
    "return the address bits as a string",
    "IPv4 uses four 8-bit groups separated by dots; IPv6 uses eight 16-bit groups separated by colons.",
    "(ip/binary $1)",
    "192.0.2.1",
    "11000000.00000000.00000010.00000001"
);

define_callable!(
    NormalizeCidr,
    CallableDefinition {
        name: "cidr",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("value", Cidr, Required)] => Some(ValueType::Cidr)),
            sig!([p!("ip", IpAddr, Required), p!("prefix", Number, Required, "PREFIX")] => Some(ValueType::Cidr))
        ]
    },
    |_context, arguments| {
        let mut arguments = exprs(arguments)?.into_iter();
        let value = Box::new(arguments.next().ok_or(ParseError::InvalidSyntax)?);
        match arguments.next() {
            Some(prefix) => expr(Expr::MakeCidr {
                ip: value,
                prefix: Box::new(prefix),
            }),
            None => expr(Expr::NormalizeCidr(value)),
        }
    },
    Network,
    "create a CIDR normalized to its network address",
    [
        "PREFIX must be an integer from 0 to 32 for IPv4 or 0 to 128 for IPv6.",
        "A single input requires address/prefix notation. No class-based default mask is inferred."
    ],
    [
        (None, "(cidr $1)", "192.168.1.42/24", "192.168.1.0/24"),
        (None, "(cidr $1 64)", "2001:db8::42", "2001:db8::/64")
    ]
);

define_callable!(
    CidrFromMask,
    CallableDefinition {
        name: "cidr/from-mask",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("ip", IpAddr, Required), p!("mask", IpAddr, Required, "MASK")] => Some(ValueType::Cidr))
        ]
    },
    |_context, arguments| {
        let [ip, mask] = expr_array(arguments)?;
        expr(Expr::CidrFromMask {
            ip: Box::new(ip),
            mask: Box::new(mask),
        })
    },
    Network,
    "create a CIDR from an IP address and netmask",
    [
        "MASK must use the same address family and contain leading ones followed by zeros; inverse masks are not inferred."
    ],
    [(
        None,
        "(cidr/from-mask $1 $2)",
        "192.168.1.42 255.255.255.0",
        "192.168.1.0/24"
    )]
);

macro_rules! cidr_part {
    ($type:ident, $name:literal, $part:ident, $result:ident, $summary:literal,
     $note:literal, $example:literal, $input:literal, $output:literal) => {
        define_callable!(
            $type,
            CallableDefinition {
                name: $name,
                aliases: &[],
                kind: CallableKind::Function,
                signatures: &[sig!([p!("value", Cidr, Required)] => Some(ValueType::$result))]
            },
            |_context, arguments| {
                let [value] = expr_array(arguments)?;
                expr(Expr::CidrPart { part: CidrPart::$part, value: Box::new(value) })
            },
            Network,
            $summary,
            [$note],
            [(None, $example, $input, $output)]
        );
    };
}

cidr_part!(
    CidrNetmask,
    "cidr/netmask",
    Netmask,
    IpAddr,
    "return the network mask",
    "Returns an IPv4 or IPv6 address with the prefix bits set to one.",
    "(cidr/netmask $1)",
    "192.168.1.42/24",
    "255.255.255.0"
);
cidr_part!(
    CidrWildcard,
    "cidr/wildcard",
    Wildcard,
    IpAddr,
    "return the inverse network mask",
    "Inverts all 32 IPv4 or 128 IPv6 bits of the netmask.",
    "(cidr/wildcard $1)",
    "192.168.1.42/24",
    "0.0.0.255"
);
cidr_part!(
    CidrHostFirst,
    "cidr/host-first",
    HostFirst,
    IpAddr,
    "return the first host address",
    "IPv4 /0 through /30 exclude network and broadcast; /31, /32, and IPv6 include both endpoints. This is a range calculation, not an allocation check.",
    "(cidr/host-first $1)",
    "192.168.1.42/24",
    "192.168.1.1"
);
cidr_part!(
    CidrHostLast,
    "cidr/host-last",
    HostLast,
    IpAddr,
    "return the last host address",
    "IPv4 /0 through /30 exclude network and broadcast; /31, /32, and IPv6 include both endpoints. This is a range calculation, not an allocation check.",
    "(cidr/host-last $1)",
    "192.168.1.42/24",
    "192.168.1.254"
);
cidr_part!(
    CidrHostCount,
    "cidr/host-count",
    HostCount,
    String,
    "return the exact host count as a decimal string",
    "IPv4 /0 through /30 subtract network and broadcast; /31, /32, and IPv6 count every address. Other reserved addresses may be included; the count does not guarantee assignability. String output has no units or separators; conversion to Number can lose precision.",
    "(cidr/host-count $1)",
    "192.168.1.42/24",
    "254"
);
cidr_part!(
    CidrSizeString,
    "cidr/size-str",
    SizeString,
    String,
    "return the exact address count as a decimal string",
    "Counts every address, including IPv6 /0 (2^128). String output has no units or separators; conversion to Number can lose precision.",
    "(cidr/size-str $1)",
    "2001:db8::/64",
    "18446744073709551616"
);
