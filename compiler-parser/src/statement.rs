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
    fn common(tokens: &mut TokenIt) -> Option<()> {
        tokens.consume(|t| t.r#type == TokenType::Newline)?;

        Some(())
    }

    pub fn parse_return(tokens: &mut TokenIt) -> Option<Self> {
        tokens.consume(|t| t.value == "ret")?;

        let expr = if tokens.0.peek()?.r#type != TokenType::Newline {
            Some(Expression::get(tokens)?)
        } else {
            None
        };

        Self::common(tokens)?;

        Some(Self::Return(expr))
    }

    #[inline]
    pub fn parse_expression(tokens: &mut TokenIt) -> Option<Self> {
        let expr = Expression::get(tokens)?;

        Self::common(tokens)?;

        Some(Self::Expression(expr))
    }

    pub fn parse_assign(tokens: &mut TokenIt) -> Option<Self> {
        let destination = Expression::get(tokens)?;

        tokens.consume(|t| t.value == "=")?;

        let source = Expression::get(tokens)?;

        Self::common(tokens)?;

        Some(Self::Assign {
            destination,
            source,
        })
    }

    pub fn parse_local(tokens: &mut TokenIt) -> Option<Self> {
        let identifier = tokens.consume(|t| t.r#type == TokenType::Identifier)?;

        let mutable = tokens.consume(|t| t.value == "mut").is_some();

        // TODO use actual format to get type info
        let r#type = if tokens.0.peek()?.value != ":=" {
            Some(Type(
                tokens
                    .0
                    .peeking_take_while(|t| t.value != ":=")
                    .map(|t| t.value)
                    .collect(),
            ))
        } else {
            None
        };

        let init = if tokens.consume(|t| t.value == ":=").is_some() {
            Expression::get(tokens)
        } else {
            None
        };

        Self::common(tokens)?;

        Some(Self::Local {
            name: Name(identifier, r#type),
            mutable,
            init,
        })
    }
}
