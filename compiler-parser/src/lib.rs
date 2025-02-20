#![feature(trait_alias)]
#![feature(let_chains)]
#![feature(associated_type_defaults)]

use std::fmt;

use anyhow::Result;
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

#[derive(Clone, Debug, PartialEq)]
pub struct Type(pub Vec<EcoString>);

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join("::"))
    }
}

#[derive(Debug, Clone)]
pub struct ArgumentName(pub EcoString, pub Type);

#[derive(Clone, Debug, PartialEq)]
pub struct Name(pub EcoString, pub Option<Type>);

pub trait ExhaustiveGet<'a, I: TokenItTrait + 'a>: Sized + 'a {
    type ParsePredicate = fn(&mut TokenIt<I>) -> Option<Self>;

    // TODO refactor from Option<Self> to Option<Result<Self>>, handle possible error cases (only Item::parse_function was)
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
pub fn parse(input: impl TokenItTrait) -> Result<AST> {
    let mut iterator = TokenIt(input.peekable());
    let mut items = Vec::new();

    while iterator.0.peek().is_some() {
        if iterator.next(|t| t.r#type == TokenType::Newline).is_none() {
            // let item = Item::get(&mut iterator).unwrap()?;
            // items.push(item);
            items.push(Item::get(&mut iterator).unwrap());
        }
    }

    Ok(AST(items))
}
