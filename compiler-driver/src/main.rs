use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use compiler_codegen_llvm::Settings;
use miette::{Context, IntoDiagnostic, Result, bail};
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
        .into_diagnostic()
        .wrap_err_with(|| format!("Couldn't read file from path `{}`", source.display()))?;

    let module = source
        .file_stem()
        .and_then(|s| s.to_str())
        .wrap_err("Incorrect file name")?;

    let tokens = compiler_lexer::tokenize(&file);

    // TODO Use collect instead. I'd rather allocate more to the heap than run the whole lexer twice lmao
    tokens.clone().try_for_each(|e| e.map(|_| ()))?;

    let ast = compiler_parser::parse(
        tokens
            .flatten()
            .filter(|t| t.r#type != compiler_lexer::definitions::TokenType::Comment),
    )
    .into_diagnostic()
    .wrap_err("Parser failed")?;

    compiler_codegen_llvm::r#gen(module, ast, Settings { ir, opt, output }).unwrap();
    // .into_diagnostic()
    // .wrap_err("Code generator failed")?;

    Ok(())
}
