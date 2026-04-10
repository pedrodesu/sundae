#![feature(trait_alias)]
#![feature(associated_type_defaults)]
#![feature(box_patterns)]

use std::fmt;

use compiler_lexer::definitions::{Token, TokenType};
use ecow::EcoString;
pub use expression::{Expression, binary::Node, operator::Operator};
pub use item::Item;
use iterator::{ExhaustiveGet, TokenIt, TokenItTrait};
use miette::Diagnostic;
pub use statement::Statement;
use thiserror::Error;

pub mod expression;
pub mod item;
mod iterator;
pub mod statement;

#[derive(Error, Debug, Diagnostic, PartialEq)]
#[error(transparent)]
pub enum ParserError
{
    #[error("Expected comma")]
    ExpectedComma,
    #[error("Expected newline")]
    ExpectedNewline,
    #[error("Expected {}", r#type)]
    ExpectedTokenType
    {
        r#type: &'static str
    },
    #[error("Expected {}", value)]
    ExpectedTokenValue
    {
        value: EcoString
    },
    #[error("Unexpected token `{:#?}`", token)]
    UnknownToken
    {
        token: Token
    },
    #[error("Unknown unary with `{:#?}`", token)]
    IllegalUnary
    {
        token: Token
    },
    #[error("Expected {}", name)]
    ExpectedASTStructure
    {
        name: &'static str
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Type(pub Vec<EcoString>);

impl fmt::Display for Type
{
    #[inline]
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

pub struct Parser<'s, I: TokenItTrait>
{
    source: &'s [u8],
    tokens: TokenIt<I>,
}

#[inline(always)]
pub fn parse(input: &[u8], tokens: impl TokenItTrait) -> Result<AST, ParserError>
{
    let mut parser = Parser {
        source: input,
        tokens: TokenIt(tokens.peekable()),
    };
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
