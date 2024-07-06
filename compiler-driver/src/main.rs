use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use clap::Parser;

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[derive(clap::Parser)]
#[command(version, about)]
struct Args {
    /// Dump intermediate representation to a file
    #[arg(short, long)]
    ir: bool,

    /// Output path
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Source file path
    #[arg(value_parser = path_is_valid_file)]
    source: PathBuf,
}

#[inline]
fn path_is_valid_file(s: &str) -> Result<PathBuf> {
    let path = Path::new(s);
    if path.is_file() {
        Ok(path.to_owned())
    } else {
        bail!("Path isn't a valid file")
    }
}

fn main() -> Result<()> {
    let Args { ir, output, source } = Args::parse();

    let file = fs::read_to_string(&source)
        .with_context(|| format!("Couldn't read file from path `{}`", source.display()))?;

    let module = source.file_stem().unwrap().to_str().unwrap();

    let tokens = compiler_lexer::tokenize(&file);

    tokens
        .clone()
        .try_for_each(|e| {
            e?;

            anyhow::Ok(())
        })
        .context("Lexer failed")?;

    // tokens.clone().for_each(|t| println!("{:?}", t));

    let ast = compiler_parser::parse(
        tokens
            .flatten()
            .filter(|t| t.r#type != compiler_lexer::definitions::TokenType::Comment),
    );

    // println!("{ast:#?}");

    compiler_codegen_llvm::gen(module, ast, ir, output)?;

    Ok(())
}
