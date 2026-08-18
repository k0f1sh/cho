#[derive(Debug, PartialEq)]
pub struct Program {
    pub expressions: Vec<Expr>,
}

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
        pattern: String,
    },
    IpPrivate(Value),
    CidrContains {
        cidr: Value,
        ip: Value,
    },
    Not(Box<Predicate>),
    And(Vec<Predicate>),
    Or(Vec<Predicate>),
}

#[derive(Debug, PartialEq)]
pub enum ComparisonType {
    Number,
    String,
    DateTime,
    IpAddr,
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
pub enum Value {
    Field(usize),
    RecordNumber,
    FieldCount,
    String(String),
    Number(f64),
    DateTimeFromUnix(Box<Value>),
    FormatDateTime {
        format: Box<Value>,
        value: Box<Value>,
    },
    DurationSeconds(Box<Value>),
    DurationMinutes(Box<Value>),
    DurationHours(Box<Value>),
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
