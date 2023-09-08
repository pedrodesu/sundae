#![feature(ascii_char)]
#![feature(lazy_cell)]

use std::{env, fs, path::Path};

use anyhow::{Context, Result};

mod codegen;
mod lexer;
mod parser;

// TODO change lexer to not use ; and use lines as statements. blocks can still be defined with {} (like in golang)
fn main() -> Result<()> {
    let path = env::args().nth(1).expect("no path");
    let file = fs::read_to_string(&path).expect("couldn't read file");

    let module = {
        let path = Path::new(&path);
        let name = path
            .file_name()
            .unwrap()
            .to_str()
            .expect("Invalid file name");
        name.rsplit_once('.').map(|(l, _)| l).unwrap_or(name)
    };

    let tokens = lexer::tokenize(&(file + "\n")).context("Lexer failed")?;
    println!(
        "{}",
        tabled::Table::new(tokens.clone()).with(tabled::settings::Style::sharp())
    );

    let ast = parser::parse(tokens);
    println!("{:#?}", ast);

    codegen::gen(module, ast);

    Ok(())
}
