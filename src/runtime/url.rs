use super::*;

pub(super) fn parse_absolute_url(
    input: &str,
    function: &'static str,
    argument: usize,
) -> EvalResult<::url::Url> {
    ::url::Url::parse(input).map_err(|_| {
        EvalError::conversion(
            function,
            argument,
            "Url (absolute URL)",
            input,
            "is not a valid absolute URL",
        )
    })
}

pub(super) fn url_part_name(part: &UrlPart) -> &'static str {
    match part {
        UrlPart::Scheme => "url/scheme",
        UrlPart::Host => "url/host",
        UrlPart::Port => "url/port",
        UrlPart::Path => "url/path",
        UrlPart::Query => "url/query",
        UrlPart::Fragment => "url/fragment",
    }
}

pub(super) fn url_encoding_name(operation: &UrlEncoding) -> &'static str {
    match operation {
        UrlEncoding::Encode => "url/encode",
        UrlEncoding::Decode => "url/decode",
    }
}

pub(super) fn encode_url_component(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push(char::from(HEX[usize::from(byte >> 4)]));
            encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
    }
    encoded
}

pub(super) fn decode_url_component(value: &str) -> Result<String, String> {
    let input = value.as_bytes();
    let mut decoded = Vec::with_capacity(input.len());
    let mut index = 0;
    while index < input.len() {
        if input[index] != b'%' {
            decoded.push(input[index]);
            index += 1;
            continue;
        }
        let high = input
            .get(index + 1)
            .and_then(|byte| hex_value(*byte))
            .ok_or_else(|| format!("contains an invalid percent escape at byte {index}"))?;
        let low = input
            .get(index + 2)
            .and_then(|byte| hex_value(*byte))
            .ok_or_else(|| format!("contains an invalid percent escape at byte {index}"))?;
        decoded.push(high << 4 | low);
        index += 3;
    }
    String::from_utf8(decoded).map_err(|_| "decodes to bytes that are not valid UTF-8".to_owned())
}

pub(super) fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
