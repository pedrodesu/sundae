use std::fmt;

use compiler_lexer::definitions::Token;

use crate::{iterator::TokenItTrait, ExhaustiveGet, TokenIt};

use super::Expression;

const OPERATOR_MAP: &[(&str, Operator)] = {
    use Operator::*;

    &[
        ("+", Sum),
        ("-", Sub),
        ("*", Star),
        ("/", Div),
        ("and", And),
        ("or", Or),
        ("<", Lt),
        (">", Gt),
        ("<=", Le),
        (">=", Ge),
        ("==", EqEq),
        ("!=", Neq),
        ("<<", Shl),
        (">>", Shr),
        ("&", ShAnd),
        ("|", ShOr),
        ("^", Xor),
    ]
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Operator {
    Sum,
    Sub,
    Star,
    Div,
    And,
    Or,
    Lt,
    Gt,
    Le,
    Ge,
    EqEq,
    Neq,
    Shl,
    Shr,
    ShAnd,
    ShOr,
    Xor,
}

impl Operator {
    const TERMS: &'static [Self] = {
        use Operator::*;

        &[
            Sum, Sub, And, Or, Lt, Gt, Le, Ge, EqEq, Neq, Shl, Shr, ShAnd, ShOr, Xor,
        ]
    };

    const FACTORS: &'static [Self] = {
        use Operator::*;

        &[Star, Div]
    };
}

impl TryFrom<&Token> for Operator {
    type Error = ();

    #[inline]
    fn try_from(token: &Token) -> Result<Self, Self::Error> {
        OPERATOR_MAP
            .iter()
            .copied()
            .find(|&(k, _)| k == token.value.as_str())
            .map(|(_, v)| v)
            .ok_or(())
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            OPERATOR_MAP
                .iter()
                .copied()
                .find(|&(_, v)| v == *self)
                .map(|(k, _)| k)
                .unwrap()
        )
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub enum BinaryNode {
    Scalar(Box<Expression>),
    Compound(Box<BinaryNode>, Operator, Box<BinaryNode>),
}

impl BinaryNode {
    fn term<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Option<Self> {
        Some(Self::Scalar(Box::new(
            Expression::PARSE_OPTIONS
                .iter()
                .filter(|&&f| f != Expression::parse_binary)
                .find(|&&f| f(&mut tokens.clone()).is_some())?(tokens)
            .unwrap(),
        )))
    }

    fn factor<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Option<Self> {
        let mut acc = Self::term(tokens)?;
        while let Some(t) = tokens.0.next_if(|t| {
            Operator::try_from(t)
                .ok()
                .is_some_and(|op| Operator::FACTORS.contains(&op))
        }) {
            let next = Self::term(tokens)?;
            acc = Self::Compound(Box::new(acc), Operator::try_from(&t).ok()?, Box::new(next));
        }
        Some(acc)
    }

    fn consume<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Option<Self> {
        let mut acc = Self::factor(tokens)?;
        // TODO manage whitespace between binary (and multiple other instances, such as assign)
        while let Some(t) = tokens.0.next_if(|t| {
            Operator::try_from(t)
                .ok()
                .is_some_and(|op| Operator::TERMS.contains(&op))
        }) {
            let next = Self::factor(tokens)?;
            acc = Self::Compound(Box::new(acc), Operator::try_from(&t).ok()?, Box::new(next));
        }
        if let Self::Compound(..) = acc {
            Some(acc)
        } else {
            None
        }
    }

    #[inline]
    pub fn parse<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Option<Self> {
        Self::consume(tokens)
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
            BinaryNode::parse(&mut TokenIt(
                compiler_lexer::tokenize("9 + 10").flatten().peekable()
            ))
            .unwrap(),
            BinaryNode::Compound(
                Box::new(BinaryNode::Scalar(Box::new(Expression::Literal {
                    value: "9".into(),
                    r#type: LiteralType::Int
                }))),
                Operator::Sum,
                Box::new(BinaryNode::Scalar(Box::new(Expression::Literal {
                    value: "10".into(),
                    r#type: LiteralType::Int
                })))
            )
        );
    }

    #[test]
    fn priority_binary_passes() {
        assert_eq!(
            BinaryNode::parse(&mut TokenIt(
                compiler_lexer::tokenize("9 - 2 * 4 + 1")
                    .flatten()
                    .peekable()
            ))
            .unwrap(),
            BinaryNode::Compound(
                Box::new(BinaryNode::Compound(
                    Box::new(BinaryNode::Scalar(Box::new(Expression::Literal {
                        value: "9".into(),
                        r#type: LiteralType::Int
                    }))),
                    Operator::Sub,
                    Box::new(BinaryNode::Compound(
                        Box::new(BinaryNode::Scalar(Box::new(Expression::Literal {
                            value: "2".into(),
                            r#type: LiteralType::Int
                        }))),
                        Operator::Star,
                        Box::new(BinaryNode::Scalar(Box::new(Expression::Literal {
                            value: "4".into(),
                            r#type: LiteralType::Int
                        })))
                    ))
                )),
                Operator::Sum,
                Box::new(BinaryNode::Scalar(Box::new(Expression::Literal {
                    value: "1".into(),
                    r#type: LiteralType::Int
                })))
            )
        );
    }

    #[test]
    fn parenthesis_binary_passes() {
        assert_eq!(
            BinaryNode::parse(&mut TokenIt(
                compiler_lexer::tokenize("9 - 2 * (4 + 1)")
                    .flatten()
                    .peekable()
            ))
            .unwrap(),
            BinaryNode::Compound(
                Box::new(BinaryNode::Compound(
                    Box::new(BinaryNode::Scalar(Box::new(Expression::Literal {
                        value: "9".into(),
                        r#type: LiteralType::Int
                    }))),
                    Operator::Sub,
                    Box::new(BinaryNode::Scalar(Box::new(Expression::Literal {
                        value: "2".into(),
                        r#type: LiteralType::Int
                    }))),
                )),
                Operator::Star,
                Box::new(BinaryNode::Compound(
                    Box::new(BinaryNode::Scalar(Box::new(Expression::Literal {
                        value: "4".into(),
                        r#type: LiteralType::Int
                    }))),
                    Operator::Star,
                    Box::new(BinaryNode::Scalar(Box::new(Expression::Literal {
                        value: "1".into(),
                        r#type: LiteralType::Int
                    })))
                ))
            )
        );
    }
}
