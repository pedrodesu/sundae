use itertools::Itertools;

use crate::{
    components::{
        parser_types::{ArgumentName, Name, Type},
        Expression,
    },
    lexer::definitions::TokenType,
    parser::{Component, TokenIt, TokenItBaseExt},
};

use super::{Item, Signature};

impl Component for Item {
    const PARSE_OPTIONS: &'static [fn(TokenIt) -> Option<Self>] =
        &[Self::parse_const, Self::parse_function];
}

impl Item {
    fn parse_const(tokens: TokenIt) -> Option<Self> {
        tokens.consume(|t| t.value == "const")?;

        let identifier = tokens.consume(|t| t.r#type == TokenType::Identifier)?;

        let r#type = if tokens.consume(|t| t.value == "=").is_none() {
            Some(Type(
                tokens
                    .take_while(|t| t.value != "=")
                    .map(|t| t.value)
                    .collect(),
            ))
        } else {
            None
        };

        let value = Expression::get(tokens)?;

        tokens.consume(|t| t.r#type == TokenType::Newline)?;

        Some(Self::Const {
            name: Name(identifier, r#type),
            value,
        })
    }

    fn parse_function(tokens: TokenIt) -> Option<Self> {
        tokens.consume(|t| t.value == "func")?;

        let identifier = tokens.consume(|t| t.r#type == TokenType::Identifier)?;

        let arguments = tokens.parse_generic_list(
            "(",
            ")",
            |t| {
                let identifier = t.consume(|t| t.r#type == TokenType::Identifier)?;

                let r#type = Type(
                    t.peeking_take_while(|t| t.value != "," && t.value != ")")
                        .map(|t| t.value)
                        .collect(),
                );

                Some(ArgumentName(identifier, r#type))
            },
            Some(","),
        )?;

        let r#type = if tokens.peek()?.value != "{" {
            Some(Type(
                tokens
                    .peeking_take_while(|t| t.value != "{")
                    .map(|t| t.value)
                    .collect(),
            ))
        } else {
            None
        };

        let body = tokens.parse_block()?;

        Some(Self::Function {
            signature: Signature {
                name: (identifier, r#type),
                arguments,
            },
            body,
        })
    }
}
