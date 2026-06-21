use compiler_lexer::definitions::{Token, TokenType};
use ecow::EcoVec;
use itertools::Itertools;

use crate::{Parser, ParserError, Statement};

pub trait TokenIt = Iterator<Item = Token> + Clone;

pub trait ExhaustiveGet<'s, I: TokenIt>: Sized {
    fn get(parser: &mut Parser<'s, I>) -> Result<Self, ParserError>;
}

impl<'s, I: TokenIt> Parser<'s, I> {
    #[inline]
    pub fn peek_value(&mut self, value: &str) -> bool {
        // TODO shouldn't we have ignore_newlines here?

        self.tokens
            .peek()
            .is_some_and(|t| t.is_value(self.source, value))
    }

    #[inline]
    pub fn ignore_newlines(&mut self) {
        self.tokens
            .peeking_take_while(|t| {
                matches!(
                    t,
                    Token {
                        r#type: TokenType::Newline,
                        ..
                    }
                )
            })
            .for_each(drop)
    }

    #[inline]
    pub fn consume(&mut self, predicate: impl FnOnce(&Token) -> bool) -> Option<Token> {
        self.tokens.next_if(predicate)
    }

    #[inline]
    pub fn consume_value(&mut self, value: &str) -> Option<Token> {
        self.consume(|t| t.is_value(self.source, value))
    }

    pub fn consume_list<T: Clone>(
        &mut self,
        (left_bound, right_bound): (&'static str, &'static str),
        predicate: impl Fn(&mut Self) -> Result<T, ParserError>,
        separator: Option<&str>,
    ) -> Result<EcoVec<T>, ParserError> {
        self.consume_value(left_bound)
            .ok_or_else(|| ParserError::ExpectedTokenValue {
                span: self.current_span(),
                value: left_bound,
            })?;

        let mut buffer = EcoVec::new();

        loop {
            self.ignore_newlines();

            if self.consume_value(right_bound).is_some() {
                break;
            }

            if let Some(sep_predicate) = separator {
                if !buffer.is_empty() {
                    let Some(_) = self.consume_value(sep_predicate) else {
                        return Err(ParserError::ExpectedComma {
                            span: self.current_span(),
                        });
                    };
                }
                self.ignore_newlines();
            }

            let value = predicate(self)?;
            buffer.push(value);

            self.ignore_newlines();
        }

        Ok(buffer)
    }

    #[inline]
    pub fn consume_block(&mut self) -> Result<EcoVec<Statement<'s>>, ParserError> {
        self.consume_list(("{", "}"), Self::next_statement, None)
    }
}
