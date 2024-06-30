use std::{env, fs, path::Path};

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let path = env::args().nth(1).expect("no path");
    let file = fs::read_to_string(&path).expect("couldn't read file");

    let module = {
        let path = Path::new(&path);
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        name.rsplit_once('.')
            .map(|n| n.0.to_string())
            .unwrap_or(name)
    };

    let tokens = compiler_lexer::tokenize(&file)
        .context("Lexer failed")?
        .into_iter()
        .filter(|t| t.r#type != compiler_lexer::definitions::TokenType::Comment)
        .collect();

    // tokens.iter().for_each(|t| println!("{:?}", t));

    let ast = compiler_parser::parse(tokens);

    // println!("{ast:#?}");

    compiler_codegen_llvm::gen(module.as_str(), ast)?;

    Ok(())
}
