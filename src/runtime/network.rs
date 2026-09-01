use std::net::IpAddr;

use ipnet::IpNet;

use crate::ast::{CidrPart, IpClass};

use super::value::{EvalError, EvalResult, RuntimeValue};

pub(super) fn expect_ip(
    value: RuntimeValue,
    function: &'static str,
    argument: usize,
) -> EvalResult<IpAddr> {
    match value {
        RuntimeValue::IpAddr(value) => Ok(value),
        RuntimeValue::String(value) => value.parse().map_err(|_| {
            EvalError::conversion(
                function,
                argument,
                "IpAddr",
                value,
                "is not a valid IPv4 or IPv6 address",
            )
        }),
        value => Err(EvalError::conversion(
            function,
            argument,
            "IpAddr",
            value.render(),
            format!("has type {}", value.type_name()),
        )),
    }
}

pub(super) fn cidr_part_name(part: &CidrPart) -> &'static str {
    match part {
        CidrPart::Network => "cidr/network",
        CidrPart::Prefix => "cidr/prefix",
        CidrPart::First => "cidr/first",
        CidrPart::Last => "cidr/last",
        CidrPart::Size => "cidr/size",
    }
}

pub(super) fn expect_cidr(
    value: RuntimeValue,
    function: &'static str,
    argument: usize,
) -> EvalResult<IpNet> {
    match value {
        RuntimeValue::String(value) => value.parse().map_err(|_| {
            EvalError::conversion(
                function,
                argument,
                "Cidr",
                value,
                "is not a valid IPv4 or IPv6 network",
            )
        }),
        value => Err(EvalError::conversion(
            function,
            argument,
            "Cidr",
            value.render(),
            format!("has type {}", value.type_name()),
        )),
    }
}

pub(super) fn is_private_ipv4(ip: std::net::Ipv4Addr) -> bool {
    let [first, second, ..] = ip.octets();
    first == 10 || (first == 172 && (16..=31).contains(&second)) || (first == 192 && second == 168)
}

pub(super) fn matches_ip_class(ip: IpAddr, kind: &IpClass) -> bool {
    match kind {
        IpClass::V4 => ip.is_ipv4(),
        IpClass::V6 => ip.is_ipv6(),
        IpClass::Private => match ip {
            IpAddr::V4(ip) => is_private_ipv4(ip),
            IpAddr::V6(ip) => ip.segments()[0] & 0xfe00 == 0xfc00,
        },
        IpClass::Loopback => match ip {
            IpAddr::V4(ip) => ip.octets()[0] == 127,
            IpAddr::V6(ip) => ip == std::net::Ipv6Addr::LOCALHOST,
        },
        IpClass::LinkLocal => match ip {
            IpAddr::V4(ip) => matches!(ip.octets(), [169, 254, _, _]),
            IpAddr::V6(ip) => ip.segments()[0] & 0xffc0 == 0xfe80,
        },
        IpClass::Multicast => match ip {
            IpAddr::V4(ip) => (224..=239).contains(&ip.octets()[0]),
            IpAddr::V6(ip) => ip.octets()[0] == 0xff,
        },
    }
}

pub(super) fn ip_class_name(kind: &IpClass) -> &'static str {
    match kind {
        IpClass::V4 => "ip/v4?",
        IpClass::V6 => "ip/v6?",
        IpClass::Private => "ip/private?",
        IpClass::Loopback => "ip/loopback?",
        IpClass::LinkLocal => "ip/link-local?",
        IpClass::Multicast => "ip/multicast?",
    }
}
