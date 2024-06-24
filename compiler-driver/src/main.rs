use std::{env, fs, path::Path};

use anyhow::{Context, Result};

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

    let tokens = compiler_lexer::tokenize(&file)
        .context("Lexer failed")?
        .into_iter()
        .filter(|t| t.r#type != compiler_lexer::definitions::TokenType::Comment)
        .collect::<Vec<_>>();

    // tokens.iter().for_each(|t| println!("{:?}", t));

    let ast = compiler_parser::parse(tokens);

    // println!("{ast:#?}");

    compiler_codegen_llvm::gen(module, ast)?;

    Ok(())
}

// adhere to syntax on sample.su
