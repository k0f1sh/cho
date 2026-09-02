use std::io::Cursor;

use crate::language::{
    Cardinality, DOCUMENTED_CALLABLES, DocumentationCategory, Parameter, ValueType,
};

pub(crate) fn render(topic: &str) -> Option<String> {
    let callable = DOCUMENTED_CALLABLES.iter().copied().find(|callable| {
        let definition = callable.definition();
        definition.name == topic || definition.aliases.contains(&topic)
    })?;
    let definition = callable.definition();
    let documentation = callable.to_doc();
    let syntaxes = definition
        .signatures
        .iter()
        .map(|signature| render_signature(definition.name, signature.parameters, signature.returns))
        .collect::<Vec<_>>();
    let signature_width = syntaxes.iter().map(String::len).max().unwrap_or(0) + 2;

    let mut output = format!("{} — {}\n", definition.name, documentation.summary);
    if !definition.aliases.is_empty() {
        output.push_str(&format!("\nAliases: {}\n", definition.aliases.join(", ")));
    }
    output.push_str("\nSignatures:\n");
    for (syntax, signature_documentation) in syntaxes.iter().zip(documentation.signatures) {
        output.push_str("  ");
        output.push_str(syntax);
        if let Some(summary) = signature_documentation.summary {
            output.push_str(&" ".repeat(signature_width.saturating_sub(syntax.len())));
            output.push_str(summary);
        }
        output.push('\n');
    }
    output.push_str("\nExamples:\n");
    for (index, signature) in documentation.signatures.iter().enumerate() {
        let input = signature
            .input
            .or_else(|| example_input(definition.name, documentation.category, index));
        output.push_str("  ");
        if let Some(input) = input {
            output.push_str("echo ");
            output.push_str(&shell_word(input));
            output.push_str(" | ");
        } else {
            output.push_str("cho --no-input ");
        }
        if input.is_some() {
            output.push_str("cho ");
        }
        output.push_str(&shell_quote(signature.example));
        let evaluated_output;
        let expected_output = if let Some(expected_output) = signature.expected_output {
            expected_output
        } else {
            evaluated_output = evaluate_example(signature.example, input);
            &evaluated_output
        };
        output.push_str("  # => ");
        output.push_str(if expected_output.is_empty() {
            "<empty>"
        } else {
            expected_output
        });
        output.push('\n');
    }
    if !documentation.notes.is_empty() {
        output.push_str("\nNotes:\n");
        for note in documentation.notes {
            output.push_str("  ");
            output.push_str(note);
            output.push('\n');
        }
    }
    Some(output.trim_end().to_owned())
}

fn evaluate_example(program: &str, input: Option<&str>) -> String {
    let mut output = Vec::new();
    let result = match input {
        Some(input) => crate::run(program, Cursor::new(format!("{input}\n")), &mut output),
        None => crate::run_no_input(program, &mut output),
    };
    result.unwrap_or_else(|error| panic!("documented example failed: {program}: {error}"));
    String::from_utf8(output)
        .expect("documented examples emit UTF-8")
        .trim_end_matches('\n')
        .to_owned()
}

fn example_input(
    name: &str,
    category: DocumentationCategory,
    signature: usize,
) -> Option<&'static str> {
    let example = DOCUMENTED_CALLABLES
        .iter()
        .find(|callable| callable.definition().name == name)?
        .to_doc()
        .signatures[signature]
        .example;
    let uses_record = example.contains('$')
        || matches!(
            category,
            DocumentationCategory::Program | DocumentationCategory::Field
        )
        || name == "reg";
    if !uses_record {
        return None;
    }

    Some(match (name, signature) {
        ("print", _) | ("field", _) | ("fields", _) | ("fields-from", _) | ("fields-to", _) => {
            "alice 30 active"
        }
        ("filter", _) => "alice 30",
        ("str", _) | ("s/join", _) => "api gateway",
        ("s/repeat", _) => "go",
        ("s/replace", _) | ("s/replace-all", _) => "api-server-prod",
        ("s/part", _) => "user:alice:admin",
        ("s/before", _) | ("s/after", _) => "status=active",
        ("s/slice", _) => "production",
        ("s/lpad", _) => "42",
        ("s/rpad", _) => "api",
        ("s/count", _) => "café",
        ("s/empty?", _) => "",
        ("s/escape", _) => r"hello\tworld",
        ("s/dquote", _) | ("s/squote", _) => "hello world",
        ("s/unquote", _) => r#""hello""#,
        ("s/lower", _) => "Production",
        ("s/upper", _) => "production",
        ("s/reverse", _) => "stressed",
        ("s/trim", 0) => "  hello  ",
        ("s/trim", 1) => "[draft]",
        ("s/ltrim", 0) => "  hello",
        ("s/ltrim", 1) => "v1.2.3",
        ("s/rtrim", 0) => "report  ",
        ("s/rtrim", 1) => "95%",
        ("s/starts-with?", _) => "api-gateway",
        ("s/ends-with?", _) => "application.log",
        ("s/contains?", _) => "request error",
        ("path/name", _) | ("path/stem", _) | ("path/ext", _) | ("path/dir", _) => {
            "/var/log/application.log"
        }
        ("if", _) | ("and", _) => "5",
        ("default", _) => "alice active",
        ("or", _) => "1",
        ("reg", 0) => "WARN request timed out",
        ("reg", 1) => "api-gateway",
        ("re/replace", _) | ("re/replace-all", _) => "order-123-item-45",
        ("re/part", _) => "alpha,beta:gamma",
        ("cidr/contains?", _) => "10.20.30.40",
        ("cidr/network", _)
        | ("cidr/prefix", _)
        | ("cidr/first", _)
        | ("cidr/last", _)
        | ("cidr/size", _) => "10.20.30.40/24",
        ("url/encode", _) => "hello world",
        ("url/decode", _) => "hello%20world",
        ("url/scheme", _)
        | ("url/host", _)
        | ("url/port", _)
        | ("url/path", _)
        | ("url/query", _)
        | ("url/fragment", _)
        | ("url/query-get", _)
        | ("url/query-has?", _) => "https://api.example.com:8443/users?page=2#results",
        ("uuid", _) | ("uuid/version", _) | ("uuid/time", _) => {
            "01890f3e-7b2c-7cc0-98c4-dc0c0c07398f"
        }
        ("uuid/>", _)
        | ("uuid/>=", _)
        | ("uuid/<", _)
        | ("uuid/<=", _)
        | ("uuid/=", _)
        | ("uuid/!=", _) => {
            "01890f3e-7b2c-7cc0-98c4-dc0c0c07398f 01890f3e-7b2c-7cc0-98c4-dc0c0c073990"
        }
        ("ulid", _) | ("ulid/time", _) => "01H7YAT00Z0000000000000000",
        ("ulid/>", _)
        | ("ulid/>=", _)
        | ("ulid/<", _)
        | ("ulid/<=", _)
        | ("ulid/=", _)
        | ("ulid/!=", _) => "01H7YAT00Z0000000000000000 01H7YAT0100000000000000000",
        ("->", _) | ("->>", _) => "  production  ",
        _ => match category {
            DocumentationCategory::Number => {
                if example.contains("$2") {
                    "12 3"
                } else {
                    "12"
                }
            }
            DocumentationCategory::Boolean | DocumentationCategory::SpecialForm => {
                if example.contains("$2") { "12 3" } else { "12" }
            }
            DocumentationCategory::DateTime => {
                if example.contains("$2") {
                    "2024-01-02T03:04:05+00:00 2024-01-01T03:04:05+00:00"
                } else {
                    "2024-01-02T03:04:05+00:00"
                }
            }
            DocumentationCategory::Network => "10.20.30.40",
            DocumentationCategory::SemanticVersion => {
                if example.contains("$2") {
                    "2.1.0 2.0.0"
                } else {
                    "2.1.0"
                }
            }
            _ => panic!("documented example with fields has no input: {name}"),
        },
    })
}

pub(crate) fn apropos(query: &str) -> Option<String> {
    if query.is_empty() {
        return None;
    }
    let query = query.to_ascii_lowercase();
    render_apropos(
        DOCUMENTED_CALLABLES
            .iter()
            .copied()
            .filter(|callable| {
                let definition = callable.definition();
                std::iter::once(definition.name)
                    .chain(definition.aliases.iter().copied())
                    .any(|name| name.to_ascii_lowercase().contains(&query))
            })
            .collect::<Vec<_>>(),
    )
}

pub(crate) fn catalog() -> String {
    render_apropos(DOCUMENTED_CALLABLES.to_vec()).expect("the callable registry is not empty")
}

fn render_apropos(matches: Vec<&'static dyn crate::language::ToDoc>) -> Option<String> {
    if matches.is_empty() {
        return None;
    }
    let labels = matches
        .iter()
        .map(|callable| {
            let definition = callable.definition();
            if definition.aliases.is_empty() {
                definition.name.to_owned()
            } else {
                format!("{} ({})", definition.name, definition.aliases.join(", "))
            }
        })
        .collect::<Vec<_>>();
    let description_column = labels.iter().map(String::len).max().unwrap_or(0) + 2;
    let mut output = String::new();
    for (callable, label) in matches.iter().zip(labels) {
        output.push_str(&label);
        output.push_str(&" ".repeat(description_column.saturating_sub(label.len())));
        output.push_str(callable.to_doc().summary);
        output.push('\n');
    }
    Some(output.trim_end().to_owned())
}

fn render_signature(name: &str, parameters: &[Parameter], returns: Option<ValueType>) -> String {
    let mut syntax = format!("({name}");
    for parameter in parameters {
        syntax.push(' ');
        let label = parameter
            .help_label
            .unwrap_or_else(|| help_name(parameter.value_type));
        match parameter.cardinality {
            Cardinality::Required => syntax.push_str(label),
            Cardinality::Optional => syntax.push_str(&format!("[{label}]")),
            Cardinality::ZeroOrMore => syntax.push_str(&format!("{label} ...")),
            Cardinality::OneOrMore => syntax.push_str(&format!("{label} {label} ...")),
        }
    }
    syntax.push(')');
    if let Some(returns) = returns {
        syntax.push_str(" -> ");
        syntax.push_str(help_name(returns));
    }
    syntax
}

fn help_name(value_type: ValueType) -> &'static str {
    match value_type {
        ValueType::Value => "VALUE",
        ValueType::String => "STRING",
        ValueType::Number => "NUMBER",
        ValueType::Boolean => "BOOLEAN",
        ValueType::DateTime => "DATETIME",
        ValueType::Duration => "DURATION",
        ValueType::IpAddr => "IPADDR",
        ValueType::Cidr => "CIDR",
        ValueType::Url => "URL",
        ValueType::SemVer => "SEMVER",
        ValueType::Uuid => "UUID",
        ValueType::Ulid => "ULID",
        ValueType::Regex => "/PATTERN/",
        ValueType::Step => "STEP",
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn shell_word(value: &str) -> String {
    if !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'/'))
    {
        value.to_owned()
    } else {
        shell_quote(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_signatures_examples_notes_and_aliases() {
        let trim = render("s/trim").unwrap();
        assert!(trim.starts_with("s/trim — trim whitespace or exact affixes\n"));
        assert!(trim.contains("(s/trim VALUE) -> STRING"));
        assert!(trim.contains("remove Unicode whitespace from both ends"));
        assert!(trim.contains("cho '(s/trim $1 \"[\" \"]\")'"));
        assert!(trim.contains("\nNotes:\n"));

        let canonical = render("s/dquote").unwrap();
        assert_eq!(render("dq"), Some(canonical.clone()));
        assert!(canonical.contains("\nAliases: dq\n"));
    }

    #[test]
    fn renders_examples_with_input_and_expected_output() {
        assert!(
            render("s/starts-with?")
                .unwrap()
                .contains("echo api-gateway | cho '(s/starts-with? $1 \"api-\")'  # => true")
        );
    }

    #[test]
    fn renders_all_callable_kinds_and_rejects_unknown_topics() {
        for topic in ["print", "if", "->", "s/upper"] {
            assert!(render(topic).is_some(), "missing help for {topic}");
        }
        for callable in DOCUMENTED_CALLABLES {
            let definition = callable.definition();
            let canonical = render(definition.name).unwrap();
            for alias in definition.aliases {
                assert_eq!(render(alias), Some(canonical.clone()));
            }
        }
        assert_eq!(render("s/not-a-function"), None);
    }

    #[test]
    fn every_signature_has_a_runnable_example_with_a_result() {
        for callable in DOCUMENTED_CALLABLES {
            let definition = callable.definition();
            let rendered = render(definition.name).unwrap();
            let examples = rendered.split_once("\nExamples:\n").unwrap().1;
            let examples = examples.split("\n\nNotes:\n").next().unwrap();
            let lines = examples.lines().collect::<Vec<_>>();
            assert_eq!(
                lines.len(),
                definition.signatures.len(),
                "{}",
                definition.name
            );
            for line in lines {
                assert!(
                    line.starts_with("  echo ") || line.starts_with("  cho --no-input "),
                    "example is not directly runnable: {line}"
                );
                assert!(line.contains("  # => "), "example has no result: {line}");
            }
        }
    }

    #[test]
    fn shell_quotes_examples_containing_single_quotes() {
        assert!(
            render("shq")
                .unwrap()
                .contains(r#"cho --no-input '(shq "it'\''s good")'"#)
        );
    }

    #[test]
    fn apropos_searches_names_and_aliases_case_insensitively_in_registry_order() {
        assert_eq!(
            apropos("QUOTE").unwrap(),
            concat!(
                "s/dquote (dq)  stringify and wrap in escaped double quotes\n",
                "s/squote (sq)  stringify and wrap in escaped single quotes\n",
                "s/unquote      remove matching quotes and decode backslash escapes",
            )
        );
        assert!(apropos("dq").unwrap().starts_with("s/dquote (dq)"));
    }

    #[test]
    fn apropos_does_not_search_descriptions_and_rejects_empty_or_missing_matches() {
        assert_eq!(apropos("fractional digits"), None);
        assert_eq!(apropos(""), None);
        assert_eq!(apropos("not-a-callable"), None);
    }
}
