use compiler_lexer::definitions::TokenType;
use ecow::{EcoString, EcoVec};
use itertools::Itertools;

use crate::{
    ArgumentName, Name, ParserError, TokenIt, Type,
    expression::Expression,
    iterator::{ExhaustiveGet, TokenItTrait},
    statement::Statement,
};

#[derive(Debug, PartialEq)]
pub struct FunctionSignature
{
    pub name: (EcoString, Option<Type>),
    pub arguments: EcoVec<ArgumentName>,
}

#[derive(Debug, PartialEq)]
pub enum Item
{
    Const
    {
        name: Name, value: Expression
    },
    Function
    {
        signature: FunctionSignature,
        body: EcoVec<Statement>,
    },
}

impl<I: TokenItTrait> ExhaustiveGet<I> for Item
{
    fn find_predicate(tokens: &mut TokenIt<I>) -> Result<Self::ParsePredicate, ParserError>
    {
        if tokens.0.peek().is_some_and(|t| t.value == "const")
        {
            Ok(Self::parse_const)
        }
        else if tokens.0.peek().is_some_and(|t| t.value == "func")
        {
            Ok(Self::parse_function)
        }
        else
        {
            Err(ParserError::ExpectedASTStructure { name: "Item" })
        }
    }
}

impl Item
{
    pub fn parse_const<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Result<Self, ParserError>
    {
        tokens
            .next(|t| t.value == "const")
            .ok_or(ParserError::ExpectedTokenValue {
                value: "const".into(),
            })?;

        // partially shared on statement.rs. make this better
        let identifier = tokens
            .next(|t| t.r#type == TokenType::Identifier)
            .ok_or(ParserError::ExpectedTokenType {
                r#type: "Identifier",
            })?
            .value;

        let r#type = {
            let r#type = tokens
                .0
                .peeking_take_while(|t| t.value != "=")
                .map(|t| t.value)
                .collect::<Vec<_>>();

            if r#type.is_empty()
            {
                None
            }
            else
            {
                Some(Type(r#type))
            }
        };

        let value = Expression::get(tokens)?;

        tokens
            .next(|t| t.r#type == TokenType::Newline)
            .ok_or(ParserError::ExpectedNewline)?;

        Ok(Self::Const {
            name: Name(identifier, r#type),
            value,
        })
    }

    pub fn parse_function<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Result<Self, ParserError>
    {
        tokens
            .next(|t| t.value == "func")
            .ok_or(ParserError::ExpectedTokenValue {
                value: "func".into(),
            })?;

        let identifier = tokens
            .next(|t| t.r#type == TokenType::Identifier)
            .ok_or(ParserError::ExpectedTokenType {
                r#type: "Identifier",
            })?
            .value;

        let arguments = tokens.consume_generic_list(
            ("(", ")"),
            |t| {
                let identifier = t
                    .next(|t| t.r#type == TokenType::Identifier)
                    .ok_or(ParserError::ExpectedTokenType {
                        r#type: "Identifier",
                    })?
                    .value;

                let r#type = Type(
                    t.0.peeking_take_while(|t| t.value != "," && t.value != ")")
                        .map(|t| t.value)
                        .collect(),
                );

                Ok(ArgumentName(identifier, r#type))
            },
            Some(","),
        )?;

        let r#type = {
            let r#type = tokens
                .0
                .peeking_take_while(|t| t.value != "{")
                .map(|t| t.value)
                .collect::<Vec<_>>();

            if r#type.is_empty()
            {
                None
            }
            else
            {
                Some(Type(r#type))
            }
        };

        let body = tokens.consume_block()?;

        /* TODO!
        if let Some(ref r#type) = r#type
            && body
                .iter()
                .find(|&s| matches!(s, Statement::Return(_)))
                .is_none()
        {
            return Some(Err(anyhow!(
                "Function {} must return {}, returns void",
                identifier,
                r#type
            )));
        }
        */

        Ok(Self::Function {
            signature: FunctionSignature {
                name: (identifier, r#type),
                arguments,
            },
            body,
        })
    }
}

// TODO tests
