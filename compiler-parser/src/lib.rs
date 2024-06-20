use std::{fmt::Debug, iter::Peekable, vec};

use compiler_lexer::definitions::{Token, TokenType};
use itertools::Itertools;

pub use expression::{
    binary::{BinaryNode, Operator},
    Expression,
};
pub use item::Item;
pub use statement::Statement;

mod expression;
mod item;
mod statement;

#[derive(Clone)]
pub struct TokenIt(Peekable<vec::IntoIter<Token>>);

#[derive(Debug, Clone)]
pub struct Type(pub Vec<String>);

#[derive(Debug, Clone)]
pub struct ArgumentName(pub String, pub Type);

#[derive(Debug, Clone)]
pub struct Name(pub String, pub Option<Type>);

pub trait ExhaustiveGet: Sized + 'static {
    const PARSE_OPTIONS: &'static [fn(&mut TokenIt) -> Option<Self>];

    fn get(tokens: &mut TokenIt) -> Option<Self> {
        Self::PARSE_OPTIONS
            .into_iter()
            .find(|f| f(&mut tokens.clone()).is_some())?(tokens)
    }
}

impl TokenIt {
    #[inline]
    fn ignore_newlines(&mut self) {
        self.0
            .peeking_take_while(|t| matches!(t.r#type, TokenType::Newline))
            .for_each(drop)
    }

    #[inline]
    fn consume(&mut self, predicate: impl Fn(&Token) -> bool) -> Option<String> {
        self.0.next_if(predicate).map(|t| t.value)
    }

    fn parse_generic_list<T>(
        &mut self,
        left_bound: &str,
        right_bound: &str,
        predicate: impl Fn(&mut TokenIt) -> Option<T>,
        sep_predicate: Option<&str>,
    ) -> Option<Vec<T>> {
        self.ignore_newlines();

        self.consume(|t| t.value == left_bound)?;

        let mut buffer = Vec::new();

        loop {
            self.ignore_newlines();

            if self.consume(|t| t.value == right_bound).is_some() {
                break;
            }

            if let Some(sep_predicate) = sep_predicate {
                if !buffer.is_empty() {
                    self.consume(|t| t.value == sep_predicate)?;
                }
            }

            let value = predicate(self)?;
            buffer.push(value);
        }

        Some(buffer)
    }

    #[inline]
    fn parse_block(&mut self) -> Option<Vec<Statement>> {
        self.parse_generic_list("{", "}", |t| Statement::get(t), None)
    }
}

#[derive(Debug)]
pub struct AST(pub Vec<Item>);

#[inline(always)]
pub fn parse(input: Vec<Token>) -> AST {
    let mut iterator = TokenIt(input.into_iter().peekable());
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
