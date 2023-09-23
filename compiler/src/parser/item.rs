use crate::lexer::definitions::TokenType;

use super::{
    expression::Expression,
    statement::Statement,
    types::{ArgumentName, Name, TokenItTypeExt, Type},
    Component, TokenIt, TokenItBaseExt,
};

#[derive(Debug)]
pub struct Signature {
    pub name: (String, Option<Type>),
    pub arguments: Vec<ArgumentName>,
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
        tokens.consume(|t| t.value == "const")?;

        let identifier = tokens.consume(|t| t.r#type == TokenType::Identifier)?;

        let r#type = if tokens.consume(|t| t.value == "=").is_none() {
            let r#type = tokens.get_type();
            tokens.consume(|t| t.value == "=")?;
            r#type
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
                let r#type = t.get_type()?;

                Some(ArgumentName(identifier, r#type))
            },
            Some(","),
        )?;

        let r#type = tokens.get_type();

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
