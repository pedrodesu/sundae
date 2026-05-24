use compiler_lexer::definitions::{Token, TokenType};
use ecow::EcoVec;
use itertools::Itertools;

use crate::{Parser, ParserError, Statement};

pub trait TokenIt = Iterator<Item = Token> + Clone;

pub trait ExhaustiveGet<I: TokenIt>: Sized {
    fn find_predicate<'s>(
        parser: &mut Parser<'s, I>,
    ) -> Result<fn(&mut Parser<'s, I>) -> Result<Self, ParserError>, ParserError>;

    #[inline]
    fn get<'s>(parser: &mut Parser<'s, I>) -> Result<Self, ParserError> {
        let predicate = Self::find_predicate(&mut parser.clone())?;
        predicate(parser)
    }
}

impl<I: TokenIt> Parser<'_, I> {
    #[inline]
    pub fn peek_value(&mut self, value: &str) -> bool {
        self.tokens
            .peek()
            .is_some_and(|token| token.is_value(self.source, value))
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
        self.ignore_newlines();

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
    pub fn consume_block(&mut self) -> Result<EcoVec<Statement>, ParserError> {
        self.consume_list(("{", "}"), Self::next_statement, None)
    }
}
