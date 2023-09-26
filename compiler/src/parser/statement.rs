use crate::lexer::definitions::{Token, TokenType};

use super::{
    expression::Expression,
    types::{Name, TokenItTypeExt},
    Component, TokenIt, TokenItBaseExt,
};

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
        tokens.consume(|t| t.r#type == TokenType::Newline)?;

        Some(())
    }

    fn parse_return(tokens: TokenIt) -> Option<Self> {
        tokens.consume(|t| t.value == "ret")?;

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

        tokens.consume(|t| t.value == "=")?;

        let source = Expression::get(tokens)?;

        Self::common(tokens)?;

        Some(Self::Assign {
            destination,
            source,
        })
    }

    #[inline]
    fn parse_local(tokens: TokenIt) -> Option<Self> {
        let identifier = tokens.consume(|t| t.r#type == TokenType::Identifier)?;

        let mutable = tokens.consume(|t| t.value == "mut").is_some();

        let r#type = if matches!(tokens.peek(), Some(Token { ref value, .. }) if value == ":=") {
            None
        } else {
            tokens.get_type()
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
