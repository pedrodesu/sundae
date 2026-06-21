use compiler_lexer::definitions::{LiteralType, Token, TokenType};
use ecow::EcoVec;
use operator::Operator;

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
    Binary(Box<binary::Node<'s>>),
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

impl<'s, I: TokenIt> ExhaustiveGet<'s, I> for Expression<'s> {
    fn get(parser: &mut Parser<'s, I>) -> Result<Self, ParserError> {
        let mut lookahead = parser.clone();

        let base_predicate = Self::get_shallow(&mut lookahead)?;
        base_predicate(&mut lookahead)?; // Consume whichever base so we can peek ahead

        if lookahead
            .tokens
            .peek()
            .is_some_and(|t| t.r#type == TokenType::Operator)
        {
            Self::parse_binary(parser)
        } else {
            base_predicate(parser)
        }
    }
}

impl<'s> Expression<'s> {
    pub fn get_shallow<I: TokenIt>(
        parser: &mut Parser<'s, I>,
    ) -> Result<fn(&mut Parser<'s, I>) -> Result<Self, ParserError>, ParserError> {
        if parser.peek_value("if") {
            Ok(Self::parse_if)
        } else if parser
            .tokens
            .peek()
            .is_some_and(|t| matches!(t.r#type, TokenType::Operator))
        {
            Ok(Self::parse_unary)
        } else if parser
            .tokens
            .peek()
            .is_some_and(|t| matches!(t.r#type, TokenType::Literal(_)))
        {
            Ok(Self::parse_literal)
        } else if parser.peek_value("[") {
            Ok(Self::parse_array)
        } else {
            {
                let mut parser = parser.clone();

                if parser.consume_value("(").is_some() && parser.next_expression().is_ok() {
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
    pub fn parse_literal<I: TokenIt>(parser: &mut Parser<'s, I>) -> Result<Self, ParserError> {
        let t @ Token {
            r#type: TokenType::Literal(lit_type),
            ..
        } = parser
            .consume(|t| {
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

    pub fn parse_path<I: TokenIt>(parser: &mut Parser<'s, I>) -> Result<Self, ParserError> {
        let mut path = EcoVec::new();

        while path.is_empty() || parser.consume_value(".").is_some() {
            let segment = parser
                .consume(|t| t.r#type == TokenType::Identifier)
                .ok_or(ParserError::ExpectedTokenType {
                    span: parser.current_span(),
                    r#type: "Identifier",
                })?
                .value(parser.source);
            path.push(segment);
        }

        Ok(Self::Path(path))
    }

    #[inline]
    pub fn parse_binary<I: TokenIt>(parser: &mut Parser<'s, I>) -> Result<Self, ParserError> {
        // TODO RPN should prolly be bettered.
        let node = binary::Node::parse(parser)?;

        Ok(Self::Binary(Box::new(node)))
    }

    pub fn parse_call<I: TokenIt>(parser: &mut Parser<'s, I>) -> Result<Self, ParserError> {
        let Self::Path(path) = Self::parse_path(parser)? else {
            unreachable!()
        };

        let args = parser.consume_list(("(", ")"), Expression::get, Some(","))?;

        Ok(Self::Call { path, args })
    }

    pub fn parse_if<I: TokenIt>(parser: &mut Parser<'s, I>) -> Result<Self, ParserError> {
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

    pub fn parse_unary<I: TokenIt>(parser: &mut Parser<'s, I>) -> Result<Self, ParserError> {
        let token = parser
            .consume(|t| {
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
        let operator = Operator::from_bytes(token.value(parser.source)).unwrap();

        if !matches!(operator, Operator::Minus | Operator::Star) {
            return Err(ParserError::IllegalUnary {
                span: token.span,
                operator,
            });
        };

        let e = parser.next_expression()?;

        Ok(Self::Unary(operator, Box::new(e)))
    }

    pub fn parse_parenthesis<I: TokenIt>(parser: &mut Parser<'s, I>) -> Result<Self, ParserError> {
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
    pub fn parse_tuple<I: TokenIt>(parser: &mut Parser<'s, I>) -> Result<Self, ParserError> {
        Ok(Self::Tuple(parser.consume_list(
            ("(", ")"),
            Expression::get,
            Some(","),
        )?))
    }

    #[inline]
    pub fn parse_array<I: TokenIt>(parser: &mut Parser<'s, I>) -> Result<Self, ParserError> {
        Ok(Self::Array(parser.consume_list(
            ("[", "]"),
            Expression::get,
            Some(","),
        )?))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{Node, Operator, tests::parser, *};

    // Expression::Literal and Expression::Binary are mere simple wrappers for already tested features, so we don't test them here

    #[test]
    fn path_passes() {
        assert_eq!(
            Expression::parse_path(&mut parser("a.path.to")),
            Ok(Expression::Path([b"a" as &[_], b"path", b"to"].into()))
        );
    }

    #[test]
    fn call_passes() {
        assert_eq!(
            Expression::parse_call(&mut parser("call_me(     )")),
            Ok(Expression::Call {
                path: [b"call_me" as &[_]].into(),
                args: [].into()
            })
        );

        assert_eq!(
            Expression::parse_call(&mut parser("call  .me()")),
            Ok(Expression::Call {
                path: [b"call" as &[_], b"me"].into(),
                args: [].into()
            })
        );

        assert_eq!(
            Expression::parse_call(&mut parser("fn    (2)")),
            Ok(Expression::Call {
                path: [b"fn" as &[_]].into(),
                args: [Expression::Literal {
                    value: b"2",
                    r#type: LiteralType::Int
                }]
                .into()
            })
        );

        assert_eq!(
            Expression::parse_call(&mut parser("fn. path(\n\n\n420,`j`\n\n ,\n6\n)")),
            Ok(Expression::Call {
                path: [b"fn" as &[_], b"path"].into(),
                args: [
                    Expression::Literal {
                        value: b"420",
                        r#type: LiteralType::Int
                    },
                    Expression::Literal {
                        value: b"`j`",
                        r#type: LiteralType::Rune
                    },
                    Expression::Literal {
                        value: b"6",
                        r#type: LiteralType::Int
                    }
                ]
                .into()
            })
        );

        // TODO better this, make sure we have good errors
        // also this probably panics atm lol gotta make this good
        assert!(Expression::parse_call(&mut parser("fn.()")).is_err());

        assert!(Expression::parse_call(&mut parser("fn(42, )")).is_err());

        assert!(Expression::parse_call(&mut parser("fn(, 42)")).is_err());
    }

    #[test]
    fn if_passes() {
        assert_eq!(
            Expression::parse_if(&mut parser("if 1 {}")),
            Ok(Expression::If {
                condition: Box::new(Expression::Literal {
                    value: b"1",
                    r#type: LiteralType::Int
                }),
                block: vec![].into(),
                else_block: None
            })
        );

        assert_eq!(
            Expression::parse_if(&mut parser("if 1 {} else {}")),
            Ok(Expression::If {
                condition: Box::new(Expression::Literal {
                    value: b"1",
                    r#type: LiteralType::Int
                }),
                block: vec![].into(),
                else_block: Some(vec![].into())
            })
        );

        assert_eq!(
            Expression::parse_if(&mut parser("if 2 + 2 {\ncall()\n}")),
            Ok(Expression::If {
                condition: Box::new(Expression::Binary(Box::new(Node::Compound(Box::new((
                    Node::Scalar(Expression::Literal {
                        value: b"2",
                        r#type: LiteralType::Int
                    }),
                    Operator::Plus,
                    Node::Scalar(Expression::Literal {
                        value: b"2",
                        r#type: LiteralType::Int
                    })
                )))))),
                block: vec![Statement::Expression(Expression::Call {
                    path: vec![b"call" as &[_]].into(),
                    args: vec![].into()
                })]
                .into(),
                else_block: None
            })
        );

        assert_eq!(
            Expression::parse_if(&mut parser("if 1 {call()}")),
            Ok(Expression::If {
                condition: Box::new(Expression::Literal {
                    value: b"1",
                    r#type: LiteralType::Int
                }),
                block: vec![Statement::Expression(Expression::Call {
                    path: vec![b"call" as &[_]].into(),
                    args: vec![].into()
                })]
                .into(),
                else_block: None
            })
        );

        assert_eq!(
            Expression::parse_if(&mut parser("if 1{ call() }else { other_call()}")),
            Ok(Expression::If {
                condition: Box::new(Expression::Literal {
                    value: b"1",
                    r#type: LiteralType::Int
                }),
                block: vec![Statement::Expression(Expression::Call {
                    path: vec![b"call" as &[_]].into(),
                    args: vec![].into()
                })]
                .into(),
                else_block: Some(
                    vec![Statement::Expression(Expression::Call {
                        path: vec![b"other_call" as &[_]].into(),
                        args: vec![].into()
                    })]
                    .into()
                )
            })
        );

        assert_eq!(
            Expression::parse_if(&mut parser(
                // TODO \n after 2 + 2 fails. try out the suggestion of always skipping newlines when getting next
                "if\n\n2 + 2{\n   \n  call  ()\n}\n\t  \nelse\n  \n\n{\n\n42\n\n\n}\n\n" // "if\n\n2 + 2\n{\n   \n  call  ()\n}\n\t  \nelse\n  \n\n{\n\n42\n\n\n}\n\n"
            )),
            Ok(Expression::If {
                condition: Box::new(Expression::Binary(Box::new(Node::Compound(Box::new((
                    Node::Scalar(Expression::Literal {
                        value: b"2",
                        r#type: LiteralType::Int
                    }),
                    Operator::Plus,
                    Node::Scalar(Expression::Literal {
                        value: b"2",
                        r#type: LiteralType::Int
                    }),
                )))))),
                block: vec![Statement::Expression(Expression::Call {
                    path: vec![b"call" as &[_]].into(),
                    args: vec![].into()
                })]
                .into(),
                else_block: Some(
                    vec![Statement::Expression(Expression::Literal {
                        value: b"42",
                        r#type: LiteralType::Int
                    })]
                    .into()
                )
            })
        );
    }

    #[test]
    fn unary_passes() {
        assert_eq!(
            Expression::parse_unary(&mut parser("-2")),
            Ok(Expression::Unary(
                Operator::Minus,
                Box::new(Expression::Literal {
                    value: b"2",
                    r#type: LiteralType::Int
                })
            ))
        );

        assert_eq!(
            Expression::parse_unary(&mut parser("-(2 - 4)")),
            Ok(Expression::Unary(
                Operator::Minus,
                Box::new(Expression::Parenthesis(Box::new(Expression::Binary(
                    Box::new(Node::Compound(Box::new((
                        Node::Scalar(Expression::Literal {
                            value: b"2",
                            r#type: LiteralType::Int
                        }),
                        Operator::Minus,
                        Node::Scalar(Expression::Literal {
                            value: b"4",
                            r#type: LiteralType::Int
                        }),
                    ))))
                ))))
            ))
        );

        assert_eq!(
            Expression::parse_unary(&mut parser("*v")),
            Ok(Expression::Unary(
                Operator::Star,
                Box::new(Expression::Path([b"v" as &[_]].into()))
            ))
        );

        assert_eq!(
            Expression::parse_unary(&mut parser("+2")),
            Err(ParserError::IllegalUnary {
                span: (0..1).into(),
                operator: Operator::Plus,
            })
        );
    }

    // TODO statement always asks for a newline, let it also work with the scope end
    // TODO impl remaining if and unary testing
    // TODO impl parenthesis, tuple and array testing
}
