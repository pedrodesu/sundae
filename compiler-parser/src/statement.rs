use compiler_lexer::definitions::{Token, TokenType};
use itertools::Itertools;

use crate::{expression::Expression, iterator::TokenItTrait, ExhaustiveGet, Name, TokenIt, Type};

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Return(Option<Expression>),
    Expression(Expression),
    Assign {
        destination: Expression,
        source: Expression,
    },
    Local {
        mutable: bool,
        name: Name,
        init: Option<Expression>,
    },
}

impl<'a, I: TokenItTrait + 'a> ExhaustiveGet<'a, I> for Statement {
    const PARSE_OPTIONS: &'a [fn(&mut TokenIt<I>) -> Option<Self>] = &[
        Self::parse_return,
        Self::parse_assign,
        Self::parse_local,
        Self::parse_expression,
    ];
}

impl Statement {
    #[inline]
    fn assert_end<I: TokenItTrait>(
        tokens: &mut TokenIt<I>,
        predicate: impl FnOnce(&mut TokenIt<I>) -> Option<Self>,
    ) -> Option<Self> {
        let value = predicate(tokens)?;

        let Some(Token { r#type: TokenType::Separator | TokenType::Newline, .. }) = tokens.0.peek() else { return None };

        Some(value)
    }

    #[inline]
    pub fn parse_return<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Option<Self> {
        Self::assert_end(tokens, |tokens| {
            tokens.next(|t| t.value == "ret")?;

            Some(Self::Return(
                if tokens.0.peek()?.r#type != TokenType::Newline {
                    Some(Expression::get(tokens)?)
                } else {
                    None
                },
            ))
        })
    }

    #[inline]
    pub fn parse_expression<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Option<Self> {
        Self::assert_end(tokens, |tokens| {
            Expression::get(tokens).map(Self::Expression)
        })
    }

    pub fn parse_assign<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Option<Self> {
        Self::assert_end(tokens, |tokens| {
            let destination = Expression::get(tokens)?;

            tokens.next(|t| t.value == "=")?;

            let source = Expression::get(tokens)?;

            Some(Self::Assign {
                destination,
                source,
            })
        })
    }

    pub fn parse_local<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Option<Self> {
        Self::assert_end(tokens, |tokens| {
            tokens.next(|t| t.value == "val")?;

            let identifier = tokens.next(|t| t.r#type == TokenType::Identifier)?.value;

            let mutable = tokens.next(|t| t.value == "mut").is_some();

            let r#type = if tokens.0.peek()?.value != "=" {
                Some(Type(
                    tokens
                        .0
                        .peeking_take_while(|t| t.value != "=" && t.r#type != TokenType::Newline)
                        .map(|t| t.value)
                        .collect(),
                ))
            } else {
                None
            };

            let init = tokens
                .next(|t| t.value == "=")
                .and_then(|_| Expression::get(tokens));

            Some(Self::Local {
                name: Name(identifier, r#type),
                mutable,
                init,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use compiler_lexer::definitions::LiteralType;
    use pretty_assertions::assert_eq;

    // Statement::Expression is a mere simple wrapper for an already tested feature, so we don't test it here

    #[test]
    fn return_passes() {
        assert_eq!(
            Statement::parse_return(&mut TokenIt(
                compiler_lexer::tokenize("ret \n").flatten().peekable()
            ))
            .unwrap(),
            Statement::Return(None)
        );

        assert_eq!(
            Statement::parse_return(&mut TokenIt(
                compiler_lexer::tokenize("ret 42\n").flatten().peekable()
            ))
            .unwrap(),
            Statement::Return(Some(Expression::Literal {
                value: "42".into(),
                r#type: LiteralType::Int
            }))
        );

        assert_eq!(
            Statement::parse_return(&mut TokenIt(
                compiler_lexer::tokenize("ret ret\n\n").flatten().peekable()
            )),
            None
        );
    }

    #[test]
    fn assign_passes() {
        assert_eq!(
            Statement::parse_assign(&mut TokenIt(
                // Can lhs of an assign expression ever be something else other than a path?
                compiler_lexer::tokenize("a = 2\n").flatten().peekable()
            ))
            .unwrap(),
            Statement::Assign {
                destination: Expression::Path(vec!["a".into()]),
                source: Expression::Literal {
                    value: "2".into(),
                    r#type: LiteralType::Int
                }
            }
        );
    }

    #[test]
    fn local_passes() {
        assert_eq!(
            Statement::parse_local(&mut TokenIt(
                compiler_lexer::tokenize("val a = 2\n").flatten().peekable()
            ))
            .unwrap(),
            Statement::Local {
                mutable: false,
                name: Name("a".into(), None),
                init: Some(Expression::Literal {
                    value: "2".into(),
                    r#type: LiteralType::Int
                })
            }
        );

        assert_eq!(
            Statement::parse_local(&mut TokenIt(
                compiler_lexer::tokenize("val b i32 = 4\n")
                    .flatten()
                    .peekable()
            ))
            .unwrap(),
            Statement::Local {
                mutable: false,
                name: Name("b".into(), Some(Type(vec!["i32".into()]))),
                init: Some(Expression::Literal {
                    value: "4".into(),
                    r#type: LiteralType::Int
                })
            }
        );

        assert_eq!(
            Statement::parse_local(&mut TokenIt(
                compiler_lexer::tokenize("val b i32\n").flatten().peekable()
            ))
            .unwrap(),
            Statement::Local {
                mutable: false,
                name: Name("b".into(), Some(Type(vec!["i32".into()]))),
                init: None
            }
        );

        // TODO finish tests
    }
}
