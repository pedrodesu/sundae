use compiler_lexer::definitions::{Token, TokenType};
use ecow::EcoVec;

use super::{Expression, operator::Operator};
use crate::{Parser, ParserError, TokenIt};

const OPERATOR_PRIORITY: &[&[Operator]] = {
    use Operator::*;

    &[&[Plus, Minus], &[Star, Div]]
};

#[inline]
fn priority(operator: Operator) -> usize {
    OPERATOR_PRIORITY
        .iter()
        .copied()
        .position(|v| v.contains(&operator))
        .map(|v| v + 1)
        // Custom operators are of the least priority, as they should be evaluated last, after every mathematical operator
        .unwrap_or_default()
}

#[derive(Clone, Debug, PartialEq)]
pub enum Node<'s> {
    Scalar(Expression<'s>),
    Compound(Box<(Node<'s>, Operator, Node<'s>)>),
}

#[derive(Debug, PartialEq, Clone)]
enum RPNItem<'s> {
    Scalar(Expression<'s>),
    Operator(Operator),
}

impl<'s> Node<'s> {
    // We do not need to concern ourselves with unary operators or parenthesis here because we already handle them as singular, regular expressions.
    // This makes our shunting yard much simpler.
    fn shunting_yard<I: TokenIt>(
        parser: &mut Parser<'s, I>,
    ) -> Result<EcoVec<RPNItem<'s>>, ParserError> {
        let mut output_queue = EcoVec::new();
        let mut operator_stack = EcoVec::new();

        let mut last_was_scalar = false;

        while let Some(p) = parser.tokens.peek()
            && (p.r#type != TokenType::Separator || p.value(parser.source) == b"(")
        {
            if last_was_scalar {
                last_was_scalar = false;

                let t = parser.consume(|t| t.r#type == TokenType::Operator).ok_or(
                    ParserError::ExpectedTokenType {
                        r#type: "Operator",
                        span: parser.current_span(),
                    },
                )?;

                while let Some(
                    t2 @ Token {
                        r#type: TokenType::Operator,
                        ..
                    },
                ) = operator_stack.last()
                {
                    let op = Operator::from_bytes(t.value(parser.source)).unwrap();
                    let op2 = Operator::from_bytes(t2.value(parser.source)).unwrap();

                    if priority(op2) >= priority(op) {
                        output_queue.push(RPNItem::Operator(
                            Operator::from_bytes(
                                &operator_stack.pop().unwrap().value(parser.source),
                            )
                            .unwrap(),
                        ));
                    } else {
                        break;
                    }
                }

                operator_stack.push(t.clone());
            } else {
                last_was_scalar = true;

                let e = (Expression::get_shallow(&mut parser.clone())?)(parser)?;
                output_queue.push(RPNItem::Scalar(e));
            }
        }

        while let Some(t) = operator_stack.pop() {
            output_queue.push(RPNItem::Operator(
                Operator::from_bytes(t.value(parser.source)).unwrap(),
            ));
        }

        Ok(output_queue)
    }

    #[inline]
    fn consume(it: &mut impl Iterator<Item = RPNItem<'s>>) -> Result<Self, ()> {
        match it.next().ok_or(())? {
            RPNItem::Operator(op) => {
                let rhs = Self::consume(it)?;
                let lhs = Self::consume(it)?;

                Ok(Node::Compound(Box::new((lhs, op, rhs))))
            }
            RPNItem::Scalar(e) => Ok(Node::Scalar(e)),
        }
    }

    #[inline]
    pub fn parse<I: TokenIt>(parser: &mut Parser<'s, I>) -> Result<Self, ParserError> {
        let rpn = Self::shunting_yard(parser)?;

        let res = Self::consume(&mut rpn.into_iter().rev()).map_err(|_| {
            ParserError::ExpectedASTStructure {
                span: parser.current_span(),
                name: "Expression",
            }
        })?;

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use compiler_lexer::definitions::LiteralType;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::tests::parser;

    #[test]
    fn simple_binary_passes() {
        assert_eq!(
            Node::parse(&mut parser("9 + 10")),
            Ok(Node::Compound(Box::new((
                Node::Scalar(Expression::Literal {
                    value: b"9",
                    r#type: LiteralType::Int
                }),
                Operator::Plus,
                Node::Scalar(Expression::Literal {
                    value: b"10",
                    r#type: LiteralType::Int
                })
            ))))
        );
    }

    #[test]
    fn binary_with_unary_passes() {
        assert_eq!(
            Node::parse(&mut parser("10 - -1")),
            Ok(Node::Compound(Box::new((
                Node::Scalar(Expression::Literal {
                    value: b"10",
                    r#type: LiteralType::Int
                }),
                Operator::Minus,
                Node::Scalar(Expression::Unary(
                    Operator::Minus,
                    Box::new(Expression::Literal {
                        value: b"1",
                        r#type: LiteralType::Int
                    })
                ))
            ))))
        );
    }

    #[test]
    fn priority_binary_passes() {
        assert_eq!(
            Node::parse(&mut parser("9 - 2 * 4 + 1")),
            Ok(Node::Compound(Box::new((
                Node::Compound(Box::new((
                    Node::Scalar(Expression::Literal {
                        value: b"9",
                        r#type: LiteralType::Int
                    }),
                    Operator::Minus,
                    Node::Compound(Box::new((
                        Node::Scalar(Expression::Literal {
                            value: b"2",
                            r#type: LiteralType::Int
                        }),
                        Operator::Star,
                        Node::Scalar(Expression::Literal {
                            value: b"4",
                            r#type: LiteralType::Int
                        })
                    )))
                ))),
                Operator::Plus,
                Node::Scalar(Expression::Literal {
                    value: b"1",
                    r#type: LiteralType::Int
                })
            ))))
        );
    }

    #[test]
    fn custom_priority_binary_passes() {
        assert_eq!(
            Node::parse(&mut parser("9 - 2 * 4 >> 1")),
            Ok(Node::Compound(Box::new((
                Node::Compound(Box::new((
                    Node::Scalar(Expression::Literal {
                        value: b"9",
                        r#type: LiteralType::Int
                    }),
                    Operator::Minus,
                    Node::Compound(Box::new((
                        Node::Scalar(Expression::Literal {
                            value: b"2",
                            r#type: LiteralType::Int
                        }),
                        Operator::Star,
                        Node::Scalar(Expression::Literal {
                            value: b"4",
                            r#type: LiteralType::Int
                        })
                    )))
                ))),
                Operator::Shr,
                Node::Scalar(Expression::Literal {
                    value: b"1",
                    r#type: LiteralType::Int
                })
            ))))
        );
    }

    #[test]
    fn parenthesis_binary_passes() {
        assert_eq!(
            Node::parse(&mut parser("9 - 2 * (4 + 1)")),
            Ok(Node::Compound(Box::new((
                Node::Scalar(Expression::Literal {
                    value: b"9",
                    r#type: LiteralType::Int
                }),
                Operator::Minus,
                Node::Compound(Box::new((
                    Node::Scalar(Expression::Literal {
                        value: b"2",
                        r#type: LiteralType::Int
                    }),
                    Operator::Star,
                    Node::Scalar(Expression::Parenthesis(Box::new(Expression::Binary(
                        Box::new(Node::Compound(Box::new((
                            Node::Scalar(Expression::Literal {
                                value: b"4",
                                r#type: LiteralType::Int
                            }),
                            Operator::Plus,
                            Node::Scalar(Expression::Literal {
                                value: b"1",
                                r#type: LiteralType::Int
                            })
                        ))))
                    ))))
                )))
            ))))
        );
    }

    #[test]
    fn binary_with_call_passes() {
        assert_eq!(
            Node::parse(&mut parser("9 << 2 * (add(2, 4) + 1)")),
            Ok(Node::Compound(Box::new((
                Node::Scalar(Expression::Literal {
                    value: b"9",
                    r#type: LiteralType::Int
                }),
                Operator::Shl,
                Node::Compound(Box::new((
                    Node::Scalar(Expression::Literal {
                        value: b"2",
                        r#type: LiteralType::Int
                    }),
                    Operator::Star,
                    Node::Scalar(Expression::Parenthesis(Box::new(Expression::Binary(
                        Box::new(Node::Compound(Box::new((
                            Node::Scalar(Expression::Call {
                                path: [b"add" as &[_]].into(),
                                args: [
                                    Expression::Literal {
                                        value: b"2",
                                        r#type: LiteralType::Int
                                    },
                                    Expression::Literal {
                                        value: b"4",
                                        r#type: LiteralType::Int
                                    }
                                ]
                                .into()
                            }),
                            Operator::Plus,
                            Node::Scalar(Expression::Literal {
                                value: b"1",
                                r#type: LiteralType::Int
                            })
                        ))))
                    ))))
                )))
            ))))
        );
    }

    #[test]
    fn invalid_binary_passes() {
        assert_eq!(
            Node::parse(&mut parser("2 + 4 2")),
            Err(ParserError::ExpectedTokenType {
                r#type: "Operator",
                span: 6.into()
            })
        );

        assert_eq!(
            Node::parse(&mut parser("2 + 4 -")),
            Err(ParserError::ExpectedASTStructure {
                name: "Expression",
                span: (7..7).into()
            })
        );
    }
}
