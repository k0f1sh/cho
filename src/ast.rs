#[derive(Debug, PartialEq)]
pub struct Program {
    pub expressions: Vec<Expr>,
    pub regex_patterns: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegexId(pub usize);

#[derive(Debug, PartialEq)]
pub enum Expr {
    Print(Vec<Value>),
    Filter(Predicate),
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
    IpClass {
        kind: IpClass,
        value: Value,
    },
    CidrContains {
        cidr: Value,
        ip: Value,
    },
    UrlQueryHas {
        name: Value,
        url: Value,
    },
    Not(Box<Predicate>),
    And(Vec<Predicate>),
    Or(Vec<Predicate>),
}

#[derive(Debug, PartialEq)]
pub enum IpClass {
    Private,
    Loopback,
    LinkLocal,
    Multicast,
}

#[derive(Debug, PartialEq)]
pub enum ComparisonType {
    Number,
    String,
    DateTime,
    IpAddr,
    SemVer,
}

#[derive(Debug, PartialEq)]
pub enum ComparisonOperator {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

#[derive(Debug, PartialEq)]
pub enum DateTimeFloorUnit {
    Second,
    Minute,
    Hour,
    Day,
}

#[derive(Debug, PartialEq)]
pub enum ArithmeticOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, PartialEq)]
pub enum UrlPart {
    Scheme,
    Host,
    Port,
    Path,
    Query,
    Fragment,
}

#[derive(Debug, PartialEq)]
pub enum UrlEncoding {
    Encode,
    Decode,
}

#[derive(Debug, PartialEq)]
pub enum CidrPart {
    Network,
    Prefix,
    First,
    Last,
    Size,
}

#[derive(Debug, PartialEq)]
pub enum SemVerPart {
    Major,
    Minor,
    Patch,
    Prerelease,
}

#[derive(Debug, PartialEq)]
pub enum Value {
    Field(usize),
    RecordNumber,
    FieldCount,
    String(String),
    Number(f64),
    Arithmetic {
        operator: ArithmeticOperator,
        left: Box<Value>,
        right: Box<Value>,
    },
    FormatNumberFixed {
        digits: Box<Value>,
        value: Box<Value>,
    },
    UrlPart {
        part: UrlPart,
        value: Box<Value>,
    },
    UrlEncoding {
        operation: UrlEncoding,
        value: Box<Value>,
    },
    UrlQueryGet {
        name: Box<Value>,
        url: Box<Value>,
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
    Predicate(Box<Predicate>),
    DateTimeFromUnix(Box<Value>),
    FormatDateTime {
        format: Box<Value>,
        value: Box<Value>,
    },
    DurationSeconds(Box<Value>),
    DurationMilliseconds(Box<Value>),
    DurationMinutes(Box<Value>),
    DurationHours(Box<Value>),
    DurationDays(Box<Value>),
    DateTimeNow,
    FloorDateTime {
        unit: DateTimeFloorUnit,
        value: Box<Value>,
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
    Part {
        delimiter: Box<Value>,
        position: Box<Value>,
        value: Box<Value>,
    },
    Count(Box<Value>),
    Escape(Box<Value>),
    If {
        predicate: Box<Predicate>,
        then_value: Box<Value>,
        else_value: Box<Value>,
    },
    Lower(Box<Value>),
    Upper(Box<Value>),
    Default {
        value: Box<Value>,
        fallback: Box<Value>,
    },
}
