use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use clap::Parser;
use compiler_codegen_llvm::Settings;
use compiler_lexer::{
    LexerEvent,
    definitions::{Token, TokenType},
};
use itertools::Itertools;
use miette::{Context, Diagnostic, IntoDiagnostic, NamedSource, Report, Result, bail};
use mimalloc::MiMalloc;
use thiserror::Error;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(clap::Parser)]
#[command(version, about)]
struct Args {
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

fn path_is_valid_file(s: &str) -> Result<PathBuf> {
    let path = Path::new(s);
    if path.is_file() {
        Ok(path.to_owned())
    } else {
        bail!("Path isn't a valid file")
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error("Compilation failed with {} diagnostic(s)", diagnostics.len())]
struct FrontendDiagnostics {
    #[related]
    diagnostics: Vec<Report>,
}

impl FrontendDiagnostics {
    #[inline]
    const fn new(diagnostics: Vec<Report>) -> Self {
        Self { diagnostics }
    }
}

fn main() -> Result<()> {
    let Args {
        ir,
        opt,
        output,
        source,
    } = Args::parse();

    let file = Arc::new(NamedSource::new(
        source.display().to_string(),
        fs::read_to_string(&source)
            .into_diagnostic()
            .wrap_err_with(|| format!("Couldn't read file from path `{}`", source.display()))?,
    ));

    let module = source
        .file_stem()
        .and_then(|s| s.to_str())
        .wrap_err("Incorrect file name")?;

    let (tokens, mut diagnostics): (Vec<Token>, Vec<Report>) =
        compiler_lexer::tokenize(file.inner())
            .filter_map(|event| match event {
                LexerEvent::Token(token) => {
                    (token.r#type != TokenType::Comment).then_some(Ok(token))
                }
                LexerEvent::Error(error) => {
                    Some(Err(Report::new(error).with_source_code(file.clone())))
                }
            })
            .partition_result();

    let ast = match compiler_parser::parse(file.inner(), tokens.into_iter()) {
        Ok(ast) => ast,
        Err(error) => {
            let error = Report::new(error).with_source_code(file.clone());

            if diagnostics.is_empty() {
                return Err(error);
            }

            diagnostics.push(error);
            return Err(Report::new(FrontendDiagnostics::new(diagnostics)));
        }
    };

    if !diagnostics.is_empty() {
        return Err(Report::new(FrontendDiagnostics::new(diagnostics)));
    }

    compiler_codegen_llvm::r#gen(module, ast, Settings { ir, opt, output }).unwrap();
    // .into_diagnostic()
    // .wrap_err("Code generator failed")?;

    Ok(())
}
