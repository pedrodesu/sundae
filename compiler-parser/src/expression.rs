use compiler_lexer::definitions::{LiteralType, Token, TokenType};
use ecow::EcoString;
use operator::{to_operator, Operator};

use crate::{iterator::TokenItTrait, statement::Statement, ExhaustiveGet, TokenIt};

pub mod binary;
pub mod operator;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Literal {
        value: EcoString,
        r#type: LiteralType,
    },
    Path(Vec<EcoString>),
    Binary(binary::Node),
    Unary(Operator, Box<Expression>),
    Call {
        path: Vec<EcoString>,
        args: Vec<Expression>,
    },
    If {
        condition: Box<Expression>,
        block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
    },
}

impl<'a, I: TokenItTrait + 'a> ExhaustiveGet<'a, I> for Expression {
    const PARSE_OPTIONS: &'a [fn(&mut TokenIt<I>) -> Option<Self>] = &[
        Self::parse_if,
        Self::parse_binary,
        Self::parse_unary,
        Self::parse_call,
        Self::parse_path,
        Self::parse_literal,
    ];
}

impl Expression {
    #[inline]
    pub fn parse_literal<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Option<Self> {
        if let Token {
            value,
            r#type: TokenType::Literal(lit_type),
        } = tokens.0.next()?
        {
            Some(Self::Literal {
                value,
                r#type: lit_type,
            })
        } else {
            None
        }
    }

    pub fn parse_path<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Option<Self> {
        let mut path = Vec::new();

        while path.is_empty() || tokens.next(|t| t.value == ".").is_some() {
            let segment = tokens.next(|t| t.r#type == TokenType::Identifier)?.value;
            path.push(segment);
        }

        Some(Self::Path(path))
    }

    #[inline]
    pub fn parse_binary<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Option<Self> {
        binary::Node::parse(tokens).map(Self::Binary)
    }

    pub fn parse_call<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Option<Self> {
        let Self::Path(path) = Self::parse_path(tokens)? else {
            unreachable!()
        };

        let args = tokens.parse_generic_list(("(", ")"), |t| Expression::get(t), Some(","))?;

        Some(Self::Call { path, args })
    }

    pub fn parse_if<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Option<Self> {
        tokens.next(|t| t.value == "if")?;

        // TODO ignore_newlines might not be necessary? if when we get next we always skip newline. is this viable? try and test.
        tokens.ignore_newlines();

        let condition = Expression::get(tokens)?;

        tokens.ignore_newlines();

        let block = tokens.parse_block()?;

        tokens.ignore_newlines();

        let r#else = tokens.next(|t| t.value == "else").and_then(|_| {
            tokens.ignore_newlines();

            tokens.parse_block()
        });

        Some(Self::If {
            condition: Box::new(condition),
            block,
            else_block: r#else,
        })
    }

    #[inline]
    pub fn parse_unary<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Option<Self> {
        let operator = to_operator(&tokens.next(|t| t.r#type == TokenType::Operator)?);
        if !matches!(operator, Operator::Minus) {
            // TODO accept Result<Option< on parser too
            // bail!("Leading `{op}` not supported here")
            return None;
        }
        // Custom order, immediate literal should have priority over another binary expression
        let expression = [
            Self::parse_if,
            Self::parse_unary,
            Self::parse_call,
            Self::parse_path,
            Self::parse_literal,
            Self::parse_binary,
        ]
        .into_iter()
        .find(|&f| f(&mut tokens.clone()).is_some())?(tokens)
        .unwrap();

        Some(Self::Unary(operator, Box::new(expression)))
    }
}

#[cfg(test)]
mod tests {
    use crate::Node;

    use super::*;

    use pretty_assertions::assert_eq;

    // Expression::Literal and Expression::Binary are mere simple wrappers for already tested features, so we don't test them here

    #[test]
    fn path_passes() {
        assert_eq!(
            Expression::parse_path(&mut TokenIt(
                compiler_lexer::tokenize("a.path.to").flatten().peekable()
            ))
            .unwrap(),
            Expression::Path(vec!["a".into(), "path".into(), "to".into()])
        );
    }

    #[test]
    fn call_passes() {
        assert_eq!(
            Expression::parse_call(&mut TokenIt(
                compiler_lexer::tokenize("call_me(     )")
                    .flatten()
                    .peekable()
            ))
            .unwrap(),
            Expression::Call {
                path: vec!["call_me".into()],
                args: vec![]
            }
        );

        assert_eq!(
            Expression::parse_call(&mut TokenIt(
                compiler_lexer::tokenize("call  .me()").flatten().peekable()
            ))
            .unwrap(),
            Expression::Call {
                path: vec!["call".into(), "me".into()],
                args: vec![]
            }
        );

        assert_eq!(
            Expression::parse_call(&mut TokenIt(
                compiler_lexer::tokenize("fn    (2)").flatten().peekable()
            ))
            .unwrap(),
            Expression::Call {
                path: vec!["fn".into()],
                args: vec![Expression::Literal {
                    value: "2".into(),
                    r#type: LiteralType::Int
                }]
            }
        );

        assert_eq!(
            Expression::parse_call(&mut TokenIt(
                compiler_lexer::tokenize("fn. path(\n\n\n420,`j`\n\n ,\n6\n)")
                    .flatten()
                    .peekable()
            ))
            .unwrap(),
            Expression::Call {
                path: vec!["fn".into(), "path".into()],
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
            }
        );

        assert!(Expression::parse_call(&mut TokenIt(
            compiler_lexer::tokenize("fn.()").flatten().peekable()
        ))
        .is_none());

        assert!(Expression::parse_call(&mut TokenIt(
            compiler_lexer::tokenize("fn(42, )").flatten().peekable()
        ))
        .is_none());

        assert!(Expression::parse_call(&mut TokenIt(
            compiler_lexer::tokenize("fn(, 42)").flatten().peekable()
        ))
        .is_none());
    }

    #[test]
    fn if_passes() {
        assert_eq!(
            Expression::parse_if(&mut TokenIt(
                compiler_lexer::tokenize("if 1 {}").flatten().peekable()
            ))
            .unwrap(),
            Expression::If {
                condition: Box::new(Expression::Literal {
                    value: "1".into(),
                    r#type: LiteralType::Int
                }),
                block: vec![],
                else_block: None
            }
        );

        assert_eq!(
            Expression::parse_if(&mut TokenIt(
                compiler_lexer::tokenize("if 1 {} else {}")
                    .flatten()
                    .peekable()
            ))
            .unwrap(),
            Expression::If {
                condition: Box::new(Expression::Literal {
                    value: "1".into(),
                    r#type: LiteralType::Int
                }),
                block: vec![],
                else_block: Some(vec![])
            }
        );

        assert_eq!(
            Expression::parse_if(&mut TokenIt(
                compiler_lexer::tokenize("if 2 + 2 {\ncall()\n}")
                    .flatten()
                    .peekable()
            ))
            .unwrap(),
            Expression::If {
                condition: Box::new(Expression::Binary(Node::Compound(
                    Box::new(Node::Scalar(Box::new(Expression::Literal {
                        value: "2".into(),
                        r#type: LiteralType::Int
                    }))),
                    Operator::Plus,
                    Box::new(Node::Scalar(Box::new(Expression::Literal {
                        value: "2".into(),
                        r#type: LiteralType::Int
                    }))),
                ))),
                block: vec![Statement::Expression(Expression::Call {
                    path: vec!["call".into()],
                    args: vec![]
                })],
                else_block: None
            }
        );

        assert_eq!(
            Expression::parse_if(&mut TokenIt(
                compiler_lexer::tokenize("if 1 { call() }")
                    .flatten()
                    .peekable()
            ))
            .unwrap(),
            Expression::If {
                condition: Box::new(
                    Expression::Literal {
                        value: "1".into(),
                        r#type: LiteralType::Int
                    }
                ),
                block: vec![Statement::Expression(Expression::Call {
                    path: vec!["call".into()],
                    args: vec![]
                })],
                else_block: None
            }
        );

        assert_eq!(
            Expression::parse_if(&mut TokenIt(
                compiler_lexer::tokenize("if 1{ call() }else { other_call()}")
                    .flatten()
                    .peekable()
            ))
            .unwrap(),
            Expression::If {
                condition: Box::new(
                    Expression::Literal {
                        value: "1".into(),
                        r#type: LiteralType::Int
                    }
                ),
                block: vec![
                    Statement::Expression(
                        Expression::Call {
                            path: vec!["call".into()],
                            args: vec![]
                        }
                    )
                ],
                else_block: Some(
                    vec![
                        Statement::Expression(
                            Expression::Call {
                                path: vec!["other_call".into()],
                                args: vec![]
                            }
                        )
                    ]
                )
            }
        );

        assert_eq!(
            Expression::parse_if(&mut TokenIt(
                compiler_lexer::tokenize(
                    "if\n\n2 + 2\n\n{\n   \n  call  ()\n}\n\t  \nelse\n  \n\n{\n\n42\n\n\n}\n\n"
                )
                .flatten()
                .peekable()
            ))
            .unwrap(),
            Expression::If {
                condition: Box::new(Expression::Binary(Node::Compound(
                    Box::new(Node::Scalar(Box::new(Expression::Literal {
                        value: "2".into(),
                        r#type: LiteralType::Int
                    }))),
                    Operator::Plus,
                    Box::new(Node::Scalar(Box::new(Expression::Literal {
                        value: "2".into(),
                        r#type: LiteralType::Int
                    }))),
                ))),
                block: vec![Statement::Expression(Expression::Call {
                    path: vec!["call".into()],
                    args: vec![]
                })],
                else_block: Some(vec![Statement::Expression(Expression::Literal {
                    value: "42".into(),
                    r#type: LiteralType::Int
                })])
            }
        );
    }

    // TODO impl if and unary extensive tests
    #[test]
    fn unary_passes() {
        assert_eq!(
            Expression::parse_unary(&mut TokenIt(
                compiler_lexer::tokenize("-2").flatten().peekable()
            ))
            .unwrap(),
            Expression::Unary(
                Operator::Minus,
                Box::new(Expression::Literal {
                    value: "2".into(),
                    r#type: LiteralType::Int
                })
            )
        );

        assert_eq!(
            Expression::parse_unary(&mut TokenIt(
                compiler_lexer::tokenize("-(2 - 4)").flatten().peekable()
            ))
            .unwrap(),
            Expression::Unary(
                Operator::Minus,
                Box::new(Expression::Binary(Node::Compound(
                    Box::new(Node::Scalar(Box::new(Expression::Literal {
                        value: "2".into(),
                        r#type: LiteralType::Int
                    }))),
                    Operator::Minus,
                    Box::new(Node::Scalar(Box::new(Expression::Literal {
                        value: "4".into(),
                        r#type: LiteralType::Int
                    }))),
                )))
            )
        );

        assert_eq!(
            Expression::parse_unary(&mut TokenIt(
                compiler_lexer::tokenize("+2").flatten().peekable()
            )),
            None
        );
    }
}
