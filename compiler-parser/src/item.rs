use compiler_lexer::definitions::TokenType;
use itertools::Itertools;

use crate::{
    expression::Expression, statement::Statement, ArgumentName, ExhaustiveGet, Name, TokenIt, Type,
};

#[derive(Debug)]
pub struct FunctionSignature {
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
        signature: FunctionSignature,
        body: Vec<Statement>,
    },
}

impl ExhaustiveGet for Item {
    const PARSE_OPTIONS: &'static [fn(&mut TokenIt) -> Option<Self>] =
        &[Self::parse_const, Self::parse_function];
}

impl Item {
    pub fn parse_const(tokens: &mut TokenIt) -> Option<Self> {
        tokens.consume(|t| t.value == "const")?;

        let identifier = tokens.consume(|t| t.r#type == TokenType::Identifier)?;

        let r#type = if tokens.consume(|t| t.value == "=").is_none() {
            Some(Type(
                tokens
                    .0
                    .by_ref()
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

    pub fn parse_function(tokens: &mut TokenIt) -> Option<Self> {
        tokens.consume(|t| t.value == "func")?;

        let identifier = tokens.consume(|t| t.r#type == TokenType::Identifier)?;

        let arguments = tokens.parse_generic_list(
            "(",
            ")",
            |t| {
                let identifier = t.consume(|t| t.r#type == TokenType::Identifier)?;

                let r#type = Type(
                    t.0.peeking_take_while(|t| t.value != "," && t.value != ")")
                        .map(|t| t.value)
                        .collect(),
                );

                Some(ArgumentName(identifier, r#type))
            },
            Some(","),
        )?;

        let r#type = if tokens.0.peek()?.value != "{" {
            Some(Type(
                tokens
                    .0
                    .peeking_take_while(|t| t.value != "{")
                    .map(|t| t.value)
                    .collect(),
            ))
        } else {
            None
        };

        let body = tokens.parse_block()?;

        Some(Self::Function {
            signature: FunctionSignature {
                name: (identifier, r#type),
                arguments,
            },
            body,
        })
    }
}
