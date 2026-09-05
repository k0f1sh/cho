#[derive(Debug, PartialEq)]
pub struct Program {
    pub forms: Vec<Form>,
    pub regex_patterns: Vec<String>,
    pub contains_field_range: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegexId(pub usize);

#[derive(Debug, PartialEq)]
pub enum Form {
    Print(Vec<Value>),
    Filter(Value),
}

#[derive(Debug, PartialEq)]
pub enum Predicate {
    Compare {
        kind: ComparisonType,
        operator: ComparisonOperator,
        left: Value,
        right: Value,
    },
    Regex {
        target: Value,
        regex: RegexId,
    },
    StringTest {
        kind: StringTest,
        value: Value,
        pattern: Value,
    },
    IpClass {
        kind: IpClass,
        value: Value,
    },
    CidrContains {
        cidr: Value,
        ip: Value,
    },
    UrlQueryHas {
        url: Value,
        name: Value,
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
pub enum CidrPart {
    Network,
    Prefix,
    First,
    Last,
    Size,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SemVerPart {
    Major,
    Minor,
    Patch,
    Prerelease,
    Build,
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Field(usize),
    DynamicField(Box<Value>),
    FieldRange {
        start: Option<usize>,
        end: Option<usize>,
    },
    DynamicFieldRange {
        start: Option<Box<Value>>,
        end: Option<Box<Value>>,
    },
    RecordNumber,
    FieldCount,
    String(String),
    Number(f64),
    Boolean(bool),
    StringEmpty(Box<Value>),
    Arithmetic {
        operator: ArithmeticOperator,
        left: Box<Value>,
        right: Box<Value>,
    },
    NumberOperation {
        operator: NumberOperator,
        value: Box<Value>,
    },
    FormatNumberFixed {
        value: Box<Value>,
        digits: Box<Value>,
    },
    NumberMinimum(Vec<Value>),
    NumberMaximum(Vec<Value>),
    ClampNumber {
        value: Box<Value>,
        minimum: Box<Value>,
        maximum: Box<Value>,
    },
    NormalizeByteSize(Box<Value>),
    ByteSizeToBytes(Box<Value>),
    UrlPart {
        part: UrlPart,
        value: Box<Value>,
    },
    UrlEncoding {
        operation: UrlEncoding,
        value: Box<Value>,
    },
    PathPart {
        part: PathPart,
        value: Box<Value>,
    },
    UrlQueryGet {
        url: Box<Value>,
        name: Box<Value>,
    },
    IpVersion(Box<Value>),
    CidrPart {
        part: CidrPart,
        value: Box<Value>,
    },
    SemVerPart {
        part: SemVerPart,
        value: Box<Value>,
    },
    NormalizeUuid(Box<Value>),
    UuidV4,
    UuidV7,
    UuidVersion(Box<Value>),
    UuidTime(Box<Value>),
    NormalizeUlid(Box<Value>),
    UlidNew,
    UlidTime(Box<Value>),
    Predicate(Box<Predicate>),
    Not(Box<Value>),
    And(Vec<Value>),
    Or(Vec<Value>),
    NormalizeDate(Box<Value>),
    DatePart {
        part: DatePart,
        value: Box<Value>,
    },
    AddDate {
        date: Box<Value>,
        days: Box<Value>,
    },
    SubtractDate {
        date: Box<Value>,
        days: Box<Value>,
    },
    DifferenceDate {
        left: Box<Value>,
        right: Box<Value>,
    },
    DateTimeFromUnix(Box<Value>),
    DateTimeToUnix(Box<Value>),
    FormatDateTime {
        value: Box<Value>,
        format: Box<Value>,
        timezone: Option<Box<Value>>,
    },
    DurationSeconds(Box<Value>),
    DurationMilliseconds(Box<Value>),
    DurationMinutes(Box<Value>),
    DurationHours(Box<Value>),
    DurationDays(Box<Value>),
    DurationToMilliseconds(Box<Value>),
    DurationToSeconds(Box<Value>),
    DurationToMinutes(Box<Value>),
    DurationToHours(Box<Value>),
    DurationToDays(Box<Value>),
    DateTimeNow,
    FloorDateTime {
        unit: DateTimeFloorUnit,
        value: Box<Value>,
        timezone: Option<Box<Value>>,
    },
    AddDateTime {
        datetime: Box<Value>,
        duration: Box<Value>,
    },
    SubtractDateTime {
        datetime: Box<Value>,
        duration: Box<Value>,
    },
    DifferenceDateTime {
        left: Box<Value>,
        right: Box<Value>,
    },
    Concat(Vec<Value>),
    Join {
        separator: Box<Value>,
        values: Vec<Value>,
    },
    CsvJoin(Vec<Value>),
    Repeat {
        value: Box<Value>,
        count: Box<Value>,
    },
    Replace {
        mode: ReplaceMode,
        value: Box<Value>,
        from: Box<Value>,
        to: Box<Value>,
    },
    RegexReplace {
        mode: ReplaceMode,
        value: Box<Value>,
        regex: RegexId,
        replacement: Box<Value>,
    },
    RegexPart {
        value: Box<Value>,
        regex: RegexId,
        position: Box<Value>,
    },
    Part {
        value: Box<Value>,
        delimiter: Box<Value>,
        position: Box<Value>,
    },
    Boundary {
        kind: StringBoundary,
        value: Box<Value>,
        delimiter: Box<Value>,
    },
    Slice {
        value: Box<Value>,
        start: Box<Value>,
        length: Option<Box<Value>>,
    },
    Pad {
        kind: StringPadding,
        value: Box<Value>,
        width: Box<Value>,
        fill: Option<Box<Value>>,
    },
    Count(Box<Value>),
    Escape(Box<Value>),
    Quote {
        kind: StringQuote,
        value: Box<Value>,
    },
    Unquote(Box<Value>),
    ShellQuote(Box<Value>),
    If {
        condition: Box<Value>,
        then_value: Box<Value>,
        else_value: Box<Value>,
    },
    Lower(Box<Value>),
    Upper(Box<Value>),
    Reverse(Box<Value>),
    Trim {
        kind: StringTrim,
        value: Box<Value>,
    },
    TrimAffixes {
        value: Box<Value>,
        prefix: Option<Box<Value>>,
        suffix: Option<Box<Value>>,
    },
    Default {
        value: Box<Value>,
        fallback: Box<Value>,
    },
}

impl Value {
    /// Number of nested operations after threading has been expanded.
    /// Leaf values have depth zero, matching the parser's list depth convention.
    pub(crate) fn depth(&self) -> usize {
        let children = match self {
            Self::Field(_)
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
            Self::RegexPart {
                value, position, ..
            } => value.depth().max(position.depth()),
            Self::Part {
                value,
                delimiter,
                position,
            } => value.depth().max(delimiter.depth()).max(position.depth()),
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
