use std::{fmt, iter::Peekable};

use compiler_lexer::definitions::{Token, TokenType};
use itertools::Itertools;

use crate::{ExhaustiveGet, Statement};

pub trait TokenItTrait = Iterator<Item = Token> + Clone + fmt::Debug;

#[derive(Clone)]
pub struct TokenIt<I: TokenItTrait>(pub Peekable<I>);

impl<I: Iterator<Item = Token> + Clone + fmt::Debug> TokenIt<I> {
    #[inline]
    pub fn ignore_newlines(&mut self) {
        self.0
            .peeking_take_while(|t| matches!(t.r#type, TokenType::Newline))
            .for_each(drop)
    }

    #[inline]
    pub fn next(&mut self, predicate: impl FnOnce(&Token) -> bool) -> Option<Token> {
        /*
        let mut it = self.0.clone();

        let increment = it
            .by_ref()
            .peeking_take_while(|t| t.r#type == TokenType::Newline)
            .count();

        if let t @ Some(_) = it.next_if(predicate) {
            self.0.advance_by(increment + 1).unwrap();
            t
        } else {
            None
        }
        */
        self.0.next_if(predicate)
    }

    pub fn parse_generic_list<T>(
        &mut self,
        (left_bound, right_bound): (&str, &str),
        predicate: impl Fn(&mut Self) -> Option<T>,
        sep_predicate: Option<&str>,
    ) -> Option<Vec<T>> {
        self.ignore_newlines();

        self.next(|t| t.value == left_bound)?;

        let mut buffer = Vec::new();

        loop {
            self.ignore_newlines();

            if self.next(|t| t.value == right_bound).is_some() {
                break;
            }

            if let Some(sep_predicate) = sep_predicate {
                if !buffer.is_empty() {
                    self.next(|t| t.value == sep_predicate)?;
                }
            }

            let value = predicate(self)?;
            buffer.push(value);
        }

        Some(buffer)
    }

    #[inline]
    pub fn parse_block(&mut self) -> Option<Vec<Statement>> {
        self.parse_generic_list(("{", "}"), |t| Statement::get(t), None)
    }
}
