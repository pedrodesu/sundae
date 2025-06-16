use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use clap::Parser;
use compiler_codegen_llvm::Settings;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(clap::Parser)]
#[command(version, about)]
struct Args
{
    /// Dump LLVM IR to a file
    #[arg(short, long)]
    ir: bool,

    /// Optimisation level
    #[arg(short = 'O', long, default_value_t = 2, value_parser = clap::value_parser!(u8).range(0..=3))]
    opt: u8,

    /// Output path
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Source file path
    #[arg(value_parser = path_is_valid_file)]
    source: PathBuf,
}

fn path_is_valid_file(s: &str) -> Result<PathBuf>
{
    let path = Path::new(s);
    if path.is_file()
    {
        Ok(path.to_owned())
    }
    else
    {
        bail!("Path isn't a valid file")
    }
}

fn main() -> Result<()>
{
    let Args {
        ir,
        opt,
        output,
        source,
    } = Args::parse();

    let file = fs::read_to_string(&source)
        .with_context(|| format!("Couldn't read file from path `{}`", source.display()))?;

    let module = source.file_stem().unwrap().to_str().unwrap();

    let tokens = compiler_lexer::tokenize(&file);

    // tokens.clone().for_each(|t| println!("{:?}", t));

    // TODO this is kinda goofy ngl...................... though this might be our best option. better clone the iterator's state than collect them all into a sizeable vector
    tokens
        .clone()
        .try_for_each(|e| e.map(|_| ()))
        .context("Lexer failed")?;

    let ast = compiler_parser::parse(
        tokens
            .flatten()
            .filter(|t| t.r#type != compiler_lexer::definitions::TokenType::Comment),
    )?;

    // println!("{ast:#?}");

    compiler_codegen_llvm::gen(
        module,
        ast,
        Settings {
            ir,
            opt,
            output: output.clone(),
        },
    )?;

    Ok(())
}
