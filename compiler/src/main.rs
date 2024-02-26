#![feature(trait_upcasting)]

use std::{env, fs, path::Path};

use anyhow::{Context, Result};

mod codegen;
mod components;
mod lexer;
mod parser;

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

    let tokens = lexer::tokenize(&file)
        .context("Lexer failed")?
        // we ignore comments for now
        .into_iter()
        .filter(|t| t.r#type != crate::lexer::definitions::TokenType::Comment)
        .collect::<Vec<_>>();

    // tokens.iter().for_each(|t| println!("{:?}", t));

    let ast = parser::parse(tokens);

    // println!("{ast:#?}");

    codegen::gen(module, ast)?;

    Ok(())
}

// TODO clean internal organisation
// adhere to syntax on sample.su
