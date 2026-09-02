mod ast;
mod compiler;
#[cfg(any(test, feature = "documentation"))]
pub mod documentation;
mod help;
mod language;
mod lexer;
mod parser;
mod runtime;

pub use ast::{ComparisonOperator, Form, Predicate, Program, Value};
pub use parser::ParseError;
pub use runtime::{run, run_csv, run_no_input, run_with_field_separator};

/// Renders help for one function or form, resolving aliases to their canonical names.
pub fn help(topic: &str) -> Option<String> {
    help::render(topic)
}

pub fn parse(program: &str) -> Result<Program, ParseError> {
    compiler::compile(parser::parse(program)?)
}
