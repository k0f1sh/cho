use std::collections::BTreeSet;

use serde::Serialize;

use crate::language::{
    CallableKind as LanguageCallableKind, Cardinality as LanguageCardinality, DOCUMENTED_CALLABLES,
    DocumentationCategory, ValueType as LanguageValueType,
};

#[derive(Debug, Serialize)]
pub struct Metadata {
    pub schema_version: u32,
    pub cho_version: String,
    pub callables: Vec<Callable>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Program,
    Field,
    Number,
    String,
    Path,
    Boolean,
    SpecialForm,
    RegularExpression,
    DateTime,
    Network,
    Url,
    SemanticVersion,
    Composition,
}

impl Category {
    const ALL: [Self; 13] = [
        Self::Program,
        Self::Field,
        Self::Number,
        Self::String,
        Self::Path,
        Self::Boolean,
        Self::SpecialForm,
        Self::RegularExpression,
        Self::DateTime,
        Self::Network,
        Self::Url,
        Self::SemanticVersion,
        Self::Composition,
    ];

    fn marker(self) -> &'static str {
        match self {
            Self::Program => "program",
            Self::Field => "field",
            Self::Number => "number",
            Self::String => "string",
            Self::Path => "path",
            Self::Boolean => "boolean",
            Self::SpecialForm => "special_form",
            Self::RegularExpression => "regular_expression",
            Self::DateTime => "date_time",
            Self::Network => "network",
            Self::Url => "url",
            Self::SemanticVersion => "semantic_version",
            Self::Composition => "composition",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CallableKind {
    ProgramForm,
    Function,
    SpecialForm,
    ThreadingForm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum ValueType {
    Value,
    String,
    Number,
    Boolean,
    DateTime,
    Duration,
    IpAddr,
    Cidr,
    Url,
    SemVer,
    Regex,
    Step,
}

impl ValueType {
    fn help_name(self) -> &'static str {
        match self {
            Self::Value => "VALUE",
            Self::String => "STRING",
            Self::Number => "NUMBER",
            Self::Boolean => "BOOLEAN",
            Self::DateTime => "DATETIME",
            Self::Duration => "DURATION",
            Self::IpAddr => "IPADDR",
            Self::Cidr => "CIDR",
            Self::Url => "URL",
            Self::SemVer => "SEMVER",
            Self::Regex => "/PATTERN/",
            Self::Step => "STEP",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Cardinality {
    Required,
    Optional,
    ZeroOrMore,
    OneOrMore,
}

#[derive(Debug, Serialize)]
pub struct Callable {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub kind: CallableKind,
    pub category: Category,
    pub summary: String,
    pub notes: Vec<String>,
    pub signatures: Vec<Signature>,
}

#[derive(Debug, Serialize)]
pub struct Signature {
    pub parameters: Vec<Parameter>,
    pub returns: Option<ValueType>,
    pub summary: Option<String>,
    pub example: String,
}

#[derive(Debug, Serialize)]
pub struct Parameter {
    pub name: &'static str,
    #[serde(rename = "type")]
    pub value_type: ValueType,
    pub cardinality: Cardinality,
    #[serde(skip)]
    pub help_label: Option<&'static str>,
}

pub fn metadata(version: &str) -> Result<Metadata, String> {
    let mut callables = Vec::with_capacity(DOCUMENTED_CALLABLES.len());

    for callable in DOCUMENTED_CALLABLES {
        let definition = callable.definition();
        let docs = callable.to_doc();
        if docs.signatures.len() != definition.signatures.len() {
            return Err(format!(
                "{} has {} signatures but {} documented examples",
                definition.name,
                definition.signatures.len(),
                docs.signatures.len()
            ));
        }
        let signatures = definition
            .signatures
            .iter()
            .zip(docs.signatures.iter())
            .map(|(signature, docs)| Signature {
                parameters: signature
                    .parameters
                    .iter()
                    .map(|parameter| Parameter {
                        name: parameter.name,
                        value_type: parameter.value_type.into(),
                        cardinality: parameter.cardinality.into(),
                        help_label: parameter.help_label,
                    })
                    .collect(),
                returns: signature.returns.map(Into::into),
                summary: docs.summary.map(str::to_owned),
                example: docs.example.to_owned(),
            })
            .collect();
        callables.push(Callable {
            name: definition.name,
            aliases: definition.aliases,
            kind: definition.kind.into(),
            category: docs.category.into(),
            summary: docs.summary.to_owned(),
            notes: docs.notes.iter().map(|note| (*note).to_owned()).collect(),
            signatures,
        });
    }

    Ok(Metadata {
        schema_version: 1,
        cho_version: version.to_owned(),
        callables,
    })
}

impl From<DocumentationCategory> for Category {
    fn from(value: DocumentationCategory) -> Self {
        match value {
            DocumentationCategory::Program => Self::Program,
            DocumentationCategory::Field => Self::Field,
            DocumentationCategory::Number => Self::Number,
            DocumentationCategory::String => Self::String,
            DocumentationCategory::Path => Self::Path,
            DocumentationCategory::Boolean => Self::Boolean,
            DocumentationCategory::SpecialForm => Self::SpecialForm,
            DocumentationCategory::RegularExpression => Self::RegularExpression,
            DocumentationCategory::DateTime => Self::DateTime,
            DocumentationCategory::Network => Self::Network,
            DocumentationCategory::Url => Self::Url,
            DocumentationCategory::SemanticVersion => Self::SemanticVersion,
            DocumentationCategory::Composition => Self::Composition,
        }
    }
}

pub fn render_help(template: &str, metadata: &Metadata) -> Result<String, String> {
    let mut output = template.to_owned();
    for category in Category::ALL {
        let marker = format!("{{{{callables:{}}}}}", category.marker());
        if output.matches(&marker).count() != 1 {
            return Err(format!("help template must contain {marker} exactly once"));
        }
        output = output.replacen(&marker, &render_category(metadata, category), 1);
    }
    if output.contains("{{callables:") {
        return Err("help template contains an unknown callable marker".to_owned());
    }
    Ok(output)
}

fn signature_width(metadata: &Metadata, category: Category) -> usize {
    metadata
        .callables
        .iter()
        .filter(|callable| callable.category == category)
        .flat_map(|callable| {
            std::iter::once(callable.name)
                .chain(callable.aliases.iter().copied())
                .flat_map(|name| {
                    callable
                        .signatures
                        .iter()
                        .map(move |signature| render_signature_syntax(name, signature).len())
                })
        })
        .max()
        .unwrap_or(0)
        .saturating_add(2)
        .max(42)
}

fn render_category(metadata: &Metadata, category: Category) -> String {
    let mut output = String::new();
    let signature_width = signature_width(metadata, category);
    for callable in metadata
        .callables
        .iter()
        .filter(|callable| callable.category == category)
    {
        for signature in &callable.signatures {
            render_signature(
                &mut output,
                callable.name,
                signature,
                &callable.summary,
                signature_width,
            );
        }
        for alias in callable.aliases {
            for signature in &callable.signatures {
                render_signature(
                    &mut output,
                    alias,
                    signature,
                    &format!("short form of {}", callable.name),
                    signature_width,
                );
            }
        }
    }
    output.trim_end().to_owned()
}

fn render_signature(
    output: &mut String,
    name: &str,
    signature: &Signature,
    fallback: &str,
    signature_width: usize,
) {
    let syntax = render_signature_syntax(name, signature);
    output.push_str("  ");
    output.push_str(&syntax);
    output.push_str(&" ".repeat(signature_width.saturating_sub(syntax.len()).max(2)));
    output.push_str(signature.summary.as_deref().unwrap_or(fallback));
    output.push('\n');
}

fn render_signature_syntax(name: &str, signature: &Signature) -> String {
    let mut syntax = format!("({name}");
    for parameter in &signature.parameters {
        syntax.push(' ');
        let label = help_label(parameter);
        match parameter.cardinality {
            Cardinality::Required => syntax.push_str(&label),
            Cardinality::Optional => syntax.push_str(&format!("[{label}]")),
            Cardinality::ZeroOrMore => syntax.push_str(&format!("{label} ...")),
            Cardinality::OneOrMore => syntax.push_str(&format!("{label} {label} ...")),
        }
    }
    syntax.push(')');
    if let Some(returns) = signature.returns {
        syntax.push_str(" -> ");
        syntax.push_str(returns.help_name());
    }
    syntax
}

fn help_label(parameter: &Parameter) -> String {
    parameter
        .help_label
        .map(str::to_owned)
        .unwrap_or_else(|| parameter.value_type.help_name().to_owned())
}

pub fn validate(metadata: &Metadata) -> Result<(), String> {
    if metadata.schema_version != 1 || metadata.cho_version.is_empty() {
        return Err("metadata requires schema_version 1 and a cho_version".to_owned());
    }
    let mut names = BTreeSet::new();
    for callable in &metadata.callables {
        if callable.name.is_empty() || callable.summary.is_empty() || callable.signatures.is_empty()
        {
            return Err(format!("{} has incomplete documentation", callable.name));
        }
        for name in std::iter::once(callable.name).chain(callable.aliases.iter().copied()) {
            if !names.insert(name) {
                return Err(format!("duplicate callable name or alias: {name}"));
            }
        }
        for signature in &callable.signatures {
            if signature.example.is_empty() {
                return Err(format!(
                    "{} has a signature without an example",
                    callable.name
                ));
            }
        }
    }
    Ok(())
}

impl From<LanguageCallableKind> for CallableKind {
    fn from(value: LanguageCallableKind) -> Self {
        match value {
            LanguageCallableKind::ProgramForm => Self::ProgramForm,
            LanguageCallableKind::Function => Self::Function,
            LanguageCallableKind::SpecialForm => Self::SpecialForm,
            LanguageCallableKind::ThreadingForm => Self::ThreadingForm,
        }
    }
}

impl From<LanguageValueType> for ValueType {
    fn from(value: LanguageValueType) -> Self {
        match value {
            LanguageValueType::Value => Self::Value,
            LanguageValueType::String => Self::String,
            LanguageValueType::Number => Self::Number,
            LanguageValueType::Boolean => Self::Boolean,
            LanguageValueType::DateTime => Self::DateTime,
            LanguageValueType::Duration => Self::Duration,
            LanguageValueType::IpAddr => Self::IpAddr,
            LanguageValueType::Cidr => Self::Cidr,
            LanguageValueType::Url => Self::Url,
            LanguageValueType::SemVer => Self::SemVer,
            LanguageValueType::Regex => Self::Regex,
            LanguageValueType::Step => Self::Step,
        }
    }
}

impl From<LanguageCardinality> for Cardinality {
    fn from(value: LanguageCardinality) -> Self {
        match value {
            LanguageCardinality::Required => Self::Required,
            LanguageCardinality::Optional => Self::Optional,
            LanguageCardinality::ZeroOrMore => Self::ZeroOrMore,
            LanguageCardinality::OneOrMore => Self::OneOrMore,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generated_metadata() -> String {
        let metadata = metadata(env!("CARGO_PKG_VERSION")).unwrap();
        validate(&metadata).unwrap();
        format!("{}\n", serde_json::to_string_pretty(&metadata).unwrap())
    }

    #[test]
    fn checked_in_documentation_matches_the_callable_types() {
        let metadata = metadata(env!("CARGO_PKG_VERSION")).unwrap();
        let help = render_help(include_str!("help.txt.in"), &metadata).unwrap();
        assert_eq!(include_str!("../metadata.json"), generated_metadata());
        assert_eq!(include_str!("help.txt"), help.trim_end());
    }

    #[test]
    fn every_documented_signature_and_alias_parses() {
        let metadata = metadata(env!("CARGO_PKG_VERSION")).unwrap();
        for callable in &metadata.callables {
            for signature in &callable.signatures {
                for name in std::iter::once(callable.name).chain(callable.aliases.iter().copied()) {
                    let invocation = signature.example.replacen(
                        &format!("({}", callable.name),
                        &format!("({name}"),
                        1,
                    );
                    let program = if callable.kind == CallableKind::ProgramForm {
                        invocation
                    } else {
                        format!("(print {invocation})")
                    };
                    crate::parse(&program).unwrap_or_else(|error| {
                        panic!("documented example for {name} does not parse: {program}: {error:?}")
                    });
                }
            }
        }
    }

    #[test]
    fn help_lists_every_registered_name() {
        let metadata = metadata(env!("CARGO_PKG_VERSION")).unwrap();
        let help = render_help(include_str!("help.txt.in"), &metadata).unwrap();
        for name in metadata.callables.iter().flat_map(|callable| {
            std::iter::once(callable.name).chain(callable.aliases.iter().copied())
        }) {
            assert!(help.contains(&format!("({name}")), "help omits {name}");
        }
    }

    #[test]
    fn help_uses_field_range_bound_names() {
        let metadata = metadata(env!("CARGO_PKG_VERSION")).unwrap();
        let help = render_help(include_str!("help.txt.in"), &metadata).unwrap();
        assert!(help.contains("(fields START END) -> STRING"));
        assert!(help.contains("(fields-to END) -> STRING"));
    }

    #[test]
    fn callable_descriptions_share_one_column() {
        let metadata = metadata(env!("CARGO_PKG_VERSION")).unwrap();
        let help = render_help(include_str!("help.txt.in"), &metadata).unwrap();
        for category in Category::ALL {
            let description_column = signature_width(&metadata, category) + 2;
            for callable in metadata
                .callables
                .iter()
                .filter(|callable| callable.category == category)
            {
                let names = std::iter::once((callable.name, callable.summary.clone()))
                    .chain(
                        callable
                            .aliases
                            .iter()
                            .map(|alias| (*alias, format!("short form of {}", callable.name))),
                    )
                    .collect::<Vec<_>>();
                for (name, fallback) in names {
                    for signature in &callable.signatures {
                        let syntax = render_signature_syntax(name, signature);
                        let prefix = format!("  {syntax}");
                        let line = help
                            .lines()
                            .find(|line| line.starts_with(&prefix))
                            .unwrap_or_else(|| panic!("help omits signature {syntax}"));
                        let summary = signature.summary.as_deref().unwrap_or(&fallback);
                        let actual_column = line
                            .strip_suffix(summary)
                            .map(str::len)
                            .unwrap_or_else(|| panic!("help omits summary for {syntax}"));
                        assert_eq!(actual_column, description_column, "{syntax}");
                    }
                }
            }
        }
    }
}
