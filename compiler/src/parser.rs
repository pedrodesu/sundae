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

trait TokenItExt {
    fn ignore_newlines(self);

    fn consume_token(self, predicate: impl Fn(&Token) -> bool) -> Option<String>;

    fn peek_token(self, predicate: impl Fn(&Token) -> bool) -> bool;

    fn parse_generic_list<T>(
        self,
        left_bound: &str,
        right_bound: &str,
        predicate: impl Fn(TokenIt) -> Option<T>,
        sep_predicate: Option<&str>,
    ) -> Option<Vec<T>>;

    fn parse_block(self) -> Option<Vec<Statement>>;
}

impl TokenItExt for TokenIt<'_> {
    #[inline]
    fn ignore_newlines(self) {
        self.peeking_take_while(|t| t.r#type == TokenType::Newline)
            .for_each(drop)
    }

    #[inline]
    fn consume_token(self, predicate: impl Fn(&Token) -> bool) -> Option<String> {
        self.next()
            .and_then(|t| if predicate(&t) { Some(t.value) } else { None })
    }

    #[inline]
    fn peek_token(self, predicate: impl Fn(&Token) -> bool) -> bool {
        self.next_if(predicate).is_some()
    }

    fn parse_generic_list<T>(
        self,
        left_bound: &str,
        right_bound: &str,
        predicate: impl Fn(TokenIt) -> Option<T>,
        sep_predicate: Option<&str>,
    ) -> Option<Vec<T>> {
        self.ignore_newlines();

        self.consume_token(|t| t.value == left_bound)?;

        let mut buffer = Vec::new();

        loop {
            self.ignore_newlines();

            if self.peek_token(|t| t.value == right_bound) {
                break;
            }

            if let Some(sep_predicate) = sep_predicate {
                if !buffer.is_empty() {
                    self.consume_token(|t| t.value == sep_predicate)?;
                }
            }

            let value = predicate(self)?;
            buffer.push(value);
        }

        Some(buffer)
    }

    #[inline]
    fn parse_block(self) -> Option<Vec<Statement>> {
        self.parse_generic_list("{", "}", |t| Statement::get(t), None)
    }
}

trait Component: Sized + 'static {
    const PARSE_OPTIONS: &'static [fn(TokenIt) -> Option<Self>];

    fn get(tokens: TokenIt) -> Option<Self> {
        Self::PARSE_OPTIONS
            .into_iter()
            .find(|f| f(&mut tokens.clone()).is_some())?(tokens)
    }
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
