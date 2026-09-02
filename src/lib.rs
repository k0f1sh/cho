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

/// Searches function and form names and renders matching names and summaries.
pub fn apropos(query: &str) -> Option<String> {
    help::apropos(query)
}

pub fn parse(program: &str) -> Result<Program, ParseError> {
    compiler::compile(parser::parse(program)?)
}
