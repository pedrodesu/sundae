use crate::lexer::definitions::TokenType;

use super::{
    consume,
    expression::Expression,
    parse_block, parse_generic_list, peek,
    statement::Statement,
    types::{get_type, Name},
    Component, TokenIt,
};

#[derive(Debug)]
pub struct Signature {
    pub name: (String, Option<String>),
    pub arguments: Vec<Name>,
}

#[derive(Debug)]
pub enum Item {
    Const {
        name: Name,
        value: Expression,
    },
    Function {
        signature: Signature,
        body: Vec<Statement>,
    },
}

impl super::Component for Item {
    const PARSE_OPTIONS: &'static [fn(TokenIt) -> Option<Self>] =
        &[Self::parse_const, Self::parse_function];
}

impl Item {
    fn parse_const(tokens: TokenIt) -> Option<Self> {
        consume(tokens, |t| t.value == "const")?;

        let identifier = consume(tokens, |t| t.r#type == TokenType::Identifier)?;

        let r#type = if !peek(tokens, |t| t.value == "=") {
            let r#type = get_type(tokens);
            consume(tokens, |t| t.value == "=")?;
            r#type
        } else {
            None
        };

        let value = Expression::get(tokens)?;

        consume(tokens, |t| t.r#type == TokenType::Newline)?;

        Some(Self::Const {
            name: Name(identifier, r#type),
            value,
        })
    }

    fn parse_function(tokens: TokenIt) -> Option<Self> {
        consume(tokens, |t| t.value == "func")?;

        let identifier = consume(tokens, |t| t.r#type == TokenType::Identifier)?;

        let arguments = parse_generic_list(
            tokens,
            "(",
            ")",
            |t| {
                let identifier = consume(t, |t| t.r#type == TokenType::Identifier)?;
                let r#type = get_type(t);

                Some(Name(identifier, r#type))
            },
            Some(","),
        )?;

        let r#type = tokens
            .next_if(|t| t.r#type == TokenType::Identifier)
            .map(|t| t.value);

        let body = parse_block(tokens)?;

        Some(Self::Function {
            signature: Signature {
                name: (identifier, r#type),
                arguments,
            },
            body,
        })
    }
}
