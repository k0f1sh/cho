use crate::ast::*;

use super::*;

define_callable!(
    GreaterThan,
    CallableDefinition {
        name: "semver/>",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", SemVer, Required), p!("right", SemVer, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::Compare {
            kind: ComparisonType::SemVer,
            operator: ComparisonOperator::GreaterThan,
            left,
            right,
        })))
    },
    SemanticVersion,
    "greater than",
    [],
    [(None, "(semver/> $1 $2)")]
);

define_callable!(
    GreaterThanOrEqual,
    CallableDefinition {
        name: "semver/>=",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", SemVer, Required), p!("right", SemVer, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::Compare {
            kind: ComparisonType::SemVer,
            operator: ComparisonOperator::GreaterThanOrEqual,
            left,
            right,
        })))
    },
    SemanticVersion,
    "greater than or equal",
    [],
    [(None, "(semver/>= $1 $2)")]
);

define_callable!(
    LessThan,
    CallableDefinition {
        name: "semver/<",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", SemVer, Required), p!("right", SemVer, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::Compare {
            kind: ComparisonType::SemVer,
            operator: ComparisonOperator::LessThan,
            left,
            right,
        })))
    },
    SemanticVersion,
    "less than",
    [],
    [(None, "(semver/< $1 $2)")]
);

define_callable!(
    LessThanOrEqual,
    CallableDefinition {
        name: "semver/<=",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", SemVer, Required), p!("right", SemVer, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::Compare {
            kind: ComparisonType::SemVer,
            operator: ComparisonOperator::LessThanOrEqual,
            left,
            right,
        })))
    },
    SemanticVersion,
    "less than or equal",
    [],
    [(None, "(semver/<= $1 $2)")]
);

define_callable!(
    Equal,
    CallableDefinition {
        name: "semver/=",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", SemVer, Required), p!("right", SemVer, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::Compare {
            kind: ComparisonType::SemVer,
            operator: ComparisonOperator::Equal,
            left,
            right,
        })))
    },
    SemanticVersion,
    "equal",
    [],
    [(None, "(semver/= $1 $2)")]
);

define_callable!(
    NotEqual,
    CallableDefinition {
        name: "semver/!=",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[
            sig!([p!("left", SemVer, Required), p!("right", SemVer, Required)] => Some(ValueType::Boolean))
        ]
    },
    |_context, arguments| {
        let [left, right] = value_array(arguments)?;
        value(Value::Predicate(Box::new(Predicate::Compare {
            kind: ComparisonType::SemVer,
            operator: ComparisonOperator::NotEqual,
            left,
            right,
        })))
    },
    SemanticVersion,
    "not equal",
    [],
    [(None, "(semver/!= $1 $2)")]
);

define_callable!(
    Major,
    CallableDefinition {
        name: "semver/major",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", SemVer, Required)] => Some(ValueType::Number))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::SemVerPart {
            part: SemVerPart::Major,
            value: Box::new(value_arg),
        })
    },
    SemanticVersion,
    "return the major version",
    [],
    [(None, "(semver/major $1)")]
);

define_callable!(
    Minor,
    CallableDefinition {
        name: "semver/minor",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", SemVer, Required)] => Some(ValueType::Number))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::SemVerPart {
            part: SemVerPart::Minor,
            value: Box::new(value_arg),
        })
    },
    SemanticVersion,
    "return the minor version",
    [],
    [(None, "(semver/minor $1)")]
);

define_callable!(
    Patch,
    CallableDefinition {
        name: "semver/patch",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", SemVer, Required)] => Some(ValueType::Number))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::SemVerPart {
            part: SemVerPart::Patch,
            value: Box::new(value_arg),
        })
    },
    SemanticVersion,
    "return the patch version",
    [],
    [(None, "(semver/patch $1)")]
);

define_callable!(
    Prerelease,
    CallableDefinition {
        name: "semver/prerelease",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", SemVer, Required)] => Some(ValueType::String))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::SemVerPart {
            part: SemVerPart::Prerelease,
            value: Box::new(value_arg),
        })
    },
    SemanticVersion,
    "return prerelease text or an empty string",
    [],
    [(None, "(semver/prerelease $1)")]
);

define_callable!(
    Build,
    CallableDefinition {
        name: "semver/build",
        aliases: &[],
        kind: CallableKind::Function,
        signatures: &[sig!([p!("value", SemVer, Required)] => Some(ValueType::String))]
    },
    |_context, arguments| {
        let [value_arg] = value_array(arguments)?;
        value(Value::SemVerPart {
            part: SemVerPart::Build,
            value: Box::new(value_arg),
        })
    },
    SemanticVersion,
    "return build metadata or an empty string",
    [],
    [(None, "(semver/build $1)")]
);
