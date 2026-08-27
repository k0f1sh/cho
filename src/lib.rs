mod ast;
mod compiler;
mod language;
mod lexer;
mod parser;
mod runtime;

pub use ast::{ComparisonOperator, Form, Predicate, Program, Value};
pub use parser::ParseError;
pub use runtime::{run, run_csv, run_no_input, run_with_field_separator};

pub fn parse(program: &str) -> Result<Program, ParseError> {
    compiler::compile(parser::parse(program)?)
}
