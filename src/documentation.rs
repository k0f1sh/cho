use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::language::{
    CALLABLES, CallableKind as LanguageCallableKind, Cardinality as LanguageCardinality,
    ValueType as LanguageValueType,
};

#[derive(Debug, Serialize)]
pub struct Metadata {
    pub schema_version: u32,
    pub cho_version: String,
    pub callables: Vec<Callable>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Program,
    Number,
    String,
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
    const ALL: [Self; 11] = [
        Self::Program,
        Self::Number,
        Self::String,
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
            Self::Number => "number",
            Self::String => "string",
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
}

#[derive(Debug, Deserialize)]
struct Documentation {
    callables: Vec<CallableDocumentation>,
}

#[derive(Debug, Deserialize)]
struct CallableDocumentation {
    name: String,
    category: Category,
    summary: String,
    notes: Vec<String>,
    signatures: Vec<SignatureDocumentation>,
}

#[derive(Debug, Deserialize)]
struct SignatureDocumentation {
    summary: Option<String>,
    example: String,
}

pub fn metadata(version: &str) -> Result<Metadata, String> {
    let documentation: Documentation = serde_json::from_str(include_str!("documentation.json"))
        .map_err(|error| format!("invalid documentation.json: {error}"))?;
    let mut documentation_by_name = BTreeMap::new();
    for callable in documentation.callables {
        let name = callable.name.clone();
        if documentation_by_name
            .insert(name.clone(), callable)
            .is_some()
        {
            return Err(format!("duplicate documentation for {name}"));
        }
    }
    let mut callables = Vec::with_capacity(CALLABLES.len());

    for callable in CALLABLES {
        let docs = documentation_by_name
            .remove(callable.name)
            .ok_or_else(|| format!("missing documentation for {}", callable.name))?;
        if docs.signatures.len() != callable.signatures.len() {
            return Err(format!(
                "{} has {} signatures but {} documented examples",
                callable.name,
                callable.signatures.len(),
                docs.signatures.len()
            ));
        }
        let signatures = callable
            .signatures
            .iter()
            .zip(docs.signatures)
            .map(|(signature, docs)| Signature {
                parameters: signature
                    .parameters
                    .iter()
                    .map(|parameter| Parameter {
                        name: parameter.name,
                        value_type: parameter.value_type.into(),
                        cardinality: parameter.cardinality.into(),
                    })
                    .collect(),
                returns: signature.returns.map(Into::into),
                summary: docs.summary,
                example: docs.example,
            })
            .collect();
        callables.push(Callable {
            name: callable.name,
            aliases: callable.aliases,
            kind: callable.kind.into(),
            category: docs.category,
            summary: docs.summary,
            notes: docs.notes,
            signatures,
        });
    }
    if let Some(name) = documentation_by_name.keys().next() {
        return Err(format!("documentation exists for unknown callable: {name}"));
    }

    Ok(Metadata {
        schema_version: 1,
        cho_version: version.to_owned(),
        callables,
    })
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

fn render_category(metadata: &Metadata, category: Category) -> String {
    let mut output = String::new();
    for callable in metadata
        .callables
        .iter()
        .filter(|callable| callable.category == category)
    {
        for signature in &callable.signatures {
            render_signature(&mut output, callable.name, signature, &callable.summary);
        }
        for alias in callable.aliases {
            for signature in &callable.signatures {
                render_signature(
                    &mut output,
                    alias,
                    signature,
                    &format!("short form of {}", callable.name),
                );
            }
        }
    }
    output.trim_end().to_owned()
}

fn render_signature(output: &mut String, name: &str, signature: &Signature, fallback: &str) {
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
    output.push_str("  ");
    output.push_str(&syntax);
    output.push_str(&" ".repeat(42_usize.saturating_sub(syntax.len()).max(2)));
    output.push_str(signature.summary.as_deref().unwrap_or(fallback));
    output.push('\n');
}

fn help_label(parameter: &Parameter) -> String {
    match parameter.name {
        "pattern" => "/PATTERN/".to_owned(),
        "digits" | "separator" | "from" | "to" | "delimiter" | "position" | "start" | "length"
        | "prefix" | "suffix" | "needle" | "fallback" | "replacement" | "timezone" => {
            parameter.name.to_ascii_uppercase()
        }
        _ => parameter.value_type.help_name().to_owned(),
    }
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
