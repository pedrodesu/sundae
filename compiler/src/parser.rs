use std::{fmt::Debug, iter::Peekable, vec};

use itertools::Itertools;

use crate::lexer::definitions::{Token, TokenType};

use self::{item::Item, statement::Statement};

mod expression;
mod item;
mod statement;
mod types;

#[derive(Debug)]
pub struct AST(pub Vec<Item>);

type TokenIt<'a> = &'a mut Peekable<vec::IntoIter<Token>>;

trait Component: Sized + 'static {
    const PARSE_OPTIONS: &'static [fn(TokenIt) -> Option<Self>];

    fn get(tokens: TokenIt) -> Option<Self> {
        Self::PARSE_OPTIONS
            .into_iter()
            .find(|f| f(&mut tokens.clone()).is_some())?(tokens)
    }
}

fn parse_generic_list<T>(
    tokens: TokenIt,
    left_bound: &str,
    right_bound: &str,
    predicate: impl Fn(TokenIt) -> Option<T>,
    sep_predicate: Option<&str>,
) -> Option<Vec<T>> {
    ignore_newlines(tokens);

    consume(tokens, |t| t.value == left_bound)?;

    let mut buffer = Vec::new();

    loop {
        ignore_newlines(tokens);

        if peek(tokens, |t| t.value == right_bound) {
            break;
        }

        if let Some(sep_predicate) = sep_predicate {
            if !buffer.is_empty() {
                consume(tokens, |t| t.value == sep_predicate)?;
            }
        }

        let value = predicate(tokens)?;
        buffer.push(value);
    }

    Some(buffer)
}

#[inline]
fn parse_block(tokens: TokenIt) -> Option<Vec<Statement>> {
    parse_generic_list(tokens, "{", "}", |t| Statement::get(t), None)
}

#[inline]
fn ignore_newlines(tokens: TokenIt) {
    tokens
        .peeking_take_while(|t| t.r#type == TokenType::Newline)
        .for_each(drop)
}

#[inline]
fn consume(tokens: TokenIt, predicate: impl Fn(&Token) -> bool) -> Option<String> {
    tokens
        .next()
        .and_then(|t| if predicate(&t) { Some(t.value) } else { None })
}

#[inline]
fn peek(tokens: TokenIt, predicate: impl Fn(&Token) -> bool) -> bool {
    tokens.next_if(predicate).is_some()
}

#[inline(always)]
pub fn parse(input: Vec<Token>) -> AST {
    let mut iterator = input.into_iter().peekable();
    let mut items = Vec::new();

    while iterator.peek().is_some() {
        if iterator
            .next_if(|t| t.r#type == TokenType::Newline)
            .is_none()
        {
            items.push(Item::get(&mut iterator).unwrap());
        }
    }

    AST(items)
}
