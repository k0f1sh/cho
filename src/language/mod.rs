use crate::ast::{Form, RegexId, Value};
use crate::parser::{ParseError, SExpr};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CallableKind {
    ProgramForm,
    Function,
    SpecialForm,
    ThreadingForm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ValueType {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Cardinality {
    Required,
    Optional,
    ZeroOrMore,
    OneOrMore,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Parameter {
    pub(crate) name: &'static str,
    pub(crate) value_type: ValueType,
    pub(crate) cardinality: Cardinality,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Signature {
    pub(crate) parameters: &'static [Parameter],
    pub(crate) returns: Option<ValueType>,
}

impl Signature {
    pub(crate) fn accepts(self, argument_count: usize) -> bool {
        let minimum = self
            .parameters
            .iter()
            .filter(|parameter| {
                matches!(
                    parameter.cardinality,
                    Cardinality::Required | Cardinality::OneOrMore
                )
            })
            .count();
        let maximum = match self.parameters.last() {
            Some(parameter)
                if matches!(
                    parameter.cardinality,
                    Cardinality::ZeroOrMore | Cardinality::OneOrMore
                ) =>
            {
                None
            }
            Some(_) | None => Some(self.parameters.len()),
        };
        argument_count >= minimum && maximum.is_none_or(|maximum| argument_count <= maximum)
    }

    pub(crate) fn parameter(self, index: usize) -> Option<Parameter> {
        self.parameters.get(index).copied().or_else(|| {
            self.parameters.last().copied().filter(|parameter| {
                matches!(
                    parameter.cardinality,
                    Cardinality::ZeroOrMore | Cardinality::OneOrMore
                )
            })
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CallableDefinition {
    pub(crate) name: &'static str,
    pub(crate) aliases: &'static [&'static str],
    pub(crate) kind: CallableKind,
    pub(crate) signatures: &'static [Signature],
}

impl CallableDefinition {
    pub(crate) fn signature(self, argument_count: usize) -> Option<Signature> {
        let mut signatures = self
            .signatures
            .iter()
            .copied()
            .filter(|signature| signature.accepts(argument_count));
        let signature = signatures.next()?;
        signatures.next().is_none().then_some(signature)
    }
}

#[derive(Debug)]
pub(crate) enum BoundArgument<'syntax> {
    Value(Value),
    Regex(RegexId),
    Step(&'syntax SExpr),
}

pub(crate) struct Arguments<'syntax>(pub(crate) Vec<BoundArgument<'syntax>>);

pub(crate) enum CompiledExpression {
    Form(Form),
    Value(Value),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ThreadDirection {
    First,
    Last,
}

pub(crate) trait AstContext {
    fn compile_threading(
        &mut self,
        direction: ThreadDirection,
        arguments: Arguments<'_>,
    ) -> Result<Value, ParseError>;
}

pub(crate) trait ToAst: Sync {
    fn definition(&self) -> &'static CallableDefinition;

    fn to_ast(
        &self,
        context: &mut dyn AstContext,
        arguments: Arguments<'_>,
    ) -> Result<CompiledExpression, ParseError>;
}

#[cfg(any(test, feature = "documentation"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DocumentationCategory {
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

#[cfg(any(test, feature = "documentation"))]
pub(crate) struct SignatureDocumentation {
    pub(crate) summary: Option<&'static str>,
    pub(crate) example: &'static str,
}

#[cfg(any(test, feature = "documentation"))]
pub(crate) struct CallableDocumentation {
    pub(crate) category: DocumentationCategory,
    pub(crate) summary: &'static str,
    pub(crate) notes: &'static [&'static str],
    pub(crate) signatures: &'static [SignatureDocumentation],
}

#[cfg(any(test, feature = "documentation"))]
pub(crate) trait ToDoc: ToAst {
    fn to_doc(&self) -> CallableDocumentation;
}

macro_rules! p {
    ($name:literal, $type:ident, $cardinality:ident) => {
        Parameter {
            name: $name,
            value_type: ValueType::$type,
            cardinality: Cardinality::$cardinality,
        }
    };
}

macro_rules! sig {
    ([$($parameter:expr),* $(,)?] => $returns:expr) => {
        Signature {
            parameters: &[$($parameter),*],
            returns: $returns,
        }
    };
}

macro_rules! define_callable {
    (
        $type:ident,
        $definition:expr,
        |$context:ident, $arguments:ident| $body:block,
        $category:ident,
        $summary:literal,
        [$($note:literal),* $(,)?],
        [$(($signature_summary:expr, $example:literal)),+ $(,)?]
    ) => {
        pub(super) struct $type;

        impl $type {
            const DEFINITION: CallableDefinition = $definition;
        }

        impl ToAst for $type {
            fn definition(&self) -> &'static CallableDefinition {
                &Self::DEFINITION
            }

            fn to_ast(
                &self,
                $context: &mut dyn AstContext,
                $arguments: Arguments<'_>,
            ) -> Result<CompiledExpression, ParseError> $body
        }

        #[cfg(any(test, feature = "documentation"))]
        impl ToDoc for $type {
            fn to_doc(&self) -> CallableDocumentation {
                CallableDocumentation {
                    category: DocumentationCategory::$category,
                    summary: $summary,
                    notes: &[$($note),*],
                    signatures: &[
                        $(SignatureDocumentation {
                            summary: $signature_summary,
                            example: $example,
                        }),+
                    ],
                }
            }
        }
    };
}

pub(crate) fn values(arguments: Arguments<'_>) -> Result<Vec<Value>, ParseError> {
    arguments.0.into_iter().map(expect_value).collect()
}

pub(crate) fn value_array<const N: usize>(
    arguments: Arguments<'_>,
) -> Result<[Value; N], ParseError> {
    values(arguments)?
        .try_into()
        .map_err(|_| ParseError::InvalidSyntax)
}

pub(crate) fn expect_value(argument: BoundArgument<'_>) -> Result<Value, ParseError> {
    match argument {
        BoundArgument::Value(value) => Ok(value),
        BoundArgument::Regex(_) | BoundArgument::Step(_) => Err(ParseError::InvalidSyntax),
    }
}

pub(crate) fn expect_regex(argument: BoundArgument<'_>) -> Result<RegexId, ParseError> {
    match argument {
        BoundArgument::Regex(regex) => Ok(regex),
        BoundArgument::Value(_) | BoundArgument::Step(_) => Err(ParseError::InvalidSyntax),
    }
}

pub(crate) fn value(value: Value) -> Result<CompiledExpression, ParseError> {
    Ok(CompiledExpression::Value(value))
}

pub(crate) fn form(form: Form) -> Result<CompiledExpression, ParseError> {
    Ok(CompiledExpression::Form(form))
}

mod boolean;
mod composition;
mod datetime;
mod network;
mod number;
mod program;
mod regex;
mod semver;
mod string;
mod url;

macro_rules! registry {
    ($($module:ident::$type:ident),+ $(,)?) => {
        pub(crate) static CALLABLES: &[&dyn ToAst] = &[$(&$module::$type),+];

        #[cfg(any(test, feature = "documentation"))]
        pub(crate) static DOCUMENTED_CALLABLES: &[&dyn ToDoc] = &[$(&$module::$type),+];
    };
}

registry!(
    program::Print,
    program::Filter,
    number::Add,
    number::Subtract,
    number::Multiply,
    number::Divide,
    number::Truncate,
    number::Floor,
    number::Ceil,
    number::Round,
    number::Absolute,
    number::Fixed,
    number::GreaterThan,
    number::GreaterThanOrEqual,
    number::LessThan,
    number::LessThanOrEqual,
    number::Equal,
    number::NotEqual,
    string::Concat,
    string::Join,
    string::Replace,
    string::ReplaceAll,
    string::Part,
    string::Slice,
    string::Count,
    string::Escape,
    string::DoubleQuote,
    string::SingleQuote,
    string::Lower,
    string::Upper,
    string::Trim,
    string::LeftTrim,
    string::RightTrim,
    string::StartsWith,
    string::EndsWith,
    string::Contains,
    string::GreaterThan,
    string::GreaterThanOrEqual,
    string::LessThan,
    string::LessThanOrEqual,
    string::Equal,
    string::NotEqual,
    boolean::Not,
    boolean::If,
    boolean::Default,
    boolean::And,
    boolean::Or,
    regex::Regex,
    regex::Replace,
    regex::ReplaceAll,
    datetime::Unix,
    datetime::Format,
    datetime::Now,
    datetime::FloorSecond,
    datetime::FloorMinute,
    datetime::FloorHour,
    datetime::FloorDay,
    datetime::Add,
    datetime::Subtract,
    datetime::Difference,
    datetime::DurationSeconds,
    datetime::DurationMilliseconds,
    datetime::DurationMinutes,
    datetime::DurationHours,
    datetime::DurationDays,
    datetime::DurationToMilliseconds,
    datetime::DurationToSeconds,
    datetime::DurationToMinutes,
    datetime::DurationToHours,
    datetime::DurationToDays,
    datetime::GreaterThan,
    datetime::GreaterThanOrEqual,
    datetime::LessThan,
    datetime::LessThanOrEqual,
    datetime::Equal,
    datetime::NotEqual,
    network::IpVersion,
    network::IpEqual,
    network::IpNotEqual,
    network::IpPrivate,
    network::IpLoopback,
    network::IpLinkLocal,
    network::IpMulticast,
    network::CidrContains,
    network::CidrNetwork,
    network::CidrPrefix,
    network::CidrFirst,
    network::CidrLast,
    network::CidrSize,
    url::Scheme,
    url::Host,
    url::Port,
    url::Path,
    url::Query,
    url::Fragment,
    url::QueryGet,
    url::QueryHas,
    url::Encode,
    url::Decode,
    semver::GreaterThan,
    semver::GreaterThanOrEqual,
    semver::LessThan,
    semver::LessThanOrEqual,
    semver::Equal,
    semver::NotEqual,
    semver::Major,
    semver::Minor,
    semver::Patch,
    semver::Prerelease,
    composition::ThreadFirst,
    composition::ThreadLast,
);

pub(crate) fn lookup(name: &str) -> Option<&'static dyn ToAst> {
    CALLABLES.iter().copied().find(|callable| {
        let definition = callable.definition();
        definition.name == name || definition.aliases.contains(&name)
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn registries_contain_the_same_callable_objects() {
        assert_eq!(CALLABLES.len(), DOCUMENTED_CALLABLES.len());
        for (callable, documented) in CALLABLES.iter().zip(DOCUMENTED_CALLABLES) {
            assert_eq!(callable.definition(), documented.definition());
        }
    }

    #[test]
    fn callable_names_aliases_and_signatures_are_valid() {
        let mut names = BTreeSet::new();
        for callable in CALLABLES {
            let definition = callable.definition();
            for name in std::iter::once(definition.name).chain(definition.aliases.iter().copied()) {
                assert!(
                    names.insert(name),
                    "duplicate callable name or alias: {name}"
                );
                assert_eq!(lookup(name).unwrap().definition(), definition);
            }
            for signature in definition.signatures {
                assert!(
                    signature
                        .parameters
                        .iter()
                        .enumerate()
                        .all(|(index, parameter)| {
                            parameter.cardinality == Cardinality::Required
                                || index + 1 == signature.parameters.len()
                        })
                );
            }
            for argument_count in 0..=16 {
                assert!(
                    definition
                        .signatures
                        .iter()
                        .filter(|signature| signature.accepts(argument_count))
                        .count()
                        <= 1
                );
            }
        }
    }
}
