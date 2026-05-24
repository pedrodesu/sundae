use compiler_lexer::definitions::{LiteralType, Token, TokenType};
use ecow::{EcoString, EcoVec};
use operator::{Operator, to_operator};

use crate::{Parser, ParserError, TokenIt, iterator::ExhaustiveGet, statement::Statement};

pub mod binary;
pub mod operator;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression<'s> {
    Literal {
        value: &'s [u8],
        r#type: LiteralType,
    },
    Path(EcoVec<&'s [u8]>),
    Binary(Box<binary::Node>),
    Unary(Operator, Box<Expression<'s>>),
    Call {
        path: EcoVec<&'s [u8]>,
        args: EcoVec<Expression<'s>>,
    },
    If {
        condition: Box<Expression<'s>>,
        block: EcoVec<Statement<'s>>,
        else_block: Option<EcoVec<Statement<'s>>>,
    },
    Parenthesis(Box<Expression<'s>>),
    Tuple(EcoVec<Expression<'s>>),
    Array(EcoVec<Expression<'s>>),
    // TODO Block
}

impl<I: TokenIt> ExhaustiveGet<I> for Expression<'_> {
    fn find_predicate<'s>(
        parser: &mut Parser<'s, I>,
    ) -> Result<fn(&mut Parser<'s, I>) -> Result<Self, ParserError>, ParserError> {
        let base_predicate = Self::shallow_find_predicate(&mut parser.clone())?;

        base_predicate(parser)?; // Consume whichever base so we can peek ahead

        if parser
            .0
            .peek()
            .is_some_and(|t| t.r#type == TokenType::Operator)
        {
            Ok(Self::parse_binary)
        } else {
            Ok(base_predicate)
        }
    }
}

impl Expression<'_> {
    pub fn shallow_find_predicate<I: TokenIt>(
        parser: &mut Parser<I>,
    ) -> Result<fn(&mut Parser<'_, I>) -> Result<Self, ParserError>, ParserError> {
        if parser.peek_value("if") {
            Ok(Self::parse_if)
        } else if parser
            .0
            .peek()
            .is_some_and(|t| t.r#type == TokenType::Operator)
        {
            Ok(Self::parse_unary)
        } else if parser
            .0
            .peek()
            .is_some_and(|t| matches!(t.r#type, TokenType::Literal(_)))
        {
            Ok(Self::parse_literal)
        } else if parser.peek_value("[") {
            Ok(Self::parse_array)
        } else {
            {
                let mut parser = parser.clone();

                if parser.consume_value("(").is_some() && parser.get_expression().is_ok() {
                    // TODO this won't work properly with a leading colon, as probably other things won't either. make a decision on this
                    if parser.peek_value(")") {
                        return Ok(Self::parse_parenthesis);
                    } else {
                        return Ok(Self::parse_tuple);
                    }
                }
            }

            {
                let mut parser = parser.clone();

                if Self::parse_path(&mut parser).is_ok() {
                    if parser.peek_value("(") {
                        return Ok(Self::parse_call);
                    } else {
                        return Ok(Self::parse_path);
                    }
                }
            }

            Err(ParserError::ExpectedASTStructure {
                span: parser.current_span(),
                name: "Expression",
            })
        }
    }

    #[inline]
    pub fn parse_literal<I: TokenIt>(parser: &mut Parser<I>) -> Result<Self, ParserError> {
        let t @ Token {
            r#type: TokenType::Literal(lit_type),
            ..
        } = parser
            .next(|t| {
                matches!(
                    t,
                    Token {
                        r#type: TokenType::Literal(_),
                        ..
                    }
                )
            })
            .ok_or(ParserError::ExpectedTokenType {
                span: parser.current_span(),
                r#type: "Literal",
            })?
        else {
            unreachable!()
        };

        Ok(Self::Literal {
            value: parser.token_value(&t),
            r#type: lit_type,
        })
    }

    pub fn parse_path<I: TokenIt>(parser: &mut Parser<I>) -> Result<Self, ParserError> {
        let mut path = EcoVec::new();

        while path.is_empty() || parser.consume_value(".").is_some() {
            let segment = parser
                .next(|t| t.r#type == TokenType::Identifier)
                .ok_or(ParserError::ExpectedTokenType {
                    span: parser.current_span(),
                    r#type: "Identifier",
                })?
                .value;
            path.push(segment);
        }

        Ok(Self::Path(path))
    }

    #[inline]
    pub fn parse_binary<I: TokenIt>(parser: &mut Parser<I>) -> Result<Self, ParserError> {
        // TODO RPN should prolly be bettered.
        let node = binary::Node::parse(parser)?;

        Ok(Self::Binary(Box::new(node)))
    }

    pub fn parse_call<I: TokenIt>(parser: &mut Parser<I>) -> Result<Self, ParserError> {
        let Self::Path(path) = Self::parse_path(parser)? else {
            unreachable!()
        };

        let args = parser.consume_list(("(", ")"), Expression::get, Some(","))?;

        Ok(Self::Call { path, args })
    }

    pub fn parse_if<I: TokenIt>(parser: &mut Parser<I>) -> Result<Self, ParserError> {
        parser
            .consume_value("if")
            .ok_or(ParserError::ExpectedTokenValue {
                span: parser.current_span(),
                value: "if".into(),
            })?;

        // TODO ignore_newlines might not be necessary? if when we get next we always skip newline. is this viable? try and test.
        parser.ignore_newlines();

        let condition = parser.next_expression()?;

        parser.ignore_newlines();

        let block = parser.consume_block()?;

        parser.ignore_newlines();

        let r#else = if parser.consume_value("else").is_some() {
            parser.ignore_newlines();

            Some(parser.consume_block()?)
        } else {
            None
        };

        Ok(Self::If {
            condition: Box::new(condition),
            block,
            else_block: r#else,
        })
    }

    pub fn parse_unary<I: TokenIt>(parser: &mut Parser<I>) -> Result<Self, ParserError> {
        let operator = parser
            .next(|t| {
                matches!(
                    t,
                    Token {
                        r#type: TokenType::Operator,
                        ..
                    }
                )
            })
            .ok_or(ParserError::ExpectedTokenType {
                span: parser.current_span(),
                r#type: "Operator",
            })?;
        let operator @ (Operator::Minus | Operator::Star) = to_operator(&operator, parser.source)
        else {
            return Err(ParserError::IllegalUnary {
                span: parser.current_span(),
                token: operator,
            });
        };

        let e = parser.next_expression()?;

        Ok(Self::Unary(operator, Box::new(e)))
    }

    pub fn parse_parenthesis<I: TokenIt>(parser: &mut Parser<I>) -> Result<Self, ParserError> {
        parser
            .consume_value("(")
            .ok_or(ParserError::ExpectedTokenValue {
                span: parser.current_span(),
                value: "(".into(),
            })?;

        let e = parser.next_expression()?;

        parser
            .consume_value(")")
            .ok_or(ParserError::ExpectedTokenValue {
                span: parser.current_span(),
                value: ")".into(),
            })?;

        Ok(Self::Parenthesis(Box::new(e)))
    }

    #[inline]
    pub fn parse_tuple<I: TokenIt>(parser: &mut Parser<I>) -> Result<Self, ParserError> {
        Ok(Self::Tuple(parser.consume_list(
            ("(", ")"),
            Expression::get,
            Some(","),
        )?))
    }

    #[inline]
    pub fn parse_array<I: TokenIt>(parser: &mut Parser<I>) -> Result<Self, ParserError> {
        Ok(Self::Array(parser.consume_list(
            ("[", "]"),
            Expression::get,
            Some(","),
        )?))
    }
}

#[cfg(test)]
mod tests {
    use compiler_lexer::definitions::Span;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::Node;

    // Expression::Literal and Expression::Binary are mere simple wrappers for already tested features, so we don't test them here

    #[test]
    fn path_passes() {
        assert_eq!(
            Expression::parse_path(&mut TokenIt(
                compiler_lexer::tokenize("a.path.to").flatten().peekable()
            )),
            Ok(Expression::Path(
                vec!["a".into(), "path".into(), "to".into()].into()
            ))
        );
    }

    #[test]
    fn call_passes() {
        assert_eq!(
            Expression::parse_call(&mut TokenIt(
                compiler_lexer::tokenize("call_me(     )")
                    .flatten()
                    .peekable()
            )),
            Ok(Expression::Call {
                path: vec!["call_me".into()].into(),
                args: vec![].into()
            })
        );

        assert_eq!(
            Expression::parse_call(&mut TokenIt(
                compiler_lexer::tokenize("call  .me()").flatten().peekable()
            )),
            Ok(Expression::Call {
                path: vec!["call".into(), "me".into()].into(),
                args: vec![].into()
            })
        );

        assert_eq!(
            Expression::parse_call(&mut TokenIt(
                compiler_lexer::tokenize("fn    (2)").flatten().peekable()
            )),
            Ok(Expression::Call {
                path: vec!["fn".into()].into(),
                args: vec![Expression::Literal {
                    value: "2".into(),
                    r#type: LiteralType::Int
                }]
                .into()
            })
        );

        assert_eq!(
            Expression::parse_call(&mut TokenIt(
                compiler_lexer::tokenize("fn. path(\n\n\n420,`j`\n\n ,\n6\n)")
                    .flatten()
                    .peekable()
            )),
            Ok(Expression::Call {
                path: vec!["fn".into(), "path".into()].into(),
                args: vec![
                    Expression::Literal {
                        value: "420".into(),
                        r#type: LiteralType::Int
                    },
                    Expression::Literal {
                        value: "`j`".into(),
                        r#type: LiteralType::Rune
                    },
                    Expression::Literal {
                        value: "6".into(),
                        r#type: LiteralType::Int
                    }
                ]
                .into()
            })
        );

        // TODO better this, make sure we have good errors
        // also this probably panics atm lol gotta make this good
        assert!(
            Expression::parse_call(&mut TokenIt(
                compiler_lexer::tokenize("fn.()").flatten().peekable()
            ))
            .is_err()
        );

        assert!(
            Expression::parse_call(&mut TokenIt(
                compiler_lexer::tokenize("fn(42, )").flatten().peekable()
            ))
            .is_err()
        );

        assert!(
            Expression::parse_call(&mut TokenIt(
                compiler_lexer::tokenize("fn(, 42)").flatten().peekable()
            ))
            .is_err()
        );
    }

    // #[test]
    // fn if_passes()
    // {
    //     // assert_eq!(
    //     //     Expression::parse_if(&mut TokenIt(
    //     //         compiler_lexer::tokenize("if 1 {}").flatten().peekable()
    //     //     )),
    //     //     Ok(Expression::If {
    //     //         condition: Box::new(Expression::Literal {
    //     //             value: "1".into(),
    //     //             r#type: LiteralType::Int
    //     //         }),
    //     //         block: vec![].into(),
    //     //         else_block: None
    //     //     })
    //     // );

    //     // assert_eq!(
    //     //     Expression::parse_if(&mut TokenIt(
    //     //         compiler_lexer::tokenize("if 1 {} else {}")
    //     //             .flatten()
    //     //             .peekable()
    //     //     )),
    //     //     Ok(Expression::If {
    //     //         condition: Box::new(Expression::Literal {
    //     //             value: "1".into(),
    //     //             r#type: LiteralType::Int
    //     //         }),
    //     //         block: vec![].into(),
    //     //         else_block: Some(vec![].into())
    //     //     })
    //     // );

    //     // assert_eq!(
    //     //     Expression::parse_if(&mut TokenIt(
    //     //         compiler_lexer::tokenize("if 2 + 2 {\ncall()\n}")
    //     //             .flatten()
    //     //             .peekable()
    //     //     )),
    //     //     Ok(Expression::If {
    //     //         condition: Box::new(Expression::Binary(Box::new(Node::Compound(Box::new((
    //     //             Node::Scalar(Expression::Literal {
    //     //                 value: "2".into(),
    //     //                 r#type: LiteralType::Int
    //     //             }),
    //     //             Operator::Plus,
    //     //             Node::Scalar(Expression::Literal {
    //     //                 value: "2".into(),
    //     //                 r#type: LiteralType::Int
    //     //             })
    //     //         )))))),
    //     //         block: vec![Statement::Expression(Expression::Call {
    //     //             path: vec!["call".into()].into(),
    //     //             args: vec![].into()
    //     //         })]
    //     //         .into(),
    //     //         else_block: None
    //     //     })
    //     // );

    //     // assert_eq!(
    //     //     Expression::parse_if(&mut TokenIt(
    //     //         compiler_lexer::tokenize("if 1 {call()}")
    //     //             .flatten()
    //     //             .peekable()
    //     //     )),
    //     //     Ok(Expression::If {
    //     //         condition: Box::new(Expression::Literal {
    //     //             value: "1".into(),
    //     //             r#type: LiteralType::Int
    //     //         }),
    //     //         block: vec![Statement::Expression(Expression::Call {
    //     //             path: vec!["call".into()].into(),
    //     //             args: vec![].into()
    //     //         })]
    //     //         .into(),
    //     //         else_block: None
    //     //     })
    //     // );

    //     // assert_eq!(
    //     //     Expression::parse_if(&mut TokenIt(
    //     //         compiler_lexer::tokenize("if 1{ call() }else { other_call()}")
    //     //             .flatten()
    //     //             .peekable()
    //     //     )),
    //     //     Ok(Expression::If {
    //     //         condition: Box::new(Expression::Literal {
    //     //             value: "1".into(),
    //     //             r#type: LiteralType::Int
    //     //         }),
    //     //         block: vec![Statement::Expression(Expression::Call {
    //     //             path: vec!["call".into()].into(),
    //     //             args: vec![].into()
    //     //         })]
    //     //         .into(),
    //     //         else_block: Some(
    //     //             vec![Statement::Expression(Expression::Call {
    //     //                 path: vec!["other_call".into()].into(),
    //     //                 args: vec![].into()
    //     //             })]
    //     //             .into()
    //     //         )
    //     //     })
    //     // );

    //     assert_eq!(
    //         Expression::parse_if(&mut TokenIt(
    //             compiler_lexer::tokenize(
    //                 // TODO \n after 2 + 2 fails. try out the suggestion of always skipping newlines when getting next
    //                 "if\n\n2 + 2\n{\n   \n  call  ()\n}\n\t  \nelse\n  \n\n{\n\n42\n\n\n}\n\n" // "if\n\n2 + 2{\n   \n  call  ()\n}\n\t  \nelse\n  \n\n{\n\n42\n\n\n}\n\n"
    //             )
    //             .flatten()
    //             .peekable()
    //         )),
    //         Ok(Expression::If {
    //             condition: Box::new(Expression::Binary(Box::new(Node::Compound(Box::new((
    //                 Node::Scalar(Expression::Literal {
    //                     value: "2".into(),
    //                     r#type: LiteralType::Int
    //                 }),
    //                 Operator::Plus,
    //                 Node::Scalar(Expression::Literal {
    //                     value: "2".into(),
    //                     r#type: LiteralType::Int
    //                 }),
    //             )))))),
    //             block: vec![Statement::Expression(Expression::Call {
    //                 path: vec!["call".into()].into(),
    //                 args: vec![].into()
    //             })]
    //             .into(),
    //             else_block: Some(
    //                 vec![Statement::Expression(Expression::Literal {
    //                     value: "42".into(),
    //                     r#type: LiteralType::Int
    //                 })]
    //                 .into()
    //             )
    //         })
    //     );
    // }

    #[test]
    fn unary_passes() {
        assert_eq!(
            Expression::parse_unary(&mut TokenIt(
                compiler_lexer::tokenize("-2").flatten().peekable()
            )),
            Ok(Expression::Unary(
                Operator::Minus,
                Box::new(Expression::Literal {
                    value: "2".into(),
                    r#type: LiteralType::Int
                })
            ))
        );

        assert_eq!(
            Expression::parse_unary(&mut TokenIt(
                compiler_lexer::tokenize("-(2 - 4)").flatten().peekable()
            )),
            Ok(Expression::Unary(
                Operator::Minus,
                Box::new(Expression::Parenthesis(Box::new(Expression::Binary(
                    Box::new(Node::Compound(Box::new((
                        Node::Scalar(Expression::Literal {
                            value: "2".into(),
                            r#type: LiteralType::Int
                        }),
                        Operator::Minus,
                        Node::Scalar(Expression::Literal {
                            value: "4".into(),
                            r#type: LiteralType::Int
                        }),
                    ))))
                ))))
            ))
        );

        assert_eq!(
            Expression::parse_unary(&mut TokenIt(
                compiler_lexer::tokenize("*v").flatten().peekable()
            )),
            Ok(Expression::Unary(
                Operator::Star,
                Box::new(Expression::Path(vec!["v".into()].into()))
            ))
        );

        assert_eq!(
            Expression::parse_unary(&mut TokenIt(
                compiler_lexer::tokenize("+2").flatten().peekable()
            )),
            Err(ParserError::IllegalUnary {
                token: Token {
                    r#type: TokenType::Operator,
                    value: "+".into(),
                    span: Span {
                        from: (0, 0),
                        to: (0, 0)
                    }
                }
            })
        );
    }

    // TODO statement always asks for a newline, let it also work with the scope end
    // TODO impl remaining if and unary testing
    // TODO impl parenthesis, tuple and array testing
}
