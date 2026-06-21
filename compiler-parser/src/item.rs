use compiler_lexer::definitions::{Token, TokenType};
use ecow::EcoVec;
use itertools::Itertools;

use crate::{
    ArgumentName, Name, Parser, ParserError, TokenIt, Type, expression::Expression,
    iterator::ExhaustiveGet, statement::Statement,
};

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionSignature<'s> {
    pub name: (&'s [u8], Option<Type<'s>>),
    pub arguments: EcoVec<ArgumentName<'s>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Item<'s> {
    Const {
        name: Name<'s>,
        value: Expression<'s>,
    },
    Function {
        signature: FunctionSignature<'s>,
        body: EcoVec<Statement<'s>>,
    },
}

impl<'s, I: TokenIt> ExhaustiveGet<'s, I> for Item<'s> {
    fn get(parser: &mut Parser<'s, I>) -> Result<Self, ParserError> {
        if parser.peek_value("const") {
            Self::parse_const(parser)
        } else if parser.peek_value("func") {
            Self::parse_function(parser)
        } else {
            Err(ParserError::ExpectedASTStructure {
                span: parser.current_span(),
                name: "Item",
            })
        }
    }
}

impl<'s> Item<'s> {
    pub fn parse_const<I: TokenIt>(parser: &mut Parser<'s, I>) -> Result<Self, ParserError> {
        parser
            .consume_value("const")
            .ok_or(ParserError::ExpectedTokenValue {
                span: parser.current_span(),
                value: "const",
            })?;

        let identifier = {
            let token = parser
                .consume(|t| t.r#type == TokenType::Identifier)
                .ok_or(ParserError::ExpectedTokenType {
                    span: parser.current_span(),
                    r#type: "Identifier",
                })?;
            parser.token_value(&token)
        };

        let r#type = {
            let r#type = parser
                .tokens
                .peeking_take_while(|t| !t.is_value(parser.source, "="))
                .map(|t| t.value(parser.source))
                .collect::<EcoVec<_>>();

            if r#type.is_empty() {
                None
            } else {
                Some(Type(r#type))
            }
        };

        parser
            .consume_value("=")
            .ok_or(ParserError::ExpectedTokenValue {
                span: parser.current_span(),
                value: "=",
            })?;

        let value = parser.next_expression()?;

        Ok(Self::Const {
            name: Name(identifier, r#type),
            value,
        })
    }

    pub fn parse_function<I: TokenIt>(parser: &mut Parser<'s, I>) -> Result<Self, ParserError> {
        parser
            .consume_value("func")
            .ok_or(ParserError::ExpectedTokenValue {
                span: parser.current_span(),
                value: "func",
            })?;

        let identifier = {
            let token = parser
                .consume(|t| t.r#type == TokenType::Identifier)
                .ok_or(ParserError::ExpectedTokenType {
                    span: parser.current_span(),
                    r#type: "Identifier",
                })?;
            parser.token_value(&token)
        };

        let arguments = parser.consume_list(
            ("(", ")"),
            |parser| {
                let identifier = {
                    let token = parser
                        .consume(|t| {
                            matches!(
                                t,
                                Token {
                                    r#type: TokenType::Identifier,
                                    ..
                                }
                            )
                        })
                        .ok_or(ParserError::ExpectedTokenType {
                            span: parser.current_span(),
                            r#type: "Identifier",
                        })?;
                    parser.token_value(&token)
                };

                let r#type = Type(
                    parser
                        .tokens
                        .peeking_take_while(|t| {
                            !t.is_value(parser.source, ",") && !t.is_value(parser.source, ")")
                        })
                        .map(|t| t.value(parser.source))
                        .collect(),
                );

                Ok(ArgumentName(identifier, r#type))
            },
            Some(","),
        )?;

        let r#type = {
            let r#type = parser
                .tokens
                .peeking_take_while(|t| !t.is_value(parser.source, "{"))
                .map(|t| t.value(parser.source))
                .collect::<EcoVec<_>>();

            if r#type.is_empty() {
                None
            } else {
                Some(Type(r#type))
            }
        };

        let body = parser.consume_block()?;

        Ok(Self::Function {
            signature: FunctionSignature {
                name: (identifier, r#type),
                arguments,
            },
            body,
        })
    }
}
