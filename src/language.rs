use crate::ast::{
    ArithmeticOperator, CidrPart, ComparisonOperator, ComparisonType, DateTimeFloorUnit, IpClass,
    NumberOperator, ReplaceMode, SemVerPart, StringQuote, StringTest, StringTrim, UrlEncoding,
    UrlPart,
};

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
pub(crate) enum DurationOperation {
    Seconds,
    Milliseconds,
    Minutes,
    Hours,
    Days,
    ToMilliseconds,
    ToSeconds,
    ToMinutes,
    ToHours,
    ToDays,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ThreadDirection {
    First,
    Last,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Lowering {
    Print,
    Filter,
    Concat,
    Arithmetic(ArithmeticOperator),
    NumberFixed,
    NumberOperation(NumberOperator),
    Join,
    Replace(ReplaceMode),
    RegexReplace(ReplaceMode),
    Part,
    Slice,
    Count,
    Escape,
    Quote(StringQuote),
    Not,
    And,
    Or,
    If,
    Lower,
    Upper,
    Trim(StringTrim),
    Default,
    StringTest(StringTest),
    Regex,
    Compare(ComparisonType, ComparisonOperator),
    DateTimeFromUnix,
    FormatDateTime,
    DateTimeNow,
    FloorDateTime(DateTimeFloorUnit),
    Duration(DurationOperation),
    AddDateTime,
    SubtractDateTime,
    DifferenceDateTime,
    IpVersion,
    IpClass(IpClass),
    CidrContains,
    CidrPart(CidrPart),
    UrlPart(UrlPart),
    UrlEncoding(UrlEncoding),
    UrlQueryGet,
    UrlQueryHas,
    SemVerPart(SemVerPart),
    Thread(ThreadDirection),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CallableSpec {
    pub(crate) name: &'static str,
    pub(crate) aliases: &'static [&'static str],
    pub(crate) kind: CallableKind,
    pub(crate) signatures: &'static [Signature],
    pub(crate) lowering: Lowering,
}

impl CallableSpec {
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

macro_rules! call {
    ($name:literal, $aliases:expr, $kind:ident, [$($signature:expr),+ $(,)?], $lowering:expr) => {
        CallableSpec {
            name: $name,
            aliases: $aliases,
            kind: CallableKind::$kind,
            signatures: &[$($signature),+],
            lowering: $lowering,
        }
    };
}

const VALUE: Option<ValueType> = Some(ValueType::Value);
const STRING: Option<ValueType> = Some(ValueType::String);
const NUMBER: Option<ValueType> = Some(ValueType::Number);
const BOOLEAN: Option<ValueType> = Some(ValueType::Boolean);
const DATETIME: Option<ValueType> = Some(ValueType::DateTime);
const DURATION: Option<ValueType> = Some(ValueType::Duration);
const IPADDR: Option<ValueType> = Some(ValueType::IpAddr);

pub(crate) static CALLABLES: &[CallableSpec] = &[
    call!(
        "print",
        &["p"],
        ProgramForm,
        [sig!([p!("value", Value, ZeroOrMore)] => None)],
        Lowering::Print
    ),
    call!(
        "filter",
        &["f"],
        ProgramForm,
        [sig!([p!("condition", Boolean, Required)] => None)],
        Lowering::Filter
    ),
    call!(
        "+",
        &[],
        Function,
        [sig!([p!("left", Number, Required), p!("right", Number, Required)] => NUMBER)],
        Lowering::Arithmetic(ArithmeticOperator::Add)
    ),
    call!(
        "-",
        &[],
        Function,
        [sig!([p!("left", Number, Required), p!("right", Number, Required)] => NUMBER)],
        Lowering::Arithmetic(ArithmeticOperator::Subtract)
    ),
    call!(
        "*",
        &[],
        Function,
        [sig!([p!("left", Number, Required), p!("right", Number, Required)] => NUMBER)],
        Lowering::Arithmetic(ArithmeticOperator::Multiply)
    ),
    call!(
        "/",
        &[],
        Function,
        [sig!([p!("left", Number, Required), p!("right", Number, Required)] => NUMBER)],
        Lowering::Arithmetic(ArithmeticOperator::Divide)
    ),
    call!(
        "n/trunc",
        &[],
        Function,
        [sig!([p!("value", Number, Required)] => NUMBER)],
        Lowering::NumberOperation(NumberOperator::Truncate)
    ),
    call!(
        "n/floor",
        &[],
        Function,
        [sig!([p!("value", Number, Required)] => NUMBER)],
        Lowering::NumberOperation(NumberOperator::Floor)
    ),
    call!(
        "n/ceil",
        &[],
        Function,
        [sig!([p!("value", Number, Required)] => NUMBER)],
        Lowering::NumberOperation(NumberOperator::Ceil)
    ),
    call!(
        "n/round",
        &[],
        Function,
        [sig!([p!("value", Number, Required)] => NUMBER)],
        Lowering::NumberOperation(NumberOperator::Round)
    ),
    call!(
        "n/abs",
        &[],
        Function,
        [sig!([p!("value", Number, Required)] => NUMBER)],
        Lowering::NumberOperation(NumberOperator::Absolute)
    ),
    call!(
        "n/fixed",
        &[],
        Function,
        [sig!([p!("value", Number, Required), p!("digits", Number, Required)] => STRING)],
        Lowering::NumberFixed
    ),
    call!(
        ">",
        &[],
        Function,
        [sig!([p!("left", Number, Required), p!("right", Number, Required)] => BOOLEAN)],
        Lowering::Compare(ComparisonType::Number, ComparisonOperator::GreaterThan)
    ),
    call!(
        ">=",
        &[],
        Function,
        [sig!([p!("left", Number, Required), p!("right", Number, Required)] => BOOLEAN)],
        Lowering::Compare(
            ComparisonType::Number,
            ComparisonOperator::GreaterThanOrEqual
        )
    ),
    call!(
        "<",
        &[],
        Function,
        [sig!([p!("left", Number, Required), p!("right", Number, Required)] => BOOLEAN)],
        Lowering::Compare(ComparisonType::Number, ComparisonOperator::LessThan)
    ),
    call!(
        "<=",
        &[],
        Function,
        [sig!([p!("left", Number, Required), p!("right", Number, Required)] => BOOLEAN)],
        Lowering::Compare(ComparisonType::Number, ComparisonOperator::LessThanOrEqual)
    ),
    call!(
        "=",
        &[],
        Function,
        [sig!([p!("left", Number, Required), p!("right", Number, Required)] => BOOLEAN)],
        Lowering::Compare(ComparisonType::Number, ComparisonOperator::Equal)
    ),
    call!(
        "!=",
        &[],
        Function,
        [sig!([p!("left", Number, Required), p!("right", Number, Required)] => BOOLEAN)],
        Lowering::Compare(ComparisonType::Number, ComparisonOperator::NotEqual)
    ),
    call!(
        "str",
        &[],
        Function,
        [sig!([p!("value", Value, ZeroOrMore)] => STRING)],
        Lowering::Concat
    ),
    call!(
        "s/join",
        &[],
        Function,
        [sig!([p!("separator", Value, Required), p!("value", Value, ZeroOrMore)] => STRING)],
        Lowering::Join
    ),
    call!(
        "s/replace",
        &[],
        Function,
        [
            sig!([p!("value", Value, Required), p!("from", Value, Required), p!("to", Value, Required)] => STRING)
        ],
        Lowering::Replace(ReplaceMode::First)
    ),
    call!(
        "s/replace-all",
        &[],
        Function,
        [
            sig!([p!("value", Value, Required), p!("from", Value, Required), p!("to", Value, Required)] => STRING)
        ],
        Lowering::Replace(ReplaceMode::All)
    ),
    call!(
        "s/part",
        &[],
        Function,
        [
            sig!([p!("value", Value, Required), p!("delimiter", Value, Required), p!("position", Number, Required)] => STRING)
        ],
        Lowering::Part
    ),
    call!(
        "s/slice",
        &[],
        Function,
        [
            sig!([p!("value", Value, Required), p!("start", Number, Required), p!("length", Number, Optional)] => STRING)
        ],
        Lowering::Slice
    ),
    call!(
        "s/count",
        &[],
        Function,
        [sig!([p!("value", Value, Required)] => NUMBER)],
        Lowering::Count
    ),
    call!(
        "s/escape",
        &[],
        Function,
        [sig!([p!("value", Value, Required)] => STRING)],
        Lowering::Escape
    ),
    call!(
        "s/dquote",
        &["dq"],
        Function,
        [sig!([p!("value", Value, Required)] => STRING)],
        Lowering::Quote(StringQuote::Double)
    ),
    call!(
        "s/squote",
        &["sq"],
        Function,
        [sig!([p!("value", Value, Required)] => STRING)],
        Lowering::Quote(StringQuote::Single)
    ),
    call!(
        "s/lower",
        &[],
        Function,
        [sig!([p!("value", Value, Required)] => STRING)],
        Lowering::Lower
    ),
    call!(
        "s/upper",
        &[],
        Function,
        [sig!([p!("value", Value, Required)] => STRING)],
        Lowering::Upper
    ),
    call!(
        "s/trim",
        &[],
        Function,
        [sig!([p!("value", Value, Required)] => STRING)],
        Lowering::Trim(StringTrim::Both)
    ),
    call!(
        "s/ltrim",
        &[],
        Function,
        [sig!([p!("value", Value, Required)] => STRING)],
        Lowering::Trim(StringTrim::Left)
    ),
    call!(
        "s/rtrim",
        &[],
        Function,
        [sig!([p!("value", Value, Required)] => STRING)],
        Lowering::Trim(StringTrim::Right)
    ),
    call!(
        "s/starts-with?",
        &[],
        Function,
        [sig!([p!("string", String, Required), p!("prefix", String, Required)] => BOOLEAN)],
        Lowering::StringTest(StringTest::StartsWith)
    ),
    call!(
        "s/ends-with?",
        &[],
        Function,
        [sig!([p!("string", String, Required), p!("suffix", String, Required)] => BOOLEAN)],
        Lowering::StringTest(StringTest::EndsWith)
    ),
    call!(
        "s/contains?",
        &[],
        Function,
        [sig!([p!("string", String, Required), p!("needle", String, Required)] => BOOLEAN)],
        Lowering::StringTest(StringTest::Contains)
    ),
    call!(
        "s/>",
        &[],
        Function,
        [sig!([p!("left", String, Required), p!("right", String, Required)] => BOOLEAN)],
        Lowering::Compare(ComparisonType::String, ComparisonOperator::GreaterThan)
    ),
    call!(
        "s/>=",
        &[],
        Function,
        [sig!([p!("left", String, Required), p!("right", String, Required)] => BOOLEAN)],
        Lowering::Compare(
            ComparisonType::String,
            ComparisonOperator::GreaterThanOrEqual
        )
    ),
    call!(
        "s/<",
        &[],
        Function,
        [sig!([p!("left", String, Required), p!("right", String, Required)] => BOOLEAN)],
        Lowering::Compare(ComparisonType::String, ComparisonOperator::LessThan)
    ),
    call!(
        "s/<=",
        &[],
        Function,
        [sig!([p!("left", String, Required), p!("right", String, Required)] => BOOLEAN)],
        Lowering::Compare(ComparisonType::String, ComparisonOperator::LessThanOrEqual)
    ),
    call!(
        "s/=",
        &[],
        Function,
        [sig!([p!("left", String, Required), p!("right", String, Required)] => BOOLEAN)],
        Lowering::Compare(ComparisonType::String, ComparisonOperator::Equal)
    ),
    call!(
        "s/!=",
        &[],
        Function,
        [sig!([p!("left", String, Required), p!("right", String, Required)] => BOOLEAN)],
        Lowering::Compare(ComparisonType::String, ComparisonOperator::NotEqual)
    ),
    call!(
        "not",
        &[],
        Function,
        [sig!([p!("value", Boolean, Required)] => BOOLEAN)],
        Lowering::Not
    ),
    call!(
        "if",
        &[],
        SpecialForm,
        [
            sig!([p!("condition", Boolean, Required), p!("then", Value, Required), p!("else", Value, Required)] => VALUE)
        ],
        Lowering::If
    ),
    call!(
        "default",
        &[],
        SpecialForm,
        [sig!([p!("value", Value, Required), p!("fallback", Value, Required)] => VALUE)],
        Lowering::Default
    ),
    call!(
        "and",
        &[],
        SpecialForm,
        [sig!([p!("value", Boolean, OneOrMore)] => BOOLEAN)],
        Lowering::And
    ),
    call!(
        "or",
        &[],
        SpecialForm,
        [sig!([p!("value", Boolean, OneOrMore)] => BOOLEAN)],
        Lowering::Or
    ),
    call!(
        "reg",
        &["~"],
        Function,
        [
            sig!([p!("pattern", Regex, Required)] => BOOLEAN),
            sig!([p!("value", Value, Required), p!("pattern", Regex, Required)] => BOOLEAN)
        ],
        Lowering::Regex
    ),
    call!(
        "re/replace",
        &[],
        Function,
        [
            sig!([p!("value", Value, Required), p!("pattern", Regex, Required), p!("replacement", Value, Required)] => STRING)
        ],
        Lowering::RegexReplace(ReplaceMode::First)
    ),
    call!(
        "re/replace-all",
        &[],
        Function,
        [
            sig!([p!("value", Value, Required), p!("pattern", Regex, Required), p!("replacement", Value, Required)] => STRING)
        ],
        Lowering::RegexReplace(ReplaceMode::All)
    ),
    call!(
        "dt/unix",
        &[],
        Function,
        [sig!([p!("value", Number, Required)] => DATETIME)],
        Lowering::DateTimeFromUnix
    ),
    call!(
        "dt/fmt",
        &[],
        Function,
        [
            sig!([p!("datetime", DateTime, Required), p!("format", String, Required)] => STRING),
            sig!([p!("datetime", DateTime, Required), p!("format", String, Required), p!("timezone", String, Required)] => STRING)
        ],
        Lowering::FormatDateTime
    ),
    call!(
        "dt/now",
        &[],
        Function,
        [sig!([] => DATETIME)],
        Lowering::DateTimeNow
    ),
    call!(
        "dt/floor-s",
        &[],
        Function,
        [
            sig!([p!("datetime", DateTime, Required)] => DATETIME),
            sig!([p!("datetime", DateTime, Required), p!("timezone", String, Required)] => DATETIME)
        ],
        Lowering::FloorDateTime(DateTimeFloorUnit::Second)
    ),
    call!(
        "dt/floor-m",
        &[],
        Function,
        [
            sig!([p!("datetime", DateTime, Required)] => DATETIME),
            sig!([p!("datetime", DateTime, Required), p!("timezone", String, Required)] => DATETIME)
        ],
        Lowering::FloorDateTime(DateTimeFloorUnit::Minute)
    ),
    call!(
        "dt/floor-h",
        &[],
        Function,
        [
            sig!([p!("datetime", DateTime, Required)] => DATETIME),
            sig!([p!("datetime", DateTime, Required), p!("timezone", String, Required)] => DATETIME)
        ],
        Lowering::FloorDateTime(DateTimeFloorUnit::Hour)
    ),
    call!(
        "dt/floor-d",
        &[],
        Function,
        [
            sig!([p!("datetime", DateTime, Required)] => DATETIME),
            sig!([p!("datetime", DateTime, Required), p!("timezone", String, Required)] => DATETIME)
        ],
        Lowering::FloorDateTime(DateTimeFloorUnit::Day)
    ),
    call!(
        "dt/add",
        &[],
        Function,
        [
            sig!([p!("datetime", DateTime, Required), p!("duration", Duration, Required)] => DATETIME)
        ],
        Lowering::AddDateTime
    ),
    call!(
        "dt/sub",
        &[],
        Function,
        [
            sig!([p!("datetime", DateTime, Required), p!("duration", Duration, Required)] => DATETIME)
        ],
        Lowering::SubtractDateTime
    ),
    call!(
        "dt/diff",
        &[],
        Function,
        [sig!([p!("left", DateTime, Required), p!("right", DateTime, Required)] => DURATION)],
        Lowering::DifferenceDateTime
    ),
    call!(
        "du/s",
        &[],
        Function,
        [sig!([p!("value", Number, Required)] => DURATION)],
        Lowering::Duration(DurationOperation::Seconds)
    ),
    call!(
        "du/ms",
        &[],
        Function,
        [sig!([p!("value", Number, Required)] => DURATION)],
        Lowering::Duration(DurationOperation::Milliseconds)
    ),
    call!(
        "du/m",
        &[],
        Function,
        [sig!([p!("value", Number, Required)] => DURATION)],
        Lowering::Duration(DurationOperation::Minutes)
    ),
    call!(
        "du/h",
        &[],
        Function,
        [sig!([p!("value", Number, Required)] => DURATION)],
        Lowering::Duration(DurationOperation::Hours)
    ),
    call!(
        "du/d",
        &[],
        Function,
        [sig!([p!("value", Number, Required)] => DURATION)],
        Lowering::Duration(DurationOperation::Days)
    ),
    call!(
        "du/to-ms",
        &[],
        Function,
        [sig!([p!("value", Duration, Required)] => NUMBER)],
        Lowering::Duration(DurationOperation::ToMilliseconds)
    ),
    call!(
        "du/to-s",
        &[],
        Function,
        [sig!([p!("value", Duration, Required)] => NUMBER)],
        Lowering::Duration(DurationOperation::ToSeconds)
    ),
    call!(
        "du/to-m",
        &[],
        Function,
        [sig!([p!("value", Duration, Required)] => NUMBER)],
        Lowering::Duration(DurationOperation::ToMinutes)
    ),
    call!(
        "du/to-h",
        &[],
        Function,
        [sig!([p!("value", Duration, Required)] => NUMBER)],
        Lowering::Duration(DurationOperation::ToHours)
    ),
    call!(
        "du/to-d",
        &[],
        Function,
        [sig!([p!("value", Duration, Required)] => NUMBER)],
        Lowering::Duration(DurationOperation::ToDays)
    ),
    call!(
        "dt/>",
        &[],
        Function,
        [sig!([p!("left", DateTime, Required), p!("right", DateTime, Required)] => BOOLEAN)],
        Lowering::Compare(ComparisonType::DateTime, ComparisonOperator::GreaterThan)
    ),
    call!(
        "dt/>=",
        &[],
        Function,
        [sig!([p!("left", DateTime, Required), p!("right", DateTime, Required)] => BOOLEAN)],
        Lowering::Compare(
            ComparisonType::DateTime,
            ComparisonOperator::GreaterThanOrEqual
        )
    ),
    call!(
        "dt/<",
        &[],
        Function,
        [sig!([p!("left", DateTime, Required), p!("right", DateTime, Required)] => BOOLEAN)],
        Lowering::Compare(ComparisonType::DateTime, ComparisonOperator::LessThan)
    ),
    call!(
        "dt/<=",
        &[],
        Function,
        [sig!([p!("left", DateTime, Required), p!("right", DateTime, Required)] => BOOLEAN)],
        Lowering::Compare(
            ComparisonType::DateTime,
            ComparisonOperator::LessThanOrEqual
        )
    ),
    call!(
        "dt/=",
        &[],
        Function,
        [sig!([p!("left", DateTime, Required), p!("right", DateTime, Required)] => BOOLEAN)],
        Lowering::Compare(ComparisonType::DateTime, ComparisonOperator::Equal)
    ),
    call!(
        "dt/!=",
        &[],
        Function,
        [sig!([p!("left", DateTime, Required), p!("right", DateTime, Required)] => BOOLEAN)],
        Lowering::Compare(ComparisonType::DateTime, ComparisonOperator::NotEqual)
    ),
    call!(
        "ip/version",
        &[],
        Function,
        [sig!([p!("value", IpAddr, Required)] => NUMBER)],
        Lowering::IpVersion
    ),
    call!(
        "ip/=",
        &[],
        Function,
        [sig!([p!("left", IpAddr, Required), p!("right", IpAddr, Required)] => BOOLEAN)],
        Lowering::Compare(ComparisonType::IpAddr, ComparisonOperator::Equal)
    ),
    call!(
        "ip/!=",
        &[],
        Function,
        [sig!([p!("left", IpAddr, Required), p!("right", IpAddr, Required)] => BOOLEAN)],
        Lowering::Compare(ComparisonType::IpAddr, ComparisonOperator::NotEqual)
    ),
    call!(
        "ip/private?",
        &[],
        Function,
        [sig!([p!("value", IpAddr, Required)] => BOOLEAN)],
        Lowering::IpClass(IpClass::Private)
    ),
    call!(
        "ip/loopback?",
        &[],
        Function,
        [sig!([p!("value", IpAddr, Required)] => BOOLEAN)],
        Lowering::IpClass(IpClass::Loopback)
    ),
    call!(
        "ip/link-local?",
        &[],
        Function,
        [sig!([p!("value", IpAddr, Required)] => BOOLEAN)],
        Lowering::IpClass(IpClass::LinkLocal)
    ),
    call!(
        "ip/multicast?",
        &[],
        Function,
        [sig!([p!("value", IpAddr, Required)] => BOOLEAN)],
        Lowering::IpClass(IpClass::Multicast)
    ),
    call!(
        "cidr/contains?",
        &[],
        Function,
        [sig!([p!("cidr", Cidr, Required), p!("ip", IpAddr, Required)] => BOOLEAN)],
        Lowering::CidrContains
    ),
    call!(
        "cidr/network",
        &[],
        Function,
        [sig!([p!("value", Cidr, Required)] => IPADDR)],
        Lowering::CidrPart(CidrPart::Network)
    ),
    call!(
        "cidr/prefix",
        &[],
        Function,
        [sig!([p!("value", Cidr, Required)] => NUMBER)],
        Lowering::CidrPart(CidrPart::Prefix)
    ),
    call!(
        "cidr/first",
        &[],
        Function,
        [sig!([p!("value", Cidr, Required)] => IPADDR)],
        Lowering::CidrPart(CidrPart::First)
    ),
    call!(
        "cidr/last",
        &[],
        Function,
        [sig!([p!("value", Cidr, Required)] => IPADDR)],
        Lowering::CidrPart(CidrPart::Last)
    ),
    call!(
        "cidr/size",
        &[],
        Function,
        [sig!([p!("value", Cidr, Required)] => NUMBER)],
        Lowering::CidrPart(CidrPart::Size)
    ),
    call!(
        "url/scheme",
        &[],
        Function,
        [sig!([p!("value", Url, Required)] => STRING)],
        Lowering::UrlPart(UrlPart::Scheme)
    ),
    call!(
        "url/host",
        &[],
        Function,
        [sig!([p!("value", Url, Required)] => STRING)],
        Lowering::UrlPart(UrlPart::Host)
    ),
    call!(
        "url/port",
        &[],
        Function,
        [sig!([p!("value", Url, Required)] => STRING)],
        Lowering::UrlPart(UrlPart::Port)
    ),
    call!(
        "url/path",
        &[],
        Function,
        [sig!([p!("value", Url, Required)] => STRING)],
        Lowering::UrlPart(UrlPart::Path)
    ),
    call!(
        "url/query",
        &[],
        Function,
        [sig!([p!("value", Url, Required)] => STRING)],
        Lowering::UrlPart(UrlPart::Query)
    ),
    call!(
        "url/fragment",
        &[],
        Function,
        [sig!([p!("value", Url, Required)] => STRING)],
        Lowering::UrlPart(UrlPart::Fragment)
    ),
    call!(
        "url/query-get",
        &[],
        Function,
        [sig!([p!("url", Url, Required), p!("name", String, Required)] => STRING)],
        Lowering::UrlQueryGet
    ),
    call!(
        "url/query-has?",
        &[],
        Function,
        [sig!([p!("url", Url, Required), p!("name", String, Required)] => BOOLEAN)],
        Lowering::UrlQueryHas
    ),
    call!(
        "url/encode",
        &[],
        Function,
        [sig!([p!("value", String, Required)] => STRING)],
        Lowering::UrlEncoding(UrlEncoding::Encode)
    ),
    call!(
        "url/decode",
        &[],
        Function,
        [sig!([p!("value", String, Required)] => STRING)],
        Lowering::UrlEncoding(UrlEncoding::Decode)
    ),
    call!(
        "semver/>",
        &[],
        Function,
        [sig!([p!("left", SemVer, Required), p!("right", SemVer, Required)] => BOOLEAN)],
        Lowering::Compare(ComparisonType::SemVer, ComparisonOperator::GreaterThan)
    ),
    call!(
        "semver/>=",
        &[],
        Function,
        [sig!([p!("left", SemVer, Required), p!("right", SemVer, Required)] => BOOLEAN)],
        Lowering::Compare(
            ComparisonType::SemVer,
            ComparisonOperator::GreaterThanOrEqual
        )
    ),
    call!(
        "semver/<",
        &[],
        Function,
        [sig!([p!("left", SemVer, Required), p!("right", SemVer, Required)] => BOOLEAN)],
        Lowering::Compare(ComparisonType::SemVer, ComparisonOperator::LessThan)
    ),
    call!(
        "semver/<=",
        &[],
        Function,
        [sig!([p!("left", SemVer, Required), p!("right", SemVer, Required)] => BOOLEAN)],
        Lowering::Compare(ComparisonType::SemVer, ComparisonOperator::LessThanOrEqual)
    ),
    call!(
        "semver/=",
        &[],
        Function,
        [sig!([p!("left", SemVer, Required), p!("right", SemVer, Required)] => BOOLEAN)],
        Lowering::Compare(ComparisonType::SemVer, ComparisonOperator::Equal)
    ),
    call!(
        "semver/!=",
        &[],
        Function,
        [sig!([p!("left", SemVer, Required), p!("right", SemVer, Required)] => BOOLEAN)],
        Lowering::Compare(ComparisonType::SemVer, ComparisonOperator::NotEqual)
    ),
    call!(
        "semver/major",
        &[],
        Function,
        [sig!([p!("value", SemVer, Required)] => NUMBER)],
        Lowering::SemVerPart(SemVerPart::Major)
    ),
    call!(
        "semver/minor",
        &[],
        Function,
        [sig!([p!("value", SemVer, Required)] => NUMBER)],
        Lowering::SemVerPart(SemVerPart::Minor)
    ),
    call!(
        "semver/patch",
        &[],
        Function,
        [sig!([p!("value", SemVer, Required)] => NUMBER)],
        Lowering::SemVerPart(SemVerPart::Patch)
    ),
    call!(
        "semver/prerelease",
        &[],
        Function,
        [sig!([p!("value", SemVer, Required)] => STRING)],
        Lowering::SemVerPart(SemVerPart::Prerelease)
    ),
    call!(
        "->",
        &[],
        ThreadingForm,
        [sig!([p!("value", Value, Required), p!("step", Step, ZeroOrMore)] => VALUE)],
        Lowering::Thread(ThreadDirection::First)
    ),
    call!(
        "->>",
        &[],
        ThreadingForm,
        [sig!([p!("value", Value, Required), p!("step", Step, ZeroOrMore)] => VALUE)],
        Lowering::Thread(ThreadDirection::Last)
    ),
];

pub(crate) fn lookup(name: &str) -> Option<&'static CallableSpec> {
    CALLABLES
        .iter()
        .find(|callable| callable.name == name || callable.aliases.contains(&name))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn callable_names_and_aliases_are_unique() {
        let mut names = BTreeSet::new();
        for callable in CALLABLES {
            for name in std::iter::once(callable.name).chain(callable.aliases.iter().copied()) {
                assert!(
                    names.insert(name),
                    "duplicate callable name or alias: {name}"
                );
                assert_eq!(lookup(name), Some(callable));
            }
        }
    }

    #[test]
    fn signatures_are_well_formed_and_unambiguous() {
        for callable in CALLABLES {
            for signature in callable.signatures {
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
                let matches = callable
                    .signatures
                    .iter()
                    .filter(|signature| signature.accepts(argument_count))
                    .count();
                assert!(matches <= 1, "ambiguous signature for {}", callable.name);
            }
        }
    }
}
