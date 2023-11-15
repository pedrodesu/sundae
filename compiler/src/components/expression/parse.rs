use crate::{
    lexer::definitions::{Token, TokenType},
    parser::{Component, TokenIt, TokenItBaseExt},
};

use super::Expression;

impl Component for Expression {
    const PARSE_OPTIONS: &'static [fn(TokenIt) -> Option<Self>] = &[
        Self::parse_if,
        Self::parse_binary,
        Self::parse_literal,
        Self::parse_call,
        Self::parse_path,
    ];
}

impl Expression {
    #[inline]
    fn parse_literal(tokens: TokenIt) -> Option<Self> {
        if let Token {
            value,
            r#type: TokenType::Literal(lit_type),
        } = tokens.next()?
        {
            Some(Self::Literal {
                value,
                r#type: lit_type,
            })
        } else {
            None
        }
    }

    fn parse_path(tokens: TokenIt) -> Option<Self> {
        let mut path = Vec::new();

        while path.is_empty() || tokens.consume(|t| t.value == ".").is_some() {
            let segment = tokens.consume(|t| t.r#type == TokenType::Identifier)?;
            path.push(segment);
        }

        Some(Self::Path(path))
    }

    #[inline]
    pub fn parse_binary(tokens: TokenIt) -> Option<Self> {
        Some(Self::Binary(super::binary::parse::parse(tokens)?))
    }

    fn parse_call(tokens: TokenIt) -> Option<Self> {
        let Expression::Path(path) = Self::parse_path(tokens)? else {
            unreachable!()
        };

        let args = tokens.parse_generic_list("(", ")", |t| Expression::get(t), Some(","))?;

        Some(Self::Call { path, args })
    }

    fn parse_if(tokens: TokenIt) -> Option<Self> {
        tokens.consume(|t| t.value == "if")?;

        let condition = Expression::get(tokens)?;

        let block = tokens.parse_block()?;

        let mut tokens_clone = tokens.clone();
        tokens_clone.ignore_newlines();

        let r#else = if tokens_clone.consume(|t| t.value == "else").is_some() {
            *tokens = tokens_clone;
            tokens.parse_block()
        } else {
            None
        };

        Some(Self::If {
            condition: Box::new(condition),
            block,
            else_block: r#else,
        })
    }
}
