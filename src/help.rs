use crate::language::{Cardinality, DOCUMENTED_CALLABLES, Parameter, ValueType};

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
    for signature in documentation.signatures {
        output.push_str("  cho ");
        output.push_str(&shell_quote(signature.example));
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
    fn shell_quotes_examples_containing_single_quotes() {
        assert!(
            render("shq")
                .unwrap()
                .contains(r#"cho '(shq "it'\''s good")'"#)
        );
    }
}
