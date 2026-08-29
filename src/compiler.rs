use crate::ast::{Form, Program, RegexId, Value};
use crate::language::{
    Arguments, AstContext, BoundArgument, CallableKind, CompiledExpression, Parameter,
    ThreadDirection, ValueType, expect_value, lookup,
};
use crate::parser::{Atom, ParseError, SExpr};

#[derive(Debug)]
enum InputArgument<'syntax> {
    Syntax(&'syntax SExpr),
    Compiled(Value),
}

#[derive(Clone, Copy)]
enum CompileContext {
    Form,
    Value,
}

pub(crate) fn compile(expressions: Vec<SExpr>) -> Result<Program, ParseError> {
    Compiler::new().compile_program(expressions)
}

struct Compiler {
    regex_patterns: Vec<String>,
    contains_field_range: bool,
}

impl Compiler {
    fn new() -> Self {
        Self {
            regex_patterns: Vec::new(),
            contains_field_range: false,
        }
    }

    fn compile_program(mut self, expressions: Vec<SExpr>) -> Result<Program, ParseError> {
        let mut forms = Vec::with_capacity(expressions.len());
        let mut has_explicit_print = false;
        let mut has_implicit_print = false;
        for expression in &expressions {
            let compiled = self.compile_top_level(expression)?;
            match compiled {
                CompiledExpression::Form(Form::Print(_)) if has_implicit_print => {
                    return Err(ParseError::InvalidSyntax);
                }
                CompiledExpression::Form(Form::Print(values)) => {
                    has_explicit_print = true;
                    forms.push(Form::Print(values));
                }
                CompiledExpression::Form(Form::Filter(_)) if has_implicit_print => {
                    return Err(ParseError::InvalidSyntax);
                }
                CompiledExpression::Form(form) => forms.push(form),
                CompiledExpression::Value(_) if has_explicit_print || has_implicit_print => {
                    return Err(ParseError::InvalidSyntax);
                }
                CompiledExpression::Value(value) => {
                    has_implicit_print = true;
                    forms.push(Form::Print(vec![value]));
                }
            }
        }
        Ok(Program {
            forms,
            regex_patterns: self.regex_patterns,
            contains_field_range: self.contains_field_range,
        })
    }

    fn compile_top_level(&mut self, expression: &SExpr) -> Result<CompiledExpression, ParseError> {
        match expression {
            SExpr::List(_) => {
                let (operator, arguments) = call_parts(expression)?;
                let callable = lookup(operator).ok_or(ParseError::InvalidSyntax)?;
                let context = if callable.definition().kind == CallableKind::ProgramForm {
                    CompileContext::Form
                } else {
                    CompileContext::Value
                };
                self.compile_invocation(operator, syntax_arguments(arguments), context)
            }
            _ => self
                .compile_value(expression)
                .map(CompiledExpression::Value),
        }
    }

    fn compile_value(&mut self, expression: &SExpr) -> Result<Value, ParseError> {
        match expression {
            SExpr::Atom(Atom::Symbol(symbol)) => self.compile_symbol(symbol),
            SExpr::Atom(Atom::String(value)) => Ok(Value::String(value.clone())),
            SExpr::Atom(Atom::Regex(_)) => Err(ParseError::InvalidSyntax),
            SExpr::List(items) => self.compile_call(items),
        }
    }

    fn compile_symbol(&mut self, symbol: &str) -> Result<Value, ParseError> {
        match symbol {
            "NR" => Ok(Value::RecordNumber),
            "NF" => Ok(Value::FieldCount),
            "true" => Ok(Value::Boolean(true)),
            "false" => Ok(Value::Boolean(false)),
            _ if symbol.parse::<f64>().is_ok() => {
                Ok(Value::Number(symbol.parse().expect("number was validated")))
            }
            _ => {
                let field = parse_field(symbol)?;
                self.contains_field_range |= matches!(field, Value::FieldRange { .. });
                Ok(field)
            }
        }
    }

    fn compile_call(&mut self, items: &[SExpr]) -> Result<Value, ParseError> {
        let Some(SExpr::Atom(Atom::Symbol(operator))) = items.first() else {
            return Err(ParseError::InvalidSyntax);
        };
        match self.compile_invocation(
            operator,
            syntax_arguments(&items[1..]),
            CompileContext::Value,
        )? {
            CompiledExpression::Value(value) => Ok(value),
            CompiledExpression::Form(_) => Err(ParseError::InvalidSyntax),
        }
    }

    fn compile_invocation<'syntax>(
        &mut self,
        operator: &str,
        arguments: Vec<InputArgument<'syntax>>,
        context: CompileContext,
    ) -> Result<CompiledExpression, ParseError> {
        let callable = lookup(operator).ok_or(ParseError::InvalidSyntax)?;
        let definition = callable.definition();
        let valid_context = match context {
            CompileContext::Form => definition.kind == CallableKind::ProgramForm,
            CompileContext::Value => definition.kind != CallableKind::ProgramForm,
        };
        if !valid_context {
            return Err(ParseError::InvalidSyntax);
        }
        let signature = definition
            .signature(arguments.len())
            .ok_or(ParseError::InvalidSyntax)?;
        let arguments = arguments
            .into_iter()
            .enumerate()
            .map(|(index, argument)| {
                let parameter = signature
                    .parameter(index)
                    .expect("signature accepts argument");
                self.bind_argument(parameter, argument)
            })
            .collect::<Result<Vec<_>, _>>()?;
        callable.to_ast(self, Arguments(arguments))
    }

    fn bind_argument<'syntax>(
        &mut self,
        parameter: Parameter,
        argument: InputArgument<'syntax>,
    ) -> Result<BoundArgument<'syntax>, ParseError> {
        match parameter.value_type {
            ValueType::Regex => self.compile_pattern(argument).map(BoundArgument::Regex),
            ValueType::Step => match argument {
                InputArgument::Syntax(expression) => Ok(BoundArgument::Step(expression)),
                InputArgument::Compiled(_) => Err(ParseError::InvalidSyntax),
            },
            _ => self.compile_argument(argument).map(BoundArgument::Value),
        }
    }

    fn compile_pattern(&mut self, argument: InputArgument<'_>) -> Result<RegexId, ParseError> {
        let pattern = match argument {
            InputArgument::Syntax(SExpr::Atom(Atom::String(pattern) | Atom::Regex(pattern))) => {
                pattern.clone()
            }
            InputArgument::Syntax(_) | InputArgument::Compiled(_) => {
                return Err(ParseError::InvalidSyntax);
            }
        };
        let id = RegexId(self.regex_patterns.len());
        self.regex_patterns.push(pattern);
        Ok(id)
    }

    fn compile_argument(&mut self, argument: InputArgument<'_>) -> Result<Value, ParseError> {
        match argument {
            InputArgument::Syntax(expression) => self.compile_value(expression),
            InputArgument::Compiled(value) => Ok(value),
        }
    }
}

impl AstContext for Compiler {
    fn compile_threading(
        &mut self,
        direction: ThreadDirection,
        arguments: Arguments<'_>,
    ) -> Result<Value, ParseError> {
        let mut arguments = arguments.0.into_iter();
        let mut value = expect_value(arguments.next().ok_or(ParseError::InvalidSyntax)?)?;
        for step in arguments {
            let step = expect_step(step)?;
            let (operator, expressions) = match step {
                SExpr::Atom(Atom::Symbol(operator)) => (operator.as_str(), &[][..]),
                SExpr::List(items) => {
                    let Some(SExpr::Atom(Atom::Symbol(operator))) = items.first() else {
                        return Err(ParseError::InvalidSyntax);
                    };
                    (operator.as_str(), &items[1..])
                }
                _ => return Err(ParseError::InvalidSyntax),
            };
            let mut step_arguments = syntax_arguments(expressions);
            match direction {
                ThreadDirection::First => step_arguments.insert(0, InputArgument::Compiled(value)),
                ThreadDirection::Last => step_arguments.push(InputArgument::Compiled(value)),
            }
            value =
                match self.compile_invocation(operator, step_arguments, CompileContext::Value)? {
                    CompiledExpression::Value(value) => value,
                    CompiledExpression::Form(_) => return Err(ParseError::InvalidSyntax),
                };
        }
        Ok(value)
    }
}

fn call_parts(expression: &SExpr) -> Result<(&str, &[SExpr]), ParseError> {
    let SExpr::List(items) = expression else {
        return Err(ParseError::InvalidSyntax);
    };
    let Some(SExpr::Atom(Atom::Symbol(operator))) = items.first() else {
        return Err(ParseError::InvalidSyntax);
    };
    Ok((operator, &items[1..]))
}

fn syntax_arguments(arguments: &[SExpr]) -> Vec<InputArgument<'_>> {
    arguments.iter().map(InputArgument::Syntax).collect()
}

fn expect_step(argument: BoundArgument<'_>) -> Result<&SExpr, ParseError> {
    match argument {
        BoundArgument::Step(step) => Ok(step),
        BoundArgument::Value(_) | BoundArgument::Regex(_) => Err(ParseError::InvalidSyntax),
    }
}

fn parse_field(field: &str) -> Result<Value, ParseError> {
    let field = field.strip_prefix('$').ok_or(ParseError::InvalidSyntax)?;
    if !field.contains('-') {
        return field
            .parse::<usize>()
            .map(Value::Field)
            .map_err(|_| ParseError::InvalidField);
    }

    let mut bounds = field.split('-');
    let start = parse_range_bound(bounds.next().expect("split always yields one item"))?;
    let end = parse_range_bound(bounds.next().ok_or(ParseError::InvalidField)?)?;
    if bounds.next().is_some()
        || start.is_none() && end.is_none()
        || start == Some(0)
        || end == Some(0)
        || matches!((start, end), (Some(start), Some(end)) if start > end)
    {
        return Err(ParseError::InvalidField);
    }
    Ok(Value::FieldRange { start, end })
}

fn parse_range_bound(bound: &str) -> Result<Option<usize>, ParseError> {
    if bound.is_empty() {
        Ok(None)
    } else {
        bound
            .parse::<usize>()
            .map(Some)
            .map_err(|_| ParseError::InvalidField)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{ComparisonOperator, ComparisonType, Predicate, ReplaceMode};
    use crate::parse;

    #[test]
    fn parses_a_complete_program() {
        assert_eq!(
            parse(r#"(filter (> (s/count $1) 3)) (print (str NR ":" $1))"#),
            Ok(Program {
                forms: vec![
                    Form::Filter(Value::Predicate(Box::new(Predicate::Compare {
                        kind: ComparisonType::Number,
                        operator: ComparisonOperator::GreaterThan,
                        left: Value::Count(Box::new(Value::Field(1))),
                        right: Value::Number(3.0),
                    }))),
                    Form::Print(vec![Value::Concat(vec![
                        Value::RecordNumber,
                        Value::String(":".into()),
                        Value::Field(1),
                    ])]),
                ],
                regex_patterns: vec![],
                contains_field_range: false,
            })
        );
    }

    #[test]
    fn parses_field_ranges_with_explicit_bounds() {
        assert_eq!(
            parse("(print $-3 $3- $2-4)"),
            Ok(Program {
                forms: vec![Form::Print(vec![
                    Value::FieldRange {
                        start: None,
                        end: Some(3),
                    },
                    Value::FieldRange {
                        start: Some(3),
                        end: None,
                    },
                    Value::FieldRange {
                        start: Some(2),
                        end: Some(4),
                    },
                ])],
                regex_patterns: vec![],
                contains_field_range: true,
            })
        );
    }

    #[test]
    fn parses_short_top_level_aliases() {
        assert_eq!(
            parse("(f (> $2 20)) (p $1)"),
            parse("(filter (> $2 20)) (print $1)")
        );
        assert_eq!(parse("(p)"), parse("(print)"));
    }

    #[test]
    fn parses_one_top_level_value_as_a_print() {
        assert_eq!(parse("$1"), parse("(print $1)"));
        assert_eq!(
            parse("(filter (> $2 20)) (s/upper $1)"),
            parse("(filter (> $2 20)) (print (s/upper $1))")
        );
    }

    #[test]
    fn rejects_ambiguous_top_level_values() {
        for program in [
            "$1 $2",
            "(print $1) $2",
            "$1 (print $2)",
            "$1 (filter (> $2 20))",
        ] {
            assert_eq!(parse(program), Err(ParseError::InvalidSyntax), "{program}");
        }
    }

    #[test]
    fn parses_regex_filters() {
        assert!(parse(r#"(filter (reg "error"))"#).is_ok());
        assert!(parse(r#"(filter (reg $1 "^[A-Z]"))"#).is_ok());
        assert!(parse(r#"(filter (reg /^error/))"#).is_ok());
        assert!(parse(r#"(filter (~ $1 /^\d+$/))"#).is_ok());
    }

    #[test]
    fn assigns_regex_ids_in_source_order() {
        let program = parse(r#"(filter (and (reg /error/) (~ $1 "^warn")))"#).unwrap();
        assert_eq!(program.regex_patterns, vec!["error", "^warn"]);
        let Form::Filter(Value::And(predicates)) = &program.forms[0] else {
            panic!("expected an and filter");
        };
        assert!(matches!(
            predicates.as_slice(),
            [
                Value::Predicate(predicate),
                Value::Predicate(other_predicate),
            ] if matches!(predicate.as_ref(), Predicate::Regex {
                    regex: RegexId(0),
                    ..
                }) && matches!(other_predicate.as_ref(), Predicate::Regex {
                    regex: RegexId(1),
                    ..
                })
        ));
    }

    #[test]
    fn parses_boolean_predicates() {
        assert!(
            parse(r#"(filter (and (not (reg "debug")) (or (s/= $1 "info") (s/= $1 "warn"))))"#)
                .is_ok()
        );
        assert!(parse("(filter true)").is_ok());
        assert!(parse("(filter (if (> $1 0) true false))").is_ok());
        assert!(parse("(print (not (if (> $1 0) true false)))").is_ok());
        assert!(parse(r#"(filter (s/starts-with? $1 "api-"))"#).is_ok());
        assert!(parse(r#"(print (s/ends-with? $1 ".log") (s/contains? $1 "error"))"#).is_ok());
        assert_eq!(
            parse(r#"(print (-> $1 (s/starts-with? "api-")))"#),
            parse(r#"(print (s/starts-with? $1 "api-"))"#)
        );
    }

    #[test]
    fn parses_predicates_as_boolean_values() {
        assert!(parse("(print (> $1 $2) (semver/> $1 $2) (ip/loopback? $1))").is_ok());
        assert!(parse("(print (str \"match=\" (and (> $1 0) (< $1 10))))").is_ok());
        assert_eq!(
            parse("(print (-> $1 (semver/> \"1.0.0\")))"),
            parse("(print (semver/> $1 \"1.0.0\"))")
        );
        assert_eq!(
            parse("(print (-> true (and (> $1 0))))"),
            parse("(print (and true (> $1 0)))")
        );
    }

    #[test]
    fn parses_join_values() {
        assert_eq!(
            parse(r#"(print (s/join "," $1 $2))"#),
            Ok(Program {
                forms: vec![Form::Print(vec![Value::Join {
                    separator: Box::new(Value::String(",".into())),
                    values: vec![Value::Field(1), Value::Field(2)],
                }])],
                regex_patterns: vec![],
                contains_field_range: false,
            })
        );
    }

    #[test]
    fn parses_literal_and_regex_replacements() {
        assert_eq!(
            parse(r#"(print (-> $1 (s/replace-all "a" "b")))"#),
            parse(r#"(print (s/replace-all $1 "a" "b"))"#)
        );
        assert_eq!(
            parse(r#"(print (-> $1 (re/replace /a+/ "b")))"#),
            parse(r#"(print (re/replace $1 /a+/ "b"))"#)
        );

        let program =
            parse(r#"(print (s/replace $1 "a" "b") (re/replace-all $2 "(?P<x>x)" "${x}"))"#)
                .unwrap();
        assert_eq!(program.regex_patterns, vec!["(?P<x>x)"]);
        assert!(matches!(
            &program.forms[0],
            Form::Print(values)
                if matches!(values[0], Value::Replace { mode: ReplaceMode::First, .. })
                    && matches!(values[1], Value::RegexReplace {
                        mode: ReplaceMode::All,
                        regex: RegexId(0),
                        ..
                    })
        ));
    }

    #[test]
    fn parses_part_values() {
        assert_eq!(
            parse(r#"(print (s/part $1 (str "]" ":") (s/count "x")))"#),
            Ok(Program {
                forms: vec![Form::Print(vec![Value::Part {
                    delimiter: Box::new(Value::Concat(vec![
                        Value::String("]".into()),
                        Value::String(":".into()),
                    ])),
                    position: Box::new(Value::Count(Box::new(Value::String("x".into())))),
                    value: Box::new(Value::Field(1)),
                }])],
                regex_patterns: vec![],
                contains_field_range: false,
            })
        );
        assert_eq!(
            parse(r#"(print (-> $1 (s/part "=" 2)))"#),
            parse(r#"(print (s/part $1 "=" 2))"#)
        );
        let program = parse(r#"(print (re/part $1 /[,:]+/ (s/count "x")))"#).unwrap();
        assert_eq!(program.regex_patterns, vec!["[,:]+"]);
        assert!(matches!(
            &program.forms[0],
            Form::Print(values)
                if matches!(values[0], Value::RegexPart {
                    regex: RegexId(0),
                    ..
                })
        ));
        assert_eq!(
            parse(r#"(print (-> $1 (re/part /:/ 2)))"#),
            parse(r#"(print (re/part $1 /:/ 2))"#)
        );
    }

    #[test]
    fn parses_slice_with_an_optional_length() {
        assert_eq!(
            parse(r#"(print (s/slice $1 2) (s/slice $1 2 3))"#),
            Ok(Program {
                forms: vec![Form::Print(vec![
                    Value::Slice {
                        start: Box::new(Value::Number(2.0)),
                        length: None,
                        value: Box::new(Value::Field(1)),
                    },
                    Value::Slice {
                        start: Box::new(Value::Number(2.0)),
                        length: Some(Box::new(Value::Number(3.0))),
                        value: Box::new(Value::Field(1)),
                    },
                ])],
                regex_patterns: vec![],
                contains_field_range: false,
            })
        );
    }

    #[test]
    fn parses_conditional_and_string_values() {
        assert!(
            parse(r#"(print (if (s/= (s/lower $1) "alice") (s/upper $2) (default $3 "unknown")))"#)
                .is_ok()
        );
    }

    #[test]
    fn parses_quote_values_and_threading() {
        assert!(parse("(print (s/dquote $1) (s/squote 42) (dq $2) (sq true) (shq $0))").is_ok());
        assert_eq!(parse("(print (dq $1))"), parse("(print (s/dquote $1))"));
        assert_eq!(parse("(print (sq $1))"), parse("(print (s/squote $1))"));
        assert_eq!(
            parse("(print (-> $1 (s/dquote) (s/upper)))"),
            parse("(print (s/upper (s/dquote $1)))")
        );
    }

    #[test]
    fn parses_trim_values_and_threading() {
        assert!(parse("(print (s/trim $1) (s/ltrim $2) (s/rtrim 42))").is_ok());
        assert_eq!(
            parse("(print (-> $1 (s/trim) (s/upper)))"),
            parse("(print (s/upper (s/trim $1)))")
        );
    }

    #[test]
    fn parses_binary_arithmetic_values() {
        assert!(parse("(print (+ $1 2.0) (- $1 $2) (* 3 4) (/ 10 2))").is_ok());
        assert!(parse("(print (+ (* $1 2) (/ $2 4)))").is_ok());
    }

    #[test]
    fn parses_computed_fields_and_rejects_invalid_arities() {
        assert!(parse("(print (field (- NF 2)))").is_ok());
        assert_eq!(parse("(print (field))"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(print (field 1 2))"), Err(ParseError::InvalidSyntax));
    }

    #[test]
    fn parses_fixed_number_formatting() {
        assert!(parse("(print (n/fixed $1 2))").is_ok());
        assert!(parse("(print (str \"$\" (n/fixed (* $1 $2) (+ 1 1))))").is_ok());
    }

    #[test]
    fn parses_url_component_extraction() {
        assert!(parse("(print (url/scheme $1) (url/host $1) (url/port $1))").is_ok());
        assert!(parse("(print (url/path $1) (url/query $1) (url/fragment $1))").is_ok());
        assert!(parse("(print (-> $1 (url/path) (s/upper)))").is_ok());
    }

    #[test]
    fn parses_url_component_encoding() {
        assert!(parse("(print (url/encode $1) (url/decode $2))").is_ok());
        assert!(parse("(print (-> $1 (url/encode) (url/decode)))").is_ok());
    }

    #[test]
    fn parses_typed_values_and_predicates() {
        assert!(parse(r#"(filter (dt/>= $1 "2026-08-01T00:00:00Z"))"#).is_ok());
        assert!(parse(r#"(filter (s/= $1 "Alice"))"#).is_ok());
        assert!(parse(r#"(filter (ip/private? $1))"#).is_ok());
        assert!(parse(r#"(print (ip/version $1))"#).is_ok());
        assert!(parse(r#"(filter (ip/loopback? $1))"#).is_ok());
        assert!(parse(r#"(filter (ip/link-local? $1))"#).is_ok());
        assert!(parse(r#"(filter (ip/multicast? $1))"#).is_ok());
        assert!(parse(r#"(filter (semver/>= $1 "2.4.0"))"#).is_ok());
        assert!(parse(r#"(filter (cidr/contains? "10.0.0.0/8" $1))"#).is_ok());
        assert!(
            parse(r#"(print (cidr/network $1) (cidr/prefix $1) (cidr/first $1) (cidr/last $1))"#)
                .is_ok()
        );
        assert!(parse(r#"(print (cidr/size $1))"#).is_ok());
        assert!(parse(r#"(print (url/query-get $1 "lang") (url/query-has? $1 "page"))"#).is_ok());
        assert!(
            parse(r#"(print (semver/major $1) (semver/minor $1) (semver/patch $1) (semver/prerelease $1))"#)
                .is_ok()
        );
        assert!(parse(r#"(print (dt/add (dt/unix $1) (du/m 2)))"#).is_ok());
        assert!(parse(r#"(print (du/ms 250) (du/d -1.5))"#).is_ok());
        assert!(parse(r#"(print (dt/fmt $1 "%Y-%m-%d"))"#).is_ok());
        assert!(parse(r#"(print (dt/fmt $1 "%Y-%m-%d" "Asia/Tokyo"))"#).is_ok());
        assert!(parse(r#"(print (dt/floor-m (dt/now)))"#).is_ok());
        assert!(parse(r#"(print (dt/floor-d (dt/now) "Asia/Tokyo"))"#).is_ok());
    }

    #[test]
    fn threading_expands_to_existing_value_ast() {
        assert_eq!(
            parse(r#"(print (-> $1 (dt/add (du/s 10))))"#),
            parse(r#"(print (dt/add $1 (du/s 10)))"#)
        );
        assert_eq!(
            parse(r#"(print (-> $1 (n/fixed 2)))"#),
            parse(r#"(print (n/fixed $1 2))"#)
        );
        assert_eq!(
            parse(r#"(print (-> $1 (dt/fmt "%Y/%m/%d" "Asia/Tokyo")))"#),
            parse(r#"(print (dt/fmt $1 "%Y/%m/%d" "Asia/Tokyo"))"#)
        );
        assert!(parse("(print (-> $1 (s/lower) (s/count)))").is_ok());
        assert_eq!(
            parse(r#"(print (-> $1 (dt/floor-h)))"#),
            parse(r#"(print (dt/floor-h $1))"#)
        );
        assert_eq!(
            parse(r#"(print (-> $1 (dt/floor-d "Asia/Tokyo")))"#),
            parse(r#"(print (dt/floor-d $1 "Asia/Tokyo"))"#)
        );
        assert_eq!(
            parse(r#"(print (-> $1 (ip/version)))"#),
            parse(r#"(print (ip/version $1))"#)
        );
        assert_eq!(
            parse(r#"(print (-> $1 (cidr/network) (ip/version)))"#),
            parse(r#"(print (ip/version (cidr/network $1)))"#)
        );
        assert_eq!(
            parse(r#"(print (-> $1 (url/query-get "lang")))"#),
            parse(r#"(print (url/query-get $1 "lang"))"#)
        );
        assert_eq!(
            parse(r#"(print (->> $1 (str "value=")))"#),
            parse(r#"(print (str "value=" $1))"#)
        );
        assert_eq!(
            parse(r#"(print (-> $1 (semver/major)))"#),
            parse(r#"(print (semver/major $1))"#)
        );
        assert_eq!(
            parse(r#"(print (-> $1 s/trim s/upper s/count))"#),
            parse(r#"(print (s/count (s/upper (s/trim $1))))"#)
        );
        assert_eq!(
            parse(r#"(print (->> $1 s/upper (str "value=")))"#),
            parse(r#"(print (str "value=" (s/upper $1)))"#)
        );
        assert_eq!(
            parse(r#"(print (-> $1 s/trim (s/replace "-" "_") s/upper))"#),
            parse(r#"(print (s/upper (s/replace (s/trim $1) "-" "_")))"#)
        );
    }

    #[test]
    fn rejects_invalid_programs() {
        assert_eq!(parse("(print $x)"), Err(ParseError::InvalidField));
        for field in ["$-", "$0-", "$-0", "$2-1", "$1-2-3", "$a-2", "$1-b"] {
            assert_eq!(
                parse(&format!("(print {field})")),
                Err(ParseError::InvalidField)
            );
        }
        assert_eq!(parse("print $1"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(filter (> $1))"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(f)"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(f (> $1 0) $2)"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(filter (reg $1))"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(filter (not))"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(filter (and))"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(filter (or))"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(print (fmt $1))"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(print (s/join))"), Err(ParseError::InvalidSyntax));
        for program in [
            "(print (+))",
            "(print (+ $1))",
            "(print (+ $1 $2 $3))",
            "(print (-))",
            "(print (* $1))",
            "(print (/ $1 $2 $3))",
            "(print (n/fixed))",
            "(print (n/fixed 2))",
            "(print (n/fixed 2 $1 $2))",
            "(print (n/trunc))",
            "(print (n/trunc $1 $2))",
            "(print (n/floor))",
            "(print (n/floor $1 $2))",
            "(print (n/ceil))",
            "(print (n/ceil $1 $2))",
            "(print (n/round))",
            "(print (n/round $1 $2))",
            "(print (n/abs))",
            "(print (n/abs $1 $2))",
            "(print (url/host))",
            "(print (url/path $1 $2))",
            "(print (url/domain $1))",
            "(print (url/encode))",
            "(print (url/decode $1 $2))",
            "(print (s/part))",
            r#"(print (s/part ":"))"#,
            r#"(print (s/part ":" 1))"#,
            r#"(print (s/part ":" 1 $1 $2))"#,
            "(print (s/slice))",
            "(print (s/slice 1))",
            "(print (s/slice 1 2 3 4))",
            "(print (s/dquote))",
            "(print (s/dquote $1 $2))",
            "(print (s/squote))",
            "(print (s/squote $1 $2))",
            "(print (dq))",
            "(print (dq $1 $2))",
            "(print (sq))",
            "(print (sq $1 $2))",
            "(print (s/unquote))",
            "(print (s/unquote $1 $2))",
            "(print (shq))",
            "(print (shq $1 $2))",
            "(print (q $1))",
        ] {
            assert_eq!(parse(program), Err(ParseError::InvalidSyntax), "{program}");
        }
        assert_eq!(parse("(print (s/count))"), Err(ParseError::InvalidSyntax));
        assert_eq!(
            parse("(print (s/count $1 $2))"),
            Err(ParseError::InvalidSyntax)
        );
        assert_eq!(parse("(print (s/escape))"), Err(ParseError::InvalidSyntax));
        assert_eq!(
            parse("(print (s/escape $1 $2))"),
            Err(ParseError::InvalidSyntax)
        );
        for program in [
            "(print (if))",
            "(print (if (= $1 $2) $1))",
            "(print (if (= $1 $2) $1 $2 $3))",
            "(print (s/lower))",
            "(print (s/lower $1 $2))",
            "(print (s/upper))",
            "(print (s/upper $1 $2))",
            "(print (s/trim))",
            "(print (s/trim $1 $2))",
            "(print (s/ltrim))",
            "(print (s/ltrim $1 $2))",
            "(print (s/rtrim))",
            "(print (s/rtrim $1 $2))",
            "(print (default))",
            "(print (default $1))",
            "(print (default $1 $2 $3))",
            "(print (dt/unix))",
            "(print (dt/unix $1 $2))",
            "(print (dt/fmt $1))",
            "(print (dt/fmt $1 $2 $3 $4))",
            "(print (ip/version))",
            "(print (ip/version $1 $2))",
            "(print (cidr/network))",
            "(print (cidr/network $1 $2))",
            "(print (cidr/prefix))",
            "(print (cidr/first $1 $2))",
            "(print (cidr/last))",
            "(print (cidr/size))",
            "(print (cidr/size $1 $2))",
            "(print (url/query-get))",
            "(print (url/query-get $1))",
            "(print (url/query-get $1 $2 $3))",
            "(print (url/query-has? $1))",
            "(print (url/query-has? $1 $2 $3))",
            "(print (semver/major))",
            "(print (semver/minor $1 $2))",
            "(print (semver/patch))",
            "(print (semver/prerelease $1 $2))",
            "(print (du/s))",
            "(print (du/ms))",
            "(print (du/ms $1 $2))",
            "(print (du/d))",
            "(print (du/d $1 $2))",
            "(print (du/to-ms))",
            "(print (du/to-ms $1 $2))",
            "(print (du/to-s))",
            "(print (du/to-s $1 $2))",
            "(print (du/to-m))",
            "(print (du/to-m $1 $2))",
            "(print (du/to-h))",
            "(print (du/to-h $1 $2))",
            "(print (du/to-d))",
            "(print (du/to-d $1 $2))",
            "(print (du/sec (du/s 1)))",
            "(print (du/min (du/m 1)))",
            "(print (du/hour (du/h 1)))",
            "(print (dur/s 1))",
            "(print (dt/now $1))",
            "(print (dt/floor-s))",
            "(print (dt/floor-m $1 $2 $3))",
            r#"(print (join "," $1))"#,
            r#"(print (s/replace $1 "a"))"#,
            r#"(print (s/replace $1 "a" "b" $2))"#,
            r#"(print (re/replace $1 /a/))"#,
            r#"(print (re/replace $1 /a/ "b" $2))"#,
            r#"(print (re/replace $1 $2 "b"))"#,
            r#"(print (re/part))"#,
            r#"(print (re/part $1 /:/))"#,
            r#"(print (re/part $1 /:/ 1 $2))"#,
            r#"(print (re/part $1 $2 1))"#,
            r#"(print (->> $1 (re/replace /a/ "b")))"#,
            r#"(print (-> $1 n/fixed))"#,
            r#"(print (-> $1 unknown))"#,
            r#"(print (s/starts-with? $1))"#,
            r#"(print (s/ends-with? $1 "x" "y"))"#,
            r#"(print (s/contains?))"#,
            "(print (count $1))",
            "(print (escape $1))",
            "(print (lower $1))",
            "(print (upper $1))",
            "(print (dt/add $1))",
            "(print (-> $1 (unknown)))",
            "(filter (ip/loopback?))",
            "(filter (ip/link-local? $1 $2))",
            "(filter (ip/multicast?))",
            "(filter (semver/> $1))",
            "(filter (semver/= $1 $2 $3))",
        ] {
            assert_eq!(parse(program), Err(ParseError::InvalidSyntax), "{program}");
        }
        assert_eq!(
            parse("(print \"unfinished)"),
            Err(ParseError::UnterminatedString)
        );
        assert_eq!(
            parse("(filter (~ /unfinished))"),
            Err(ParseError::UnterminatedRegex)
        );
    }

    #[test]
    fn resolves_operator_and_arity_before_compiling_arguments() {
        assert_eq!(
            parse("(print (unknown $x))"),
            Err(ParseError::InvalidSyntax)
        );
        assert_eq!(
            parse("(print (s/count $1 $x))"),
            Err(ParseError::InvalidSyntax)
        );
        assert_eq!(
            parse("(print (-> $1 (s/count $x $2)))"),
            Err(ParseError::InvalidSyntax)
        );
        assert_eq!(
            parse("(print (-> $x unknown))"),
            Err(ParseError::InvalidField)
        );
        assert_eq!(parse("(print (print $x))"), Err(ParseError::InvalidSyntax));
        assert_eq!(parse("(s/count $x)"), Err(ParseError::InvalidField));
    }
}
