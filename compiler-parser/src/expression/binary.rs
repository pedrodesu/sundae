use compiler_lexer::definitions::{Token, TokenType};
use ecow::EcoVec;

use super::{
    Expression,
    operator::{Operator, to_operator},
};
use crate::{ParserError, TokenIt, iterator::TokenItTrait};

const OPERATOR_PRIORITY: &[&[Operator]] = {
    use Operator::*;

    &[&[Plus, Minus], &[Star, Div]]
};

#[inline]
fn priority(operator: Operator) -> usize
{
    OPERATOR_PRIORITY
        .iter()
        .copied()
        .position(|v| v.contains(&operator))
        .map(|v| v + 1)
        // Custom operators are of the least priority, as they should be evaluated last, after every mathematical operator
        .unwrap_or_default()
}

#[derive(Clone, Debug, PartialEq)]
pub enum Node
{
    Scalar(Expression),
    Compound(Box<(Node, Operator, Node)>),
}

#[derive(Debug, PartialEq, Clone)]
enum RPNItem
{
    Scalar(Expression),
    Operator(Operator),
}

impl Node
{
    // We do not need to concern ourselves with unary operators or parenthesis here because we already handle them as singular, regular expressions.
    // This makes our shunting yard much simpler.
    fn shunting_yard<I: TokenItTrait>(
        tokens: &mut TokenIt<I>,
    ) -> Result<EcoVec<RPNItem>, ParserError>
    {
        let mut output_queue = EcoVec::new();
        let mut operator_stack = EcoVec::new();

        let mut last_was_scalar = false;

        while let Some(p) = tokens.0.peek()
            && (p.r#type != TokenType::Separator || p.value == "(")
        {
            if last_was_scalar
            {
                last_was_scalar = false;

                let t = tokens
                    .next(|t| t.r#type == TokenType::Operator)
                    .ok_or(ParserError::ExpectedTokenType { r#type: "Operator" })?;

                while let Some(
                    t2 @ Token {
                        r#type: TokenType::Operator,
                        ..
                    },
                ) = operator_stack.last()
                {
                    let op = to_operator(&t);
                    let op2 = to_operator(&t2);

                    if priority(op2) >= priority(op)
                    {
                        output_queue.push(RPNItem::Operator(to_operator(
                            &operator_stack.pop().unwrap(),
                        )));
                    }
                    else
                    {
                        break;
                    }
                }

                operator_stack.push(t.clone());
            }
            else
            {
                last_was_scalar = true;

                let e = (Expression::shallow_find_predicate(&mut tokens.clone())?)(tokens)?;
                output_queue.push(RPNItem::Scalar(e));
            }
        }

        while let Some(t) = operator_stack.pop()
        {
            output_queue.push(RPNItem::Operator(to_operator(&t)));
        }

        Ok(output_queue)
    }

    #[inline]
    fn consume(it: &mut impl Iterator<Item = RPNItem>) -> Result<Self, ParserError>
    {
        match it
            .next()
            .ok_or(ParserError::ExpectedASTStructure { name: "Expression" })?
        {
            RPNItem::Operator(op) =>
            {
                let rhs = Self::consume(it)?;
                let lhs = Self::consume(it)?;

                Ok(Node::Compound(Box::new((lhs, op, rhs))))
            }
            RPNItem::Scalar(e) => Ok(Node::Scalar(e)),
        }
    }

    #[inline]
    pub fn parse<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Result<Self, ParserError>
    {
        let rpn = Self::shunting_yard(tokens)?;

        let res = Self::consume(&mut rpn.into_iter().rev())?;

        Ok(res)
    }
}

#[cfg(test)]
mod tests
{
    use compiler_lexer::definitions::LiteralType;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn simple_binary_passes()
    {
        assert_eq!(
            Node::parse(&mut TokenIt(
                compiler_lexer::tokenize("9 + 10").flatten().peekable()
            )),
            Ok(Node::Compound(Box::new((
                Node::Scalar(Expression::Literal {
                    value: "9".into(),
                    r#type: LiteralType::Int
                }),
                Operator::Plus,
                Node::Scalar(Expression::Literal {
                    value: "10".into(),
                    r#type: LiteralType::Int
                })
            ))))
        );
    }

    #[test]
    fn hacky_binary_with_unary_passes()
    {
        assert_eq!(
            Node::parse(&mut TokenIt(
                compiler_lexer::tokenize("10 - -1").flatten().peekable()
            )),
            Ok(Node::Compound(Box::new((
                Node::Scalar(Expression::Literal {
                    value: "10".into(),
                    r#type: LiteralType::Int
                }),
                Operator::Minus,
                Node::Scalar(Expression::Unary(
                    Operator::Minus,
                    Box::new(Expression::Literal {
                        value: "1".into(),
                        r#type: LiteralType::Int
                    })
                ))
            ))))
        );
    }

    #[test]
    fn priority_binary_passes()
    {
        assert_eq!(
            Node::parse(&mut TokenIt(
                compiler_lexer::tokenize("9 - 2 * 4 + 1")
                    .flatten()
                    .peekable()
            )),
            Ok(Node::Compound(Box::new((
                Node::Compound(Box::new((
                    Node::Scalar(Expression::Literal {
                        value: "9".into(),
                        r#type: LiteralType::Int
                    }),
                    Operator::Minus,
                    Node::Compound(Box::new((
                        Node::Scalar(Expression::Literal {
                            value: "2".into(),
                            r#type: LiteralType::Int
                        }),
                        Operator::Star,
                        Node::Scalar(Expression::Literal {
                            value: "4".into(),
                            r#type: LiteralType::Int
                        })
                    )))
                ))),
                Operator::Plus,
                Node::Scalar(Expression::Literal {
                    value: "1".into(),
                    r#type: LiteralType::Int
                })
            ))))
        );
    }

    #[test]
    fn custom_priority_binary_passes()
    {
        assert_eq!(
            Node::parse(&mut TokenIt(
                compiler_lexer::tokenize("9 - 2 * 4 >> 1")
                    .flatten()
                    .peekable()
            )),
            Ok(Node::Compound(Box::new((
                Node::Compound(Box::new((
                    Node::Scalar(Expression::Literal {
                        value: "9".into(),
                        r#type: LiteralType::Int
                    }),
                    Operator::Minus,
                    Node::Compound(Box::new((
                        Node::Scalar(Expression::Literal {
                            value: "2".into(),
                            r#type: LiteralType::Int
                        }),
                        Operator::Star,
                        Node::Scalar(Expression::Literal {
                            value: "4".into(),
                            r#type: LiteralType::Int
                        })
                    )))
                ))),
                Operator::Shr,
                Node::Scalar(Expression::Literal {
                    value: "1".into(),
                    r#type: LiteralType::Int
                })
            ))))
        );
    }

    #[test]
    fn parenthesis_binary_passes()
    {
        assert_eq!(
            Node::parse(&mut TokenIt(
                compiler_lexer::tokenize("9 - 2 * (4 + 1)")
                    .flatten()
                    .peekable()
            )),
            Ok(Node::Compound(Box::new((
                Node::Scalar(Expression::Literal {
                    value: "9".into(),
                    r#type: LiteralType::Int
                }),
                Operator::Minus,
                Node::Compound(Box::new((
                    Node::Scalar(Expression::Literal {
                        value: "2".into(),
                        r#type: LiteralType::Int
                    }),
                    Operator::Star,
                    Node::Scalar(Expression::Parenthesis(Box::new(Expression::Binary(
                        Box::new(Node::Compound(Box::new((
                            Node::Scalar(Expression::Literal {
                                value: "4".into(),
                                r#type: LiteralType::Int
                            }),
                            Operator::Plus,
                            Node::Scalar(Expression::Literal {
                                value: "1".into(),
                                r#type: LiteralType::Int
                            })
                        ))))
                    ))))
                )))
            ))))
        );
    }

    #[test]
    fn binary_with_call_passes()
    {
        assert_eq!(
            Node::parse(&mut TokenIt(
                compiler_lexer::tokenize("9 << 2 * (add(2, 4) + 1)")
                    .flatten()
                    .peekable()
            )),
            Ok(Node::Compound(Box::new((
                Node::Scalar(Expression::Literal {
                    value: "9".into(),
                    r#type: LiteralType::Int
                }),
                Operator::Shl,
                Node::Compound(Box::new((
                    Node::Scalar(Expression::Literal {
                        value: "2".into(),
                        r#type: LiteralType::Int
                    }),
                    Operator::Star,
                    Node::Scalar(Expression::Parenthesis(Box::new(Expression::Binary(
                        Box::new(Node::Compound(Box::new((
                            Node::Scalar(Expression::Call {
                                path: vec!["add".into()].into(),
                                args: vec![
                                    Expression::Literal {
                                        value: "2".into(),
                                        r#type: LiteralType::Int
                                    },
                                    Expression::Literal {
                                        value: "4".into(),
                                        r#type: LiteralType::Int
                                    }
                                ]
                                .into()
                            }),
                            Operator::Plus,
                            Node::Scalar(Expression::Literal {
                                value: "1".into(),
                                r#type: LiteralType::Int
                            })
                        ))))
                    ))))
                )))
            ))))
        );
    }

    #[test]
    fn invalid_binary_passes()
    {
        assert_eq!(
            Node::parse(&mut TokenIt(
                compiler_lexer::tokenize("2 + 4 2").flatten().peekable(),
            )),
            Err(ParserError::ExpectedTokenType { r#type: "Operator" })
        );

        assert_eq!(
            Node::parse(&mut TokenIt(
                compiler_lexer::tokenize("2 + 4 -").flatten().peekable(),
            )),
            Err(ParserError::ExpectedASTStructure { name: "Expression" })
        );
    }
}
