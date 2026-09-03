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
