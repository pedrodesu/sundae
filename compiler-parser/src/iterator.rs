use std::iter::Peekable;

use compiler_lexer::definitions::{Token, TokenType};
use ecow::EcoVec;
use itertools::Itertools;

use crate::{ParserError, Statement};

pub trait TokenItTrait = Iterator<Item = Token> + Clone;

#[derive(Clone)]
pub struct TokenIt<I: TokenItTrait>(pub Peekable<I>);

pub trait ExhaustiveGet<I: TokenItTrait>: Sized
{
    type ParsePredicate = fn(&mut TokenIt<I>) -> Result<Self, ParserError>;

    fn find_predicate(
        tokens: &mut TokenIt<I>,
    ) -> Result<fn(&mut TokenIt<I>) -> Result<Self, ParserError>, ParserError>;

    #[inline]
    fn get(tokens: &mut TokenIt<I>) -> Result<Self, ParserError>
    {
        (Self::find_predicate(&mut tokens.clone())?)(tokens)
    }
}

impl<I: TokenItTrait> TokenIt<I>
{
    #[inline]
    pub fn ignore_newlines(&mut self)
    {
        self.0
            .peeking_take_while(|t| matches!(t.r#type, TokenType::Newline)) // TODO switch to this syntax everywhere when `deref_patterns` is usable
            .for_each(drop)
    }

    #[inline]
    pub fn next(&mut self, predicate: impl FnOnce(&Token) -> bool) -> Option<Token>
    {
        self.0
            .by_ref()
            .take_while_ref(|t| t.r#type == TokenType::Newline)
            .for_each(drop);

        self.0.next_if(predicate)
        // self.0.next_if(predicate)
    }

    pub fn consume_generic_list<T: Clone>(
        &mut self,
        (left_bound, right_bound): (&str, &str),
        predicate: impl Fn(&mut Self) -> Result<T, ParserError>,
        sep_predicate: Option<&str>,
    ) -> Result<EcoVec<T>, ParserError>
    {
        self.next(|t| t.value == left_bound)
            .ok_or_else(|| ParserError::ExpectedTokenValue {
                value: left_bound.into(),
            })?;

        let mut buffer = EcoVec::new();

        loop
        {
            self.ignore_newlines();

            if self.next(|t| t.value == right_bound).is_some()
            {
                break;
            }

            if let Some(sep_predicate) = sep_predicate
            {
                if !buffer.is_empty()
                {
                    let Some(_) = self.next(|t| t.value == sep_predicate)
                    else
                    {
                        return Err(ParserError::ExpectedComma);
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
    pub fn consume_block(&mut self) -> Result<EcoVec<Statement>, ParserError>
    {
        self.consume_generic_list(("{", "}"), Statement::get, None)
    }
}
