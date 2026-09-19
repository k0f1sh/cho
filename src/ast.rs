#[derive(Debug, PartialEq)]
pub struct Program {
    pub forms: Vec<Form>,
    pub regex_patterns: Vec<String>,
    pub contains_field_range: bool,
    pub header_fields: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegexId(pub usize);

#[derive(Debug, PartialEq)]
pub enum Form {
    Print(Vec<Expr>),
    Filter(Expr),
}

#[derive(Debug, PartialEq)]
pub enum Predicate {
    Compare {
        kind: ComparisonType,
        operator: ComparisonOperator,
        left: Expr,
        right: Expr,
    },
    Regex {
        target: Expr,
        regex: RegexId,
    },
    StringTest {
        kind: StringTest,
        value: Expr,
        pattern: Expr,
    },
    IpClass {
        kind: IpClass,
        value: Expr,
    },
    CidrContains {
        cidr: Expr,
        ip: Expr,
    },
    UrlQueryHas {
        url: Expr,
        name: Expr,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringTest {
    StartsWith,
    EndsWith,
    Contains,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IpClass {
    V4,
    V6,
    Private,
    Loopback,
    LinkLocal,
    Multicast,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ComparisonType {
    Number,
    String,
    Date,
    DateTime,
    ByteSize,
    IpAddr,
    SemVer,
    Uuid,
    Ulid,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DatePart {
    Year,
    Month,
    Day,
    Weekday,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ComparisonOperator {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DateTimeFloorUnit {
    Second,
    Minute,
    Hour,
    Day,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ArithmeticOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NumberOperator {
    Truncate,
    Floor,
    Ceil,
    Round,
    Absolute,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UrlPart {
    Scheme,
    Host,
    Port,
    Path,
    Query,
    Fragment,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UrlEncoding {
    Encode,
    Decode,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PathPart {
    Name,
    Stem,
    Extension,
    Directory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringQuote {
    Double,
    Single,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringTrim {
    Both,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringBoundary {
    Before,
    After,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringPadding {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplaceMode {
    First,
    All,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IpFormat {
    Expanded,
    Binary,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CidrPart {
    Network,
    Prefix,
    First,
    Last,
    Size,
    SizeString,
    Netmask,
    Wildcard,
    HostFirst,
    HostLast,
    HostCount,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SemVerPart {
    Major,
    Minor,
    Patch,
    Prerelease,
    Build,
}

/// An unevaluated expression in the program's abstract syntax tree.
#[derive(Debug, PartialEq)]
pub enum Expr {
    Field(usize),
    HeaderField(usize),
    DynamicField(Box<Expr>),
    FieldRange {
        start: Option<usize>,
        end: Option<usize>,
    },
    DynamicFieldRange {
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
    },
    RecordNumber,
    FieldCount,
    String(String),
    Number(f64),
    Boolean(bool),
    StringEmpty(Box<Expr>),
    Arithmetic {
        operator: ArithmeticOperator,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    NumberOperation {
        operator: NumberOperator,
        value: Box<Expr>,
    },
    FormatNumberFixed {
        value: Box<Expr>,
        digits: Box<Expr>,
    },
    NumberMinimum(Vec<Expr>),
    NumberMaximum(Vec<Expr>),
    ClampNumber {
        value: Box<Expr>,
        minimum: Box<Expr>,
        maximum: Box<Expr>,
    },
    NormalizeByteSize(Box<Expr>),
    ByteSizeToBytes(Box<Expr>),
    UrlPart {
        part: UrlPart,
        value: Box<Expr>,
    },
    UrlEncoding {
        operation: UrlEncoding,
        value: Box<Expr>,
    },
    PathPart {
        part: PathPart,
        value: Box<Expr>,
    },
    UrlQueryGet {
        url: Box<Expr>,
        name: Box<Expr>,
    },
    NormalizeIp(Box<Expr>),
    FormatIp {
        format: IpFormat,
        value: Box<Expr>,
    },
    NormalizeCidr(Box<Expr>),
    MakeCidr {
        ip: Box<Expr>,
        prefix: Box<Expr>,
    },
    CidrFromMask {
        ip: Box<Expr>,
        mask: Box<Expr>,
    },
    IpVersion(Box<Expr>),
    CidrPart {
        part: CidrPart,
        value: Box<Expr>,
    },
    SemVerPart {
        part: SemVerPart,
        value: Box<Expr>,
    },
    NormalizeUuid(Box<Expr>),
    UuidV4,
    UuidV7,
    UuidVersion(Box<Expr>),
    UuidTime(Box<Expr>),
    NormalizeUlid(Box<Expr>),
    UlidNew,
    UlidTime(Box<Expr>),
    Predicate(Box<Predicate>),
    Not(Box<Expr>),
    And(Vec<Expr>),
    Or(Vec<Expr>),
    NormalizeDate(Box<Expr>),
    DatePart {
        part: DatePart,
        value: Box<Expr>,
    },
    AddDate {
        date: Box<Expr>,
        days: Box<Expr>,
    },
    SubtractDate {
        date: Box<Expr>,
        days: Box<Expr>,
    },
    DifferenceDate {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    DateTimeFromUnix(Box<Expr>),
    DateTimeToUnix(Box<Expr>),
    FormatDateTime {
        value: Box<Expr>,
        format: Box<Expr>,
        timezone: Option<Box<Expr>>,
    },
    DurationSeconds(Box<Expr>),
    DurationMilliseconds(Box<Expr>),
    DurationMinutes(Box<Expr>),
    DurationHours(Box<Expr>),
    DurationDays(Box<Expr>),
    DurationToMilliseconds(Box<Expr>),
    DurationToSeconds(Box<Expr>),
    DurationToMinutes(Box<Expr>),
    DurationToHours(Box<Expr>),
    DurationToDays(Box<Expr>),
    DateTimeNow,
    FloorDateTime {
        unit: DateTimeFloorUnit,
        value: Box<Expr>,
        timezone: Option<Box<Expr>>,
    },
    AddDateTime {
        datetime: Box<Expr>,
        duration: Box<Expr>,
    },
    SubtractDateTime {
        datetime: Box<Expr>,
        duration: Box<Expr>,
    },
    DifferenceDateTime {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Concat(Vec<Expr>),
    Join {
        separator: Box<Expr>,
        values: Vec<Expr>,
    },
    CsvJoin(Vec<Expr>),
    Repeat {
        value: Box<Expr>,
        count: Box<Expr>,
    },
    Replace {
        mode: ReplaceMode,
        value: Box<Expr>,
        from: Box<Expr>,
        to: Box<Expr>,
    },
    RegexReplace {
        mode: ReplaceMode,
        value: Box<Expr>,
        regex: RegexId,
        replacement: Box<Expr>,
    },
    RegexExtract {
        value: Box<Expr>,
        regex: RegexId,
        group: Box<Expr>,
    },
    WithLiteralInput {
        value: Box<Expr>,
        delimiter: Option<Box<Expr>>,
        body: Box<Expr>,
    },
    WithRegexInput {
        value: Box<Expr>,
        regex: RegexId,
        body: Box<Expr>,
    },
    Boundary {
        kind: StringBoundary,
        value: Box<Expr>,
        delimiter: Box<Expr>,
    },
    Slice {
        value: Box<Expr>,
        start: Box<Expr>,
        length: Option<Box<Expr>>,
    },
    Pad {
        kind: StringPadding,
        value: Box<Expr>,
        width: Box<Expr>,
        fill: Option<Box<Expr>>,
    },
    Count(Box<Expr>),
    Escape(Box<Expr>),
    Quote {
        kind: StringQuote,
        value: Box<Expr>,
    },
    Unquote(Box<Expr>),
    ShellQuote(Box<Expr>),
    If {
        condition: Box<Expr>,
        then_value: Box<Expr>,
        else_value: Box<Expr>,
    },
    Lower(Box<Expr>),
    Upper(Box<Expr>),
    Reverse(Box<Expr>),
    Trim {
        kind: StringTrim,
        value: Box<Expr>,
    },
    TrimAffixes {
        value: Box<Expr>,
        prefix: Option<Box<Expr>>,
        suffix: Option<Box<Expr>>,
    },
    Default {
        value: Box<Expr>,
        fallback: Box<Expr>,
    },
}

impl Expr {
    /// Number of nested operations after threading has been expanded.
    /// Leaf values have depth zero, matching the parser's list depth convention.
    pub(crate) fn depth(&self) -> usize {
        let children = match self {
            Self::Field(_)
            | Self::HeaderField(_)
            | Self::FieldRange { .. }
            | Self::RecordNumber
            | Self::FieldCount
            | Self::String(_)
            | Self::Number(_)
            | Self::Boolean(_) => return 0,
            Self::UuidV4 | Self::UuidV7 | Self::UlidNew | Self::DateTimeNow => 0,
            Self::DynamicField(value)
            | Self::StringEmpty(value)
            | Self::NormalizeByteSize(value)
            | Self::ByteSizeToBytes(value)
            | Self::NormalizeIp(value)
            | Self::NormalizeCidr(value)
            | Self::FormatIp { value, .. }
            | Self::IpVersion(value)
            | Self::NormalizeUuid(value)
            | Self::UuidVersion(value)
            | Self::UuidTime(value)
            | Self::NormalizeUlid(value)
            | Self::UlidTime(value)
            | Self::Not(value)
            | Self::NormalizeDate(value)
            | Self::DateTimeFromUnix(value)
            | Self::DateTimeToUnix(value)
            | Self::DurationSeconds(value)
            | Self::DurationMilliseconds(value)
            | Self::DurationMinutes(value)
            | Self::DurationHours(value)
            | Self::DurationDays(value)
            | Self::DurationToMilliseconds(value)
            | Self::DurationToSeconds(value)
            | Self::DurationToMinutes(value)
            | Self::DurationToHours(value)
            | Self::DurationToDays(value)
            | Self::Count(value)
            | Self::Escape(value)
            | Self::Unquote(value)
            | Self::ShellQuote(value)
            | Self::Lower(value)
            | Self::Upper(value)
            | Self::Reverse(value)
            | Self::NumberOperation { value, .. }
            | Self::UrlPart { value, .. }
            | Self::UrlEncoding { value, .. }
            | Self::PathPart { value, .. }
            | Self::CidrPart { value, .. }
            | Self::SemVerPart { value, .. }
            | Self::DatePart { value, .. }
            | Self::Quote { value, .. }
            | Self::Trim { value, .. } => value.depth(),
            Self::DynamicFieldRange { start, end } => start
                .iter()
                .chain(end.iter())
                .map(|value| value.depth())
                .max()
                .unwrap_or(0),
            Self::Arithmetic { left, right, .. }
            | Self::DifferenceDate { left, right }
            | Self::DifferenceDateTime { left, right } => left.depth().max(right.depth()),
            Self::MakeCidr { ip, prefix } => ip.depth().max(prefix.depth()),
            Self::CidrFromMask { ip, mask } => ip.depth().max(mask.depth()),
            Self::FormatNumberFixed { value, digits } => value.depth().max(digits.depth()),
            Self::NumberMinimum(values)
            | Self::NumberMaximum(values)
            | Self::And(values)
            | Self::Or(values)
            | Self::Concat(values)
            | Self::CsvJoin(values) => values.iter().map(Self::depth).max().unwrap_or(0),
            Self::ClampNumber {
                value,
                minimum,
                maximum,
            } => value.depth().max(minimum.depth()).max(maximum.depth()),
            Self::UrlQueryGet { url, name } => url.depth().max(name.depth()),
            Self::Predicate(predicate) => match predicate.as_ref() {
                Predicate::Compare { left, right, .. } => left.depth().max(right.depth()),
                Predicate::Regex { target, .. } => target.depth(),
                Predicate::StringTest { value, pattern, .. } => value.depth().max(pattern.depth()),
                Predicate::IpClass { value, .. } => value.depth(),
                Predicate::CidrContains { cidr, ip } => cidr.depth().max(ip.depth()),
                Predicate::UrlQueryHas { url, name } => url.depth().max(name.depth()),
            },
            Self::AddDate { date, days } | Self::SubtractDate { date, days } => {
                date.depth().max(days.depth())
            }
            Self::FormatDateTime {
                value,
                format,
                timezone,
            } => value
                .depth()
                .max(format.depth())
                .max(timezone.as_ref().map_or(0, |value| value.depth())),
            Self::FloorDateTime {
                value, timezone, ..
            } => value
                .depth()
                .max(timezone.as_ref().map_or(0, |value| value.depth())),
            Self::AddDateTime { datetime, duration }
            | Self::SubtractDateTime { datetime, duration } => {
                datetime.depth().max(duration.depth())
            }
            Self::Join { separator, values } => separator
                .depth()
                .max(values.iter().map(Self::depth).max().unwrap_or(0)),
            Self::Repeat { value, count } => value.depth().max(count.depth()),
            Self::Replace {
                value, from, to, ..
            } => value.depth().max(from.depth()).max(to.depth()),
            Self::RegexReplace {
                value, replacement, ..
            } => value.depth().max(replacement.depth()),
            Self::RegexExtract { value, group, .. } => value.depth().max(group.depth()),
            Self::WithLiteralInput {
                value,
                delimiter,
                body,
            } => value
                .depth()
                .max(delimiter.as_ref().map_or(0, |value| value.depth()))
                .max(body.depth()),
            Self::WithRegexInput { value, body, .. } => value.depth().max(body.depth()),
            Self::Boundary {
                value, delimiter, ..
            } => value.depth().max(delimiter.depth()),
            Self::Slice {
                value,
                start,
                length,
            } => value
                .depth()
                .max(start.depth())
                .max(length.as_ref().map_or(0, |value| value.depth())),
            Self::Pad {
                value, width, fill, ..
            } => value
                .depth()
                .max(width.depth())
                .max(fill.as_ref().map_or(0, |value| value.depth())),
            Self::If {
                condition,
                then_value,
                else_value,
            } => condition
                .depth()
                .max(then_value.depth())
                .max(else_value.depth()),
            Self::TrimAffixes {
                value,
                prefix,
                suffix,
            } => value
                .depth()
                .max(prefix.as_ref().map_or(0, |value| value.depth()))
                .max(suffix.as_ref().map_or(0, |value| value.depth())),
            Self::Default { value, fallback } => value.depth().max(fallback.depth()),
        };
        1 + children
    }
}
