use std::{env, fs};

use anyhow::{Context, Result};

mod lexer;
mod parser;

fn main() -> Result<()> {
    let path = env::args().nth(1).expect("no path");
    let file = fs::read_to_string(path).expect("couldn't read file");

    // workaround for trailing whitespace for comments as last token
    // TODO maybe a better way to do this? check if EOF on getting token
    let tokens = lexer::tokenize(&(file + "\n")).context("Lexer failed")?;
    println!(
        "{}",
        tabled::Table::new(tokens.clone()).with(tabled::settings::Style::sharp())
    );

    let ast = parser::parse(tokens);
    println!("{:#?}", ast);

    Ok(())
}
