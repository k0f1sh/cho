#![allow(dead_code)]

#[path = "../src/ast.rs"]
mod ast;
#[path = "../src/documentation.rs"]
mod documentation;
#[path = "../src/language.rs"]
mod language;

use documentation::CallableKind;
use std::process::Command;

fn generated_metadata() -> String {
    let metadata = documentation::metadata(env!("CARGO_PKG_VERSION")).unwrap();
    documentation::validate(&metadata).unwrap();
    format!("{}\n", serde_json::to_string_pretty(&metadata).unwrap())
}

#[test]
fn checked_in_documentation_matches_the_language_registry() {
    let metadata = documentation::metadata(env!("CARGO_PKG_VERSION")).unwrap();
    let help = documentation::render_help(include_str!("../src/help.txt.in"), &metadata).unwrap();
    assert_eq!(include_str!("../metadata.json"), generated_metadata());
    assert_eq!(include_str!("../src/help.txt"), help.trim_end());
}

#[test]
fn every_documented_signature_and_alias_parses() {
    let metadata = documentation::metadata(env!("CARGO_PKG_VERSION")).unwrap();
    for callable in &metadata.callables {
        for signature in &callable.signatures {
            for name in std::iter::once(callable.name).chain(callable.aliases.iter().copied()) {
                let invocation = signature.example.replacen(
                    &format!("({}", callable.name),
                    &format!("({name}"),
                    1,
                );
                let program = if callable.kind == CallableKind::ProgramForm {
                    invocation
                } else {
                    format!("(print {invocation})")
                };
                cho::parse(&program).unwrap_or_else(|error| {
                    panic!("documented example for {name} does not parse: {program}: {error:?}")
                });
            }
        }
    }
}

#[test]
fn help_lists_every_registered_name() {
    let output = Command::new(env!("CARGO_BIN_EXE_cho"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    let metadata = documentation::metadata(env!("CARGO_PKG_VERSION")).unwrap();
    for name in metadata
        .callables
        .iter()
        .flat_map(|callable| std::iter::once(callable.name).chain(callable.aliases.iter().copied()))
    {
        assert!(help.contains(&format!("({name}")), "help omits {name}");
    }
}
