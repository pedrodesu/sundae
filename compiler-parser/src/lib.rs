#![feature(trait_alias)]
#![feature(let_chains)]
#![feature(associated_type_defaults)]
#![feature(box_patterns)]
// #![feature(deref_patterns)] // https://doc.rust-lang.org/beta/unstable-book/language-features/deref-patterns.html // this will make testing much easier!

use std::fmt;

use compiler_lexer::definitions::{Token, TokenType};
use ecow::EcoString;
pub use expression::{Expression, binary::Node, operator::Operator};
pub use item::Item;
use iterator::{ExhaustiveGet, TokenIt, TokenItTrait};
use snafu::Snafu;
pub use statement::Statement;

pub mod expression;
pub mod item;
mod iterator;
pub mod statement;

// TODO maybe segment this across the multiple modules?
#[derive(Debug, Snafu, PartialEq)]
pub enum ParserError
{
    #[snafu(display("Expected comma"))]
    ExpectedComma,
    #[snafu(display("Expected newline"))]
    ExpectedNewline,
    #[snafu(display("Expected {}", r#type))]
    ExpectedTokenType
    {
        r#type: &'static str
    },
    #[snafu(display("Expected {}", value))]
    ExpectedTokenValue
    {
        value: EcoString
    },
    #[snafu(display("Unexpected token `{:#?}`", token))]
    UnknownToken
    {
        token: Token
    },
    #[snafu(display("Unknown unary with `{:#?}`", token))]
    IllegalUnary
    {
        token: Token
    },
    #[snafu(display("Expected {}", name))]
    ExpectedASTStructure
    {
        name: &'static str
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Type(pub Vec<EcoString>);

impl fmt::Display for Type
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}", self.0.join("."))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArgumentName(pub EcoString, pub Type);

#[derive(Clone, Debug, PartialEq)]
pub struct Name(pub EcoString, pub Option<Type>);

#[derive(Debug, PartialEq)]
pub struct AST(pub Vec<Item>);

#[inline(always)]
pub fn parse(input: impl TokenItTrait) -> Result<AST, ParserError>
{
    let mut iterator = TokenIt(input.peekable());
    let mut items = Vec::new();

    while iterator.0.peek().is_some()
    {
        if iterator.next(|t| t.r#type == TokenType::Newline).is_none()
        {
            items.push(Item::get(&mut iterator)?);
        }
    }

    Ok(AST(items))
}
