use std::io::Cursor;

pub(crate) fn output(program: &str, input: &str) -> String {
    let mut result = Vec::new();
    cho::run(program, Cursor::new(input), &mut result).unwrap();
    String::from_utf8(result).unwrap()
}

pub(crate) fn output_with_separator(program: &str, separator: &str, input: &str) -> String {
    let mut result = Vec::new();
    cho::run_with_field_separator(program, Some(separator), Cursor::new(input), &mut result)
        .unwrap();
    String::from_utf8(result).unwrap()
}
