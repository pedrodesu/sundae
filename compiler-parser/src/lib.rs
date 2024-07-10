#![feature(trait_alias)]
#![feature(let_chains)]
#![feature(const_option)]
#![feature(const_mut_refs)]
#![feature(const_trait_impl)]
#![feature(effects)]
#![feature(iter_advance_by)]

use std::fmt::Debug;

use compiler_lexer::definitions::TokenType;

use ecow::EcoString;
pub use expression::{binary::Node, operator::Operator, Expression};
pub use item::Item;
use iterator::{TokenIt, TokenItTrait};
pub use statement::Statement;

mod expression;
mod item;
mod iterator;
mod statement;

#[derive(Debug, Clone, PartialEq)]
pub struct Type(pub Vec<EcoString>);

#[derive(Debug, Clone)]
pub struct ArgumentName(pub EcoString, pub Type);

#[derive(Debug, Clone, PartialEq)]
pub struct Name(pub EcoString, pub Option<Type>);

pub trait ExhaustiveGet<'a, I: TokenItTrait + 'a>: Sized + 'a + std::fmt::Debug {
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
        if iterator.next(|t| t.r#type == TokenType::Newline).is_none() {
            items.push(Item::get(&mut iterator).unwrap());
        }
    }

    AST(items)
}
