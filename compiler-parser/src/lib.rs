#![feature(trait_alias)]

use std::fmt::Debug;

use compiler_lexer::definitions::TokenType;

use ecow::EcoString;
pub use expression::{
    binary::{BinaryNode, Operator},
    Expression,
};
pub use item::Item;
use iterator::{TokenIt, TokenItTrait};
pub use statement::Statement;

mod expression;
mod item;
mod iterator;
mod statement;

#[derive(Debug, Clone)]
pub struct Type(pub Vec<EcoString>);

#[derive(Debug, Clone)]
pub struct ArgumentName(pub EcoString, pub Type);

#[derive(Debug, Clone)]
pub struct Name(pub EcoString, pub Option<Type>);

pub trait ExhaustiveGet<'a, I: TokenItTrait + 'a>: Sized + 'a {
    const PARSE_OPTIONS: &'a [fn(&mut TokenIt<I>) -> Option<Self>];

    fn get(tokens: &mut TokenIt<I>) -> Option<Self> {
        Self::PARSE_OPTIONS
            .into_iter()
            .find(|f| f(&mut tokens.clone()).is_some())?(tokens)
    }
}

#[derive(Debug)]
pub struct AST(pub Vec<Item>);

#[inline(always)]
pub fn parse<'a>(input: impl TokenItTrait) -> AST {
    let mut iterator = TokenIt(input.peekable());
    let mut items = Vec::new();

    while iterator.0.peek().is_some() {
        if iterator
            .consume(|t| t.r#type == TokenType::Newline)
            .is_none()
        {
            items.push(Item::get(&mut iterator).unwrap());
        }
    }

    AST(items)
}
