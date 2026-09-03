pub(super) fn join(values: &[String]) -> String {
    if values.len() == 1 && values[0].is_empty() {
        return "\"\"".to_owned();
    }

    let mut record = String::new();
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            record.push(',');
        }
        if value.contains([',', '"', '\r', '\n']) {
            record.push('"');
            for character in value.chars() {
                if character == '"' {
                    record.push('"');
                }
                record.push(character);
            }
            record.push('"');
        } else {
            record.push_str(value);
        }
    }
    record
}
