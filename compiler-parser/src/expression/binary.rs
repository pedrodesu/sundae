use std::collections::VecDeque;

use compiler_lexer::definitions::TokenType;

use crate::{iterator::TokenItTrait, ExhaustiveGet, TokenIt};

use super::{
    operator::{to_operator, Operator},
    Expression,
};

const OPERATOR_PRIORITY: &[&[Operator]] = {
    use Operator::*;

    &[&[Plus, Minus], &[Star, Div]]
};

// TODO const priority() when Option::unwrap_or is const
#[inline]
fn priority(operator: Operator) -> usize {
    OPERATOR_PRIORITY
        .into_iter()
        .copied()
        .position(|v| v.contains(&operator))
        .map(|v| v + 1)
        // Custom operators are of the least priority, as they should be evaluated last, after every mathematical operator
        .unwrap_or_default()
}

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Scalar(Box<Expression>),
    Compound(Box<Node>, Operator, Box<Node>),
}

#[derive(Debug, PartialEq, Clone)]
enum RPNItem {
    Scalar(Expression),
    Operator(Operator),
    LParenthesis,
}

impl Node {
    // TODO this function looks kinda terrible rn but works for every use case, refactor
    fn to_reverse_polish<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Option<Vec<RPNItem>> {
        let mut output = Vec::new();
        let mut operators = VecDeque::new();

        let mut last_was_scalar = false;

        while tokens.0.peek().is_some() {
            if let Some(e_predicate) = Expression::PARSE_OPTIONS
                .iter()
                .filter(|&&f| {
                    // We cannot allow getting another binary immediately inside binary expression because then we would do it infinitely and get an overflow
                    // We only allow unary if we already have an operator, else "unary's operator" is actually a binary expression operator
                    if last_was_scalar {
                        f != Expression::parse_binary && f != Expression::parse_unary
                    } else {
                        f != Expression::parse_binary
                    }
                })
                .find(|&&f| f(&mut tokens.clone()).is_some())
            {
                if last_was_scalar {
                    return None;
                }
                output.push(RPNItem::Scalar(e_predicate(tokens).unwrap()));
                last_was_scalar = true;
            } else {
                let Some(t) = tokens.0.next_if(|t| {
                    t.r#type == TokenType::Operator || t.value == "(" || t.value == ")"
                }) else {
                    break;
                };

                if t.r#type == TokenType::Operator {
                    if !last_was_scalar {
                        return None;
                    }
                    let op = to_operator(&t);
                    while let Some(RPNItem::Operator(prev_op)) = operators.front()
                        && (priority(op) <= priority(*prev_op))
                    {
                        output.push(operators.pop_front().unwrap());
                    }
                    operators.push_front(RPNItem::Operator(op));
                    last_was_scalar = false;
                } else if t.value == "(" {
                    operators.push_front(RPNItem::LParenthesis);
                } else if t.value == ")" {
                    if !operators.contains(&RPNItem::LParenthesis) {
                        return None;
                    }
                    while let Some(op) = operators.pop_front() {
                        if matches!(op, RPNItem::LParenthesis) {
                            break;
                        }
                        output.push(op);
                    }
                } else {
                    unreachable!()
                }
            }
        }

        output.extend(operators);

        if !output.is_empty() {
            Some(output)
        } else {
            None
        }
    }

    #[inline]
    fn consume(it: &mut impl Iterator<Item = RPNItem>) -> Self {
        match it.next() {
            Some(RPNItem::Operator(op)) => {
                let rhs = Self::consume(it);
                let lhs = Self::consume(it);

                Node::Compound(Box::new(lhs), op, Box::new(rhs))
            }
            Some(RPNItem::Scalar(e)) => Node::Scalar(Box::new(e)),
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn parse<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Option<Self> {
        let rpn = Self::to_reverse_polish(tokens)?;

        let res = Self::consume(&mut rpn.into_iter().rev());
        if !matches!(res, Self::Scalar(_)) {
            Some(res)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use compiler_lexer::definitions::LiteralType;

    use pretty_assertions::assert_eq;

    #[test]
    fn simple_binary_passes() {
        assert_eq!(
            Node::parse(&mut TokenIt(
                compiler_lexer::tokenize("9 + 10").flatten().peekable()
            ))
            .unwrap(),
            Node::Compound(
                Box::new(Node::Scalar(Box::new(Expression::Literal {
                    value: "9".into(),
                    r#type: LiteralType::Int
                }))),
                Operator::Plus,
                Box::new(Node::Scalar(Box::new(Expression::Literal {
                    value: "10".into(),
                    r#type: LiteralType::Int
                })))
            )
        );
    }

    #[test]
    fn hacky_binary_with_unary_passes() {
        assert_eq!(
            Node::parse(&mut TokenIt(
                compiler_lexer::tokenize("10 - -1").flatten().peekable()
            ))
            .unwrap(),
            Node::Compound(
                Box::new(Node::Scalar(Box::new(Expression::Literal {
                    value: "10".into(),
                    r#type: LiteralType::Int
                }))),
                Operator::Minus,
                Box::new(Node::Scalar(Box::new(Expression::Unary(
                    Operator::Minus,
                    Box::new(Expression::Literal {
                        value: "1".into(),
                        r#type: LiteralType::Int
                    })
                ))))
            )
        );
    }

    #[test]
    fn priority_binary_passes() {
        assert_eq!(
            Node::parse(&mut TokenIt(
                compiler_lexer::tokenize("9 - 2 * 4 + 1")
                    .flatten()
                    .peekable()
            ))
            .unwrap(),
            Node::Compound(
                Box::new(Node::Compound(
                    Box::new(Node::Scalar(Box::new(Expression::Literal {
                        value: "9".into(),
                        r#type: LiteralType::Int
                    }))),
                    Operator::Minus,
                    Box::new(Node::Compound(
                        Box::new(Node::Scalar(Box::new(Expression::Literal {
                            value: "2".into(),
                            r#type: LiteralType::Int
                        }))),
                        Operator::Star,
                        Box::new(Node::Scalar(Box::new(Expression::Literal {
                            value: "4".into(),
                            r#type: LiteralType::Int
                        })))
                    ))
                )),
                Operator::Plus,
                Box::new(Node::Scalar(Box::new(Expression::Literal {
                    value: "1".into(),
                    r#type: LiteralType::Int
                }))),
            )
        );
    }

    #[test]
    fn custom_priority_binary_passes() {
        assert_eq!(
            Node::parse(&mut TokenIt(
                compiler_lexer::tokenize("9 - 2 * 4 >> 1")
                    .flatten()
                    .peekable()
            ))
            .unwrap(),
            Node::Compound(
                Box::new(Node::Compound(
                    Box::new(Node::Scalar(Box::new(Expression::Literal {
                        value: "9".into(),
                        r#type: LiteralType::Int
                    }))),
                    Operator::Minus,
                    Box::new(Node::Compound(
                        Box::new(Node::Scalar(Box::new(Expression::Literal {
                            value: "2".into(),
                            r#type: LiteralType::Int
                        }))),
                        Operator::Star,
                        Box::new(Node::Scalar(Box::new(Expression::Literal {
                            value: "4".into(),
                            r#type: LiteralType::Int
                        })))
                    )),
                )),
                Operator::Shr,
                Box::new(Node::Scalar(Box::new(Expression::Literal {
                    value: "1".into(),
                    r#type: LiteralType::Int
                }))),
            )
        );
    }

    #[test]
    fn parenthesis_binary_passes() {
        assert_eq!(
            Node::parse(&mut TokenIt(
                compiler_lexer::tokenize("(4 + 1)").flatten().peekable()
            ))
            .unwrap(),
            Node::Compound(
                Box::new(Node::Scalar(Box::new(Expression::Literal {
                    value: "4".into(),
                    r#type: LiteralType::Int
                }))),
                Operator::Plus,
                Box::new(Node::Scalar(Box::new(Expression::Literal {
                    value: "1".into(),
                    r#type: LiteralType::Int
                })))
            )
        );

        assert_eq!(
            Node::parse(&mut TokenIt(
                compiler_lexer::tokenize("9 - 2 * (4 + 1)")
                    .flatten()
                    .peekable()
            ))
            .unwrap(),
            Node::Compound(
                Box::new(Node::Scalar(Box::new(Expression::Literal {
                    value: "9".into(),
                    r#type: LiteralType::Int
                }))),
                Operator::Minus,
                Box::new(Node::Compound(
                    Box::new(Node::Scalar(Box::new(Expression::Literal {
                        value: "2".into(),
                        r#type: LiteralType::Int
                    }))),
                    Operator::Star,
                    Box::new(Node::Compound(
                        Box::new(Node::Scalar(Box::new(Expression::Literal {
                            value: "4".into(),
                            r#type: LiteralType::Int
                        }))),
                        Operator::Plus,
                        Box::new(Node::Scalar(Box::new(Expression::Literal {
                            value: "1".into(),
                            r#type: LiteralType::Int
                        })))
                    ))
                ))
            )
        );
    }

    #[test]
    fn binary_with_call_passes() {
        assert_eq!(
            Node::parse(&mut TokenIt(
                compiler_lexer::tokenize("9 << 2 * (add(2, 4) + 1)")
                    .flatten()
                    .peekable()
            ))
            .unwrap(),
            Node::Compound(
                Box::new(Node::Scalar(Box::new(Expression::Literal {
                    value: "9".into(),
                    r#type: LiteralType::Int
                }))),
                Operator::Shl,
                Box::new(Node::Compound(
                    Box::new(Node::Scalar(Box::new(Expression::Literal {
                        value: "2".into(),
                        r#type: LiteralType::Int
                    }))),
                    Operator::Star,
                    Box::new(Node::Compound(
                        Box::new(Node::Scalar(Box::new(Expression::Call {
                            path: vec!["add".into()],
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
                        }))),
                        Operator::Plus,
                        Box::new(Node::Scalar(Box::new(Expression::Literal {
                            value: "1".into(),
                            r#type: LiteralType::Int
                        })))
                    ))
                ))
            )
        );
    }

    #[test]
    #[should_panic]
    fn invalid_binary_passes() {
        Node::parse(&mut TokenIt(
            compiler_lexer::tokenize("2 + 4 2").flatten().peekable(),
        ))
        .unwrap();
    }
}
