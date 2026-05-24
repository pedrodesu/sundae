use compiler_lexer::definitions::{Token, TokenType};
use ecow::EcoVec;
use itertools::Itertools;

use crate::{
    ArgumentName, Name, Parser, ParserError, TokenIt, Type, expression::Expression,
    iterator::ExhaustiveGet, statement::Statement,
};

#[derive(Debug, PartialEq)]
pub struct FunctionSignature<'s> {
    pub name: (&'s [u8], Option<Type<'s>>),
    pub arguments: EcoVec<ArgumentName<'s>>,
}

#[derive(Debug, PartialEq)]
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

impl<I: TokenIt> ExhaustiveGet<I> for Item<'_> {
    fn find_predicate<'s>(
        parser: &mut Parser<'s, I>,
    ) -> Result<fn(&mut Parser<'s, I>) -> Result<Self, ParserError>, ParserError> {
        if parser.peek_value("const") {
            Ok(Self::parse_const)
        } else if parser.peek_value("func") {
            Ok(Self::parse_function)
        } else {
            Err(ParserError::ExpectedASTStructure {
                span: parser.current_span(),
                name: "Item",
            })
        }
    }
}

impl Item<'_> {
    pub fn parse_const<I: TokenIt>(parser: &mut Parser<I>) -> Result<Self, ParserError> {
        parser
            .consume_value("const")
            .ok_or(ParserError::ExpectedTokenValue {
                span: parser.current_span(),
                value: "const".into(),
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
            let source = parser.source;
            let r#type = parser
                .tokens
                .peeking_take_while(|t| !t.is_value(source, "="))
                .map(|t| t.value(source))
                .collect::<Vec<_>>();

            if r#type.is_empty() {
                None
            } else {
                Some(Type(r#type))
            }
        };

        let value = parser.next_expression()?;

        parser
            .consume(|t| t.r#type == TokenType::Newline)
            .ok_or(ParserError::ExpectedNewline {
                span: parser.current_span(),
            })?;

        Ok(Self::Const {
            name: Name(identifier, r#type),
            value,
        })
    }

    pub fn parse_function<I: TokenIt>(parser: &mut Parser<I>) -> Result<Self, ParserError> {
        parser
            .consume_value("func")
            .ok_or(ParserError::ExpectedTokenValue {
                span: parser.current_span(),
                value: "func".into(),
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

                let source = parser.source;
                let r#type = Type(
                    parser
                        .tokens
                        .peeking_take_while(|t| {
                            !t.is_value(source, ",") && !t.is_value(source, ")")
                        })
                        .map(|t| t.value(source))
                        .collect(),
                );

                Ok(ArgumentName(identifier, r#type))
            },
            Some(","),
        )?;

        let r#type = {
            let source = parser.source;
            let r#type = parser
                .tokens
                .peeking_take_while(|t| !t.is_value(source, "{"))
                .map(|t| t.value(source))
                .collect::<Vec<_>>();

            if r#type.is_empty() {
                None
            } else {
                Some(Type(r#type))
            }
        };

        let body = parser.consume_block()?;

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
