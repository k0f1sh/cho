#![allow(dead_code)]

#[path = "../src/ast.rs"]
mod ast;
#[path = "../src/documentation.rs"]
mod documentation;
#[path = "../src/language.rs"]
mod language;

use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let metadata = documentation::metadata(env!("CARGO_PKG_VERSION"))?;
    documentation::validate(&metadata)?;
    let help = documentation::render_help(include_str!("../src/help.txt.in"), &metadata)?;
    let json = format!("{}\n", serde_json::to_string_pretty(&metadata)?);

    fs::write("metadata.json", json)?;
    fs::write("src/help.txt", help.trim_end())?;
    Ok(())
}
