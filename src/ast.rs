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
        operator: ComparisonOperator,
        left: Value,
        right: Value,
    },
    Regex {
        target: Value,
        pattern: String,
    },
    Not(Box<Predicate>),
    And(Vec<Predicate>),
    Or(Vec<Predicate>),
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
pub enum Value {
    Field(usize),
    RecordNumber,
    FieldCount,
    String(String),
    Number(f64),
    Concat(Vec<Value>),
    Count(Box<Value>),
}
