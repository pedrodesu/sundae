use std::iter::Peekable;

use compiler_lexer::definitions::{Token, TokenType};
use itertools::Itertools;

use crate::{ExhaustiveGet, Statement};

pub trait TokenItTrait = Iterator<Item = Token> + Clone;

#[derive(Clone)]
pub struct TokenIt<I: TokenItTrait>(pub Peekable<I>);

impl<I: TokenItTrait> TokenIt<I> {
    #[inline]
    pub fn ignore_newlines(&mut self) {
        self.0
            .peeking_take_while(|t| matches!(t.r#type, TokenType::Newline))
            .for_each(drop)
    }

    #[inline]
    pub fn consume(&mut self, predicate: impl Fn(&Token) -> bool) -> Option<String> {
        self.0.next_if(predicate).map(|t| t.value)
    }

    pub fn parse_generic_list<T>(
        &mut self,
        left_bound: &str,
        right_bound: &str,
        predicate: impl Fn(&mut Self) -> Option<T>,
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
    pub fn parse_block(&mut self) -> Option<Vec<Statement>> {
        self.parse_generic_list("{", "}", |t| Statement::get(t), None)
    }
}
