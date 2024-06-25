use compiler_lexer::definitions::TokenType;
use itertools::Itertools;

use crate::{expression::Expression, ExhaustiveGet, Name, TokenIt, Type};

#[derive(Debug, Clone)]
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

impl ExhaustiveGet for Statement {
    const PARSE_OPTIONS: &'static [fn(&mut TokenIt) -> Option<Self>] = &[
        Self::parse_return,
        Self::parse_expression,
        Self::parse_assign,
        Self::parse_local,
    ];
}

impl Statement {
    #[inline]
    fn assert_end(
        tokens: &mut TokenIt,
        predicate: impl Fn(&mut TokenIt) -> Option<Self>,
    ) -> Option<Self> {
        let value = predicate(tokens)?;

        tokens.consume(|t| t.r#type == TokenType::Newline)?;

        Some(value)
    }

    #[inline]
    pub fn parse_return(tokens: &mut TokenIt) -> Option<Self> {
        Self::assert_end(tokens, |tokens| {
            tokens.consume(|t| t.value == "ret")?;

            Some(Self::Return(
                if tokens.0.peek()?.r#type != TokenType::Newline {
                    Some(Expression::get(tokens)?) // needs Some(x)? to assert it gets a valid expression
                } else {
                    None
                },
            ))
        })
    }

    #[inline]
    pub fn parse_expression(tokens: &mut TokenIt) -> Option<Self> {
        Self::assert_end(tokens, |tokens| {
            Expression::get(tokens).map(Self::Expression)
        })
    }

    pub fn parse_assign(tokens: &mut TokenIt) -> Option<Self> {
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

    pub fn parse_local(tokens: &mut TokenIt) -> Option<Self> {
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
