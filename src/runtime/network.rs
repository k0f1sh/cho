use std::net::IpAddr;

use ipnet::IpNet;

use crate::ast::{CidrPart, IpClass, IpFormat};

use super::value::{EvalError, EvalResult, RuntimeValue, exact_u64_number, expect_number};

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
        CidrPart::SizeString => "cidr/size-str",
        CidrPart::Netmask => "cidr/netmask",
        CidrPart::Wildcard => "cidr/wildcard",
        CidrPart::HostFirst => "cidr/host-first",
        CidrPart::HostLast => "cidr/host-last",
        CidrPart::HostCount => "cidr/host-count",
    }
}

pub(super) fn expect_cidr(
    value: RuntimeValue,
    function: &'static str,
    argument: usize,
) -> EvalResult<IpNet> {
    match value {
        RuntimeValue::Cidr(value) => Ok(value),
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

pub(super) fn ip_format_name(format: &IpFormat) -> &'static str {
    match format {
        IpFormat::Expanded => "ip/expand",
        IpFormat::Binary => "ip/binary",
    }
}

pub(super) fn format_ip(ip: IpAddr, format: &IpFormat) -> String {
    match (ip, format) {
        (IpAddr::V4(ip), IpFormat::Expanded) => ip.to_string(),
        (IpAddr::V6(ip), IpFormat::Expanded) => ip
            .segments()
            .map(|segment| format!("{segment:04x}"))
            .join(":"),
        (IpAddr::V4(ip), IpFormat::Binary) => {
            ip.octets().map(|octet| format!("{octet:08b}")).join(".")
        }
        (IpAddr::V6(ip), IpFormat::Binary) => ip
            .segments()
            .map(|segment| format!("{segment:016b}"))
            .join(":"),
    }
}

pub(super) fn make_cidr(ip: IpAddr, prefix: RuntimeValue) -> EvalResult<RuntimeValue> {
    let prefix = expect_number(prefix, "cidr", 2)?;
    let max = if ip.is_ipv4() { 32 } else { 128 };
    if prefix.fract() != 0.0 || !(0.0..=max as f64).contains(&prefix) {
        return Err(EvalError::conversion(
            "cidr",
            2,
            "Number prefix length",
            prefix.to_string(),
            format!("must be an integer from 0 to {max}"),
        ));
    }
    // The address-family-specific prefix bound was checked above.
    Ok(RuntimeValue::Cidr(
        IpNet::new(ip, prefix as u8).unwrap().trunc(),
    ))
}

pub(super) fn cidr_from_mask(ip: IpAddr, mask: IpAddr) -> EvalResult<RuntimeValue> {
    let prefix = match (ip, mask) {
        (IpAddr::V4(_), IpAddr::V4(mask)) => {
            let bits = u32::from(mask);
            (bits.leading_ones() + bits.trailing_zeros() == 32).then_some(bits.leading_ones() as u8)
        }
        (IpAddr::V6(_), IpAddr::V6(mask)) => {
            let bits = u128::from(mask);
            (bits.leading_ones() + bits.trailing_zeros() == 128)
                .then_some(bits.leading_ones() as u8)
        }
        _ => {
            return Err(EvalError::conversion(
                "cidr/from-mask",
                2,
                "IpAddr mask of the same address family as argument 1",
                mask.to_string(),
                "has a different address family",
            ));
        }
    };
    let prefix = prefix.ok_or_else(|| {
        EvalError::conversion(
            "cidr/from-mask",
            2,
            "IpAddr contiguous netmask",
            mask.to_string(),
            "must contain leading ones followed by zeros",
        )
    })?;
    Ok(RuntimeValue::Cidr(IpNet::new(ip, prefix).unwrap().trunc()))
}

pub(super) fn cidr_part(cidr: IpNet, part: &CidrPart) -> EvalResult<RuntimeValue> {
    let function = cidr_part_name(part);
    let address_bits = if cidr.addr().is_ipv4() { 32 } else { 128 };
    let host_bits = address_bits - cidr.prefix_len();
    let exclude_endpoints = cidr.addr().is_ipv4() && cidr.prefix_len() <= 30;
    match part {
        CidrPart::Network | CidrPart::First => Ok(RuntimeValue::IpAddr(cidr.network())),
        CidrPart::Prefix => Ok(RuntimeValue::Number(cidr.prefix_len() as f64)),
        CidrPart::Last => Ok(RuntimeValue::IpAddr(cidr.broadcast())),
        CidrPart::Netmask => Ok(RuntimeValue::IpAddr(cidr.netmask())),
        CidrPart::Wildcard => Ok(RuntimeValue::IpAddr(cidr.hostmask())),
        CidrPart::HostFirst => {
            let first = match cidr.network() {
                IpAddr::V4(ip) if exclude_endpoints => IpAddr::V4((u32::from(ip) + 1).into()),
                ip => ip,
            };
            Ok(RuntimeValue::IpAddr(first))
        }
        CidrPart::HostLast => {
            let last = match cidr.broadcast() {
                IpAddr::V4(ip) if exclude_endpoints => IpAddr::V4((u32::from(ip) - 1).into()),
                ip => ip,
            };
            Ok(RuntimeValue::IpAddr(last))
        }
        CidrPart::SizeString | CidrPart::HostCount => {
            // IPv6 /0 has 2^128 addresses, one more than u128::MAX.
            let count = if host_bits == 128 {
                "340282366920938463463374607431768211456".to_owned()
            } else {
                let reserved = if matches!(part, CidrPart::HostCount) && exclude_endpoints {
                    2
                } else {
                    0
                };
                ((1_u128 << host_bits) - reserved).to_string()
            };
            Ok(RuntimeValue::String(count))
        }
        CidrPart::Size => {
            if host_bits >= 53 {
                return Err(EvalError::conversion(
                    function,
                    1,
                    "Cidr whose size fits Number's safe integer range",
                    cidr.to_string(),
                    "contains more than 2^53 - 1 addresses",
                ));
            }
            exact_u64_number(1_u64 << host_bits, function, 1, cidr.to_string())
        }
    }
}
