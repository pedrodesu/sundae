use compiler_lexer::definitions::{Token, TokenType};
use itertools::Itertools;

use crate::{
    Name, Parser, ParserError, TokenIt, Type, expression::Expression, iterator::ExhaustiveGet,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Statement<'s> {
    Return(Option<Expression<'s>>),
    Expression(Expression<'s>),
    // TODO add another field here. identify and refactor operator assign exprs such as +=
    Assign {
        destination: Expression<'s>,
        source: Expression<'s>,
    },
    Local {
        mutable: bool,
        name: Name<'s>,
        init: Option<Expression<'s>>,
    },
}

impl<I: TokenIt> ExhaustiveGet<I> for Statement<'_> {
    fn find_predicate<'s>(
        parser: &mut Parser<'s, I>,
    ) -> Result<fn(&mut Parser<'s, I>) -> Result<Self, ParserError>, ParserError> {
        if parser.peek_value("ret") {
            Ok(Self::parse_return)
        } else if parser.peek_value("let") {
            Ok(Self::parse_local)
        } else {
            {
                let mut parser = parser.clone();
                if parser.next_expression().is_ok() {
                    if parser.consume_value("=").is_some() {
                        return Ok(Self::parse_assign);
                    } else {
                        return Ok(Self::parse_expression);
                    }
                }
            }

            Err(ParserError::ExpectedASTStructure {
                span: parser.current_span(),
                name: "Statement",
            })
        }
    }
}

// TODO implement concrete error types for everything that may fuck up
// also make error handling more proper with line and col and what not

impl<'s> Statement<'s> {
    #[inline]
    fn assert_end<I: TokenIt>(
        parser: &mut Parser<'s, I>,
        predicate: impl FnOnce(&mut Parser<'s, I>) -> Result<Self, ParserError>,
    ) -> Result<Self, ParserError> {
        let value = predicate(parser)?;

        if let None
        | Some(Token {
            r#type: TokenType::Separator | TokenType::Newline,
            ..
        }) = parser.tokens.peek()
        {
            Ok(value)
        } else {
            Err(ParserError::ExpectedNewline {
                span: parser.current_span(),
            })
        }
    }

    #[inline]
    pub fn parse_return<I: TokenIt>(parser: &mut Parser<'s, I>) -> Result<Self, ParserError> {
        Self::assert_end(parser, |parser| {
            parser
                .consume_value("ret")
                .ok_or(ParserError::ExpectedTokenValue {
                    span: parser.current_span(),
                    value: "ret",
                })?;

            if let None
            | Some(Token {
                r#type: TokenType::Newline,
                ..
            }) = parser.tokens.peek()
            {
                Ok(Self::Return(None))
            } else {
                let e = Expression::get(parser)?;

                Ok(Self::Return(Some(e)))
            }

            // if tokens.0.peek().unwrap().r#type == TokenType::Newline {
            //     Ok(Self::Return(None))
            // } else {
            //     // Expression::find(tokens)
            //     //     .ok_or_else(|| ParserError::Unexpected { token: next })
            //     //     .flatten()
            //     //     .map(|e| Self::Return(Some(e)))
            // }
        })
    }

    #[inline]
    pub fn parse_expression<I: TokenIt>(parser: &mut Parser<'s, I>) -> Result<Self, ParserError> {
        Self::assert_end(parser, |parser| {
            Ok(Self::Expression(Expression::get(parser)?))
        })
    }

    pub fn parse_assign<I: TokenIt>(parser: &mut Parser<'s, I>) -> Result<Self, ParserError> {
        Self::assert_end(parser, |parser| {
            let destination = Expression::get(parser)?;

            parser
                .consume_value("=")
                .ok_or(ParserError::ExpectedTokenValue {
                    span: parser.current_span(),
                    value: "=",
                })?;

            let source = Expression::get(parser)?;

            Ok(Self::Assign {
                destination,
                source,
            })
        })
    }

    pub fn parse_local<I: TokenIt>(parser: &mut Parser<'s, I>) -> Result<Self, ParserError> {
        Self::assert_end(parser, |parser| {
            parser
                .consume_value("let")
                .ok_or(ParserError::ExpectedTokenValue {
                    span: parser.current_span(),
                    value: "let",
                })?;

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

            let mutable = parser.consume_value("mut").is_some();

            // shouldn't mut always only be intrinsic to the type?
            // No. a variable can be mutable. a type does not have this qualification. a pointer, however, may or may not be mutable.

            let r#type = {
                let source = parser.source;
                let r#type = parser
                    .tokens
                    .peeking_take_while(|t| !t.is_value(source, "=") && t.r#type != TokenType::Newline)
                    .map(|t| t.value(source))
                    .collect::<Vec<_>>();

                if r#type.is_empty() {
                    None
                } else {
                    Some(Type(r#type))
                }
            };

            let init = if parser.consume_value("=").is_some() {
                Some(Expression::get(parser)?)
            } else {
                None
            };

            Ok(Self::Local {
                name: Name(identifier, r#type),
                mutable,
                init,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use compiler_lexer::{LexerEvent, definitions::LiteralType};
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::Operator;

    fn parser(source: &str) -> Parser<'_, impl TokenIt> {
        Parser::new(
            source,
            compiler_lexer::tokenize(source).map(|event| match event {
                LexerEvent::Token(token) => token,
                LexerEvent::Error(err) => panic!("lexer error: {err:?}"),
            }),
        )
    }

    // Statement::parse_expression is just a struct wrapper for an already tested function, so we don't test it here

    #[test]
    fn return_passes() {
        assert_eq!(
            Statement::parse_return(&mut parser("ret \n")),
            Ok(Statement::Return(None))
        );

        assert_eq!(
            Statement::parse_return(&mut parser("ret 42")),
            Ok(Statement::Return(Some(Expression::Literal {
                value: "42".as_bytes(),
                r#type: LiteralType::Int
            })))
        );

        assert_eq!(
            Statement::parse_return(&mut parser("ret ret\n\n")),
            Err(ParserError::ExpectedASTStructure {
                name: "Expression",
                span: (3..5).into()
            })
        );
    }

    #[test]
    fn assign_passes() {
        assert_eq!(
            Statement::parse_assign(&mut parser("a = 2")),
            Ok(Statement::Assign {
                destination: Expression::Path(vec!["a".as_bytes()].into()),
                source: Expression::Literal {
                    value: "2".as_bytes(),
                    r#type: LiteralType::Int
                }
            })
        );

        assert_eq!(
            Statement::parse_assign(&mut parser("*func_to_ptr() = 42")),
            Ok(Statement::Assign {
                destination: Expression::Unary(
                    Operator::Star,
                    Box::new(Expression::Call {
                        path: vec!["func_to_ptr".as_bytes()].into(),
                        args: vec![].into()
                    })
                ),
                source: Expression::Literal {
                    value: "42".as_bytes(),
                    r#type: LiteralType::Int
                }
            })
        );
    }

    #[test]
    fn local_passes() {
        assert_eq!(
            Statement::parse_local(&mut parser("let v")),
            Ok(Statement::Local {
                mutable: false,
                name: Name("v".as_bytes(), None),
                init: None,
            })
        );

        assert_eq!(
            Statement::parse_local(&mut parser("let a = 2\n")),
            Ok(Statement::Local {
                mutable: false,
                name: Name("a".as_bytes(), None),
                init: Some(Expression::Literal {
                    value: "2".as_bytes(),
                    r#type: LiteralType::Int
                })
            })
        );

        assert_eq!(
            Statement::parse_local(&mut parser("let b i32 = 4\n")),
            Ok(Statement::Local {
                mutable: false,
                name: Name("b".as_bytes(), Some(Type(vec!["i32".as_bytes()]))),
                init: Some(Expression::Literal {
                    value: "4".as_bytes(),
                    r#type: LiteralType::Int
                })
            })
        );

        assert_eq!(
            Statement::parse_local(&mut parser("let b i32\n")),
            Ok(Statement::Local {
                mutable: false,
                name: Name("b".as_bytes(), Some(Type(vec!["i32".as_bytes()]))),
                init: None
            })
        );

        assert_eq!(
            Statement::parse_local(&mut parser("let c *i32\n")),
            Ok(Statement::Local {
                mutable: false,
                name: Name(
                    "c".as_bytes(),
                    Some(Type(vec!["*".as_bytes(), "i32".as_bytes()]))
                ),
                init: None
            })
        );

        assert_eq!(
            Statement::parse_local(&mut parser("let s []i32\n")),
            Ok(Statement::Local {
                mutable: false,
                name: Name(
                    "s".as_bytes(),
                    Some(Type(vec!["[".as_bytes(), "]".as_bytes(), "i32".as_bytes()]))
                ),
                init: None
            })
        );

        // TODO finish tests
    }
}
