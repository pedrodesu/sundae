use compiler_lexer::definitions::TokenType;
use itertools::Itertools;

use crate::{expression::Expression, iterator::TokenItTrait, ExhaustiveGet, Name, TokenIt, Type};

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Statement {
    Return(Option<Expression>),
    Expression(Expression),
    Assign {
        destination: Expression,
        source: Expression,
    },
    Local {
        mutable: bool,
        name: Name,
        init: Option<Expression>,
    },
}

impl<'a, I: TokenItTrait + 'a> ExhaustiveGet<'a, I> for Statement {
    const PARSE_OPTIONS: &'a [fn(&mut TokenIt<I>) -> Option<Self>] = &[
        Self::parse_return,
        Self::parse_expression,
        Self::parse_assign,
        Self::parse_local,
    ];
}

impl Statement {
    #[inline]
    fn assert_end<I: TokenItTrait>(
        tokens: &mut TokenIt<I>,
        predicate: impl Fn(&mut TokenIt<I>) -> Option<Self>,
    ) -> Option<Self> {
        let value = predicate(tokens)?;

        tokens.consume(|t| t.r#type == TokenType::Newline)?;

        Some(value)
    }

    #[inline]
    pub fn parse_return<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Option<Self> {
        Self::assert_end(tokens, |tokens| {
            tokens.consume(|t| t.value == "ret")?;

            Some(Self::Return(
                if tokens.0.peek()?.r#type != TokenType::Newline {
                    Some(Expression::get(tokens)?)
                } else {
                    None
                },
            ))
        })
    }

    #[inline]
    pub fn parse_expression<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Option<Self> {
        Self::assert_end(tokens, |tokens| {
            Expression::get(tokens).map(Self::Expression)
        })
    }

    pub fn parse_assign<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Option<Self> {
        Self::assert_end(tokens, |tokens| {
            let destination = Expression::get(tokens)?;

            tokens.consume(|t| t.value == "=")?;

            let source = Expression::get(tokens)?;

            Some(Self::Assign {
                destination,
                source,
            })
        })
    }

    pub fn parse_local<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Option<Self> {
        Self::assert_end(tokens, |tokens| {
            let identifier = tokens.consume(|t| t.r#type == TokenType::Identifier)?;

            let mutable = tokens.consume(|t| t.value == "mut").is_some();

            let r#type = if tokens.0.peek()?.value != ":=" {
                Some(Type(
                    tokens
                        .0
                        .peeking_take_while(|t| t.value != ":=" && t.r#type != TokenType::Newline)
                        .map(|t| t.value)
                        .collect(),
                ))
            } else {
                None
            };

            let init = tokens
                .consume(|t| t.value == ":=")
                .and_then(|_| Expression::get(tokens));

            Some(Self::Local {
                name: Name(identifier, r#type),
                mutable,
                init,
            })
        })
    }
}
