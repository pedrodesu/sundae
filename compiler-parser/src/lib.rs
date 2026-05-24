#![feature(trait_alias)]
#![feature(associated_type_defaults)]
#![feature(box_patterns)]

use std::{fmt, iter::Peekable};

use compiler_lexer::definitions::{Span, Token};
pub use expression::{Expression, binary::Node, operator::Operator};
pub use item::Item;
use iterator::ExhaustiveGet;
use miette::Diagnostic;
pub use statement::Statement;
use thiserror::Error;

use crate::iterator::TokenIt;

pub mod expression;
pub mod item;
mod iterator;
pub mod statement;

#[derive(Error, Debug, Diagnostic, PartialEq)]
pub enum ParserError {
    #[error("Expected comma")]
    ExpectedComma {
        #[label("Here")]
        span: Span,
    },
    #[error("Expected newline")]
    ExpectedNewline {
        #[label("Here")]
        span: Span,
    },
    #[error("Expected {}", r#type)]
    ExpectedTokenType {
        #[label("Here")]
        span: Span,
        r#type: &'static str,
    },
    #[error("Expected {}", value)]
    ExpectedTokenValue {
        #[label("Here")]
        span: Span,
        value: &'static str,
    },
    #[error("Unexpected token `{:#?}`", token)]
    UnknownToken {
        #[label("Here")]
        span: Span,
        token: Token,
    },
    #[error("Unknown unary with `{:#?}`", token)]
    IllegalUnary {
        #[label("Here")]
        span: Span,
        token: Token,
    },
    #[error("Expected {}", name)]
    ExpectedASTStructure {
        #[label("Here")]
        span: Span,
        name: &'static str,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Type<'s>(pub Vec<&'s [u8]>);

impl fmt::Display for Type<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().enumerate().try_for_each(|(i, &c)| {
            if i > 0 {
                f.write_str(".")?;
            }
            f.write_str(&String::from_utf8_lossy(c))?;
            Ok(())
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArgumentName<'s>(pub &'s [u8], pub Type<'s>);

#[derive(Clone, Debug, PartialEq)]
pub struct Name<'s>(pub &'s [u8], pub Option<Type<'s>>);

#[derive(Debug, PartialEq)]
pub struct AST<'s>(pub Vec<Item<'s>>);

#[derive(Clone)]
pub struct Parser<'s, I: TokenIt> {
    source: &'s [u8],
    tokens: Peekable<I>,
}

impl<'s, I: TokenIt> Parser<'s, I> {
    #[inline]
    pub fn new<S: AsRef<[u8]> + ?Sized>(source: &'s S, tokens: I) -> Self {
        Self {
            source: source.as_ref(),
            tokens: tokens.peekable(),
        }
    }

    #[inline]
    pub fn current_span(&mut self) -> Span {
        self.tokens
            .peek()
            .map_or(Span::empty(self.source.len()), |t| t.span)
    }

    #[inline]
    pub fn token_value(&self, token: &Token) -> &'s [u8] {
        token.value(self.source)
    }
}

impl<'s, I: TokenIt> Parser<'s, I> {
    #[inline]
    pub fn next_item(&mut self) -> Result<Item<'s>, ParserError> {
        Item::get(self)
    }

    #[inline]
    pub fn next_statement(&mut self) -> Result<Statement<'s>, ParserError> {
        Statement::get(self)
    }

    #[inline]
    pub fn next_expression(&mut self) -> Result<Expression<'s>, ParserError> {
        Expression::get(self)
    }
}

#[inline(always)]
pub fn parse<'s, S: AsRef<[u8]> + ?Sized>(
    source: &'s S,
    tokens: impl TokenIt + 's,
) -> Result<AST<'s>, ParserError> {
    let mut parser = Parser::new(source, tokens);
    let mut items = Vec::new();

    loop {
        parser.ignore_newlines();

        if parser.tokens.peek().is_none() {
            break;
        }

        items.push(parser.next_item()?);
    }

    Ok(AST(items))
}
