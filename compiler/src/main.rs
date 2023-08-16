#![feature(exact_size_is_empty, stmt_expr_attributes)]

use std::{env, fs};

mod lexer;
mod parser;

// TODO add comments (catch comments on lexer and filter them out on collection)
fn main() {
    let path = env::args().nth(1).expect("no path");
    let file = fs::read_to_string(path).expect("couldn't read file");

    let tokens = lexer::tokenize(&file);
    println!(
        "{}",
        tabled::Table::new(tokens.clone()).with(tabled::settings::Style::sharp())
    );

    let ast = parser::parse(tokens);
    println!("{:#?}", ast);
}
