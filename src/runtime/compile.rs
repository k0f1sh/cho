use std::io;

use regex::Regex;

use crate::ast::{Form, Program, Value};
use crate::parse;

pub(super) struct CompiledProgram {
    pub(super) program: Program,
    pub(super) regexes: Vec<Regex>,
}

pub(super) fn compile_program(source: &str) -> io::Result<CompiledProgram> {
    let mut program = parse(source)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid program"))?;
    if program
        .forms
        .iter()
        .all(|form| matches!(form, Form::Filter(_)))
    {
        program.forms.push(Form::Print(vec![Value::Field(0)]));
    }
    let regexes = program
        .regex_patterns
        .iter()
        .map(|pattern| Regex::new(pattern))
        .collect::<Result<_, _>>()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    Ok(CompiledProgram { program, regexes })
}
