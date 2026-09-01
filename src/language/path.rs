use crate::ast::*;

use super::*;

macro_rules! define_path_part {
    ($type:ident, $name:literal, $part:ident, $summary:literal, $example:literal) => {
        define_callable!(
            $type,
            CallableDefinition {
                name: $name,
                aliases: &[],
                kind: CallableKind::Function,
                signatures: &[sig!([p!("value", String, Required)] => Some(ValueType::String))]
            },
            |_context, arguments| {
                let [value_arg] = value_array(arguments)?;
                value(Value::PathPart {
                    part: PathPart::$part,
                    value: Box::new(value_arg),
                })
            },
            Path,
            $summary,
            [],
            [(None, $example)]
        );
    };
}

define_path_part!(
    Name,
    "path/name",
    Name,
    "extract the final path component",
    "(path/name $1)"
);
define_path_part!(
    Stem,
    "path/stem",
    Stem,
    "extract the final component without its extension",
    "(path/stem $1)"
);
define_path_part!(
    Extension,
    "path/ext",
    Extension,
    "extract the extension without its dot or an empty string",
    "(path/ext $1)"
);
define_path_part!(
    Directory,
    "path/dir",
    Directory,
    "extract the directory or an empty string",
    "(path/dir $1)"
);
