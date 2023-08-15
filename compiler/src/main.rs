#![feature(exact_size_is_empty, stmt_expr_attributes)]

use std::{env, fs};

mod lexer;
mod parser;

// TODO add comments and integrate math
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

/*
fn main() {
    let a = math_example::Parser {
        it: vec![
            math_example::Token::Number(2.0),
            math_example::Token::Operator(math_example::Operator::Mul),
            math_example::Token::Number(4.0),
            math_example::Token::Operator(math_example::Operator::Add),
            math_example::Token::Operator(math_example::Operator::LeftBrace),
            math_example::Token::Number(6.0),
            math_example::Token::Operator(math_example::Operator::Sub),
            math_example::Token::Number(9.0),
            math_example::Token::Operator(math_example::Operator::RightBrace),
            math_example::Token::Operator(math_example::Operator::Add),
            math_example::Token::Number(10.0),
            math_example::Token::Operator(math_example::Operator::Mul),
            math_example::Token::Number(4.0),
            math_example::Token::Operator(math_example::Operator::Sub),
            math_example::Token::Operator(math_example::Operator::LeftBrace),
            math_example::Token::Number(9.0),
            math_example::Token::Operator(math_example::Operator::Div),
            math_example::Token::Number(4.0),
            math_example::Token::Operator(math_example::Operator::RightBrace),
        ]
        .into_iter()
        .peekable(),
    }
    .parse_exp();

    println!("{:?}", a);
}
*/
