use crate::lexer::definitions::TokenType;

use super::{
    expression::Expression,
    types::{get_type, Name},
    Component, TokenIt, TokenItExt,
};

#[derive(Debug, Clone)]
pub enum Statement {
    Return(Option<Expression>),
    Expression(Expression),
    Assign {
        destination: Box<Expression>,
        source: Box<Expression>,
    },
    Local {
        mutable: bool,
        name: Name,
        init: Option<Expression>,
    },
}

impl super::Component for Statement {
    const PARSE_OPTIONS: &'static [fn(TokenIt) -> Option<Self>] = &[
        Self::parse_return,
        Self::parse_expression,
        Self::parse_assign,
        Self::parse_local,
    ];
}

impl Statement {
    #[inline]
    fn common(tokens: TokenIt) -> Option<()> {
        tokens.consume_token(|t| t.r#type == TokenType::Newline)?;

        Some(())
    }

    fn parse_return(tokens: TokenIt) -> Option<Self> {
        tokens.consume_token(|t| t.value == "ret")?;

        let expr = if tokens.peek()?.r#type != TokenType::Newline {
            Some(Expression::get(tokens)?)
        } else {
            None
        };

        Self::common(tokens)?;

        Some(Self::Return(expr))
    }

    #[inline]
    fn parse_expression(tokens: TokenIt) -> Option<Self> {
        let expr = Expression::get(tokens)?;

        Self::common(tokens)?;

        Some(Self::Expression(expr))
    }

    #[inline]
    fn parse_assign(tokens: TokenIt) -> Option<Self> {
        let destination = Expression::get(tokens)?;

        tokens.consume_token(|t| t.value == "=")?;

        let source = Expression::get(tokens)?;

        Self::common(tokens)?;

        Some(Self::Assign {
            destination: Box::new(destination),
            source: Box::new(source),
        })
    }

    #[inline]
    fn parse_local(tokens: TokenIt) -> Option<Self> {
        tokens.consume_token(|t| t.value == "let")?;

        let mutable = tokens.peek_token(|t| t.value == "mut");

        let identifier = tokens.consume_token(|t| t.r#type == TokenType::Identifier)?;

        let (r#type, init) = if tokens.peek_token(|t| t.value == "=") {
            (None, Expression::get(tokens))
        } else {
            let r#type = get_type(tokens);
            let init = if tokens.peek_token(|t| t.value == "=") {
                Expression::get(tokens)
            } else {
                None
            };
            (r#type, init)
        };

        Self::common(tokens)?;

        Some(Self::Local {
            name: Name(identifier, r#type),
            mutable,
            init,
        })
    }
}
