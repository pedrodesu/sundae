use compiler_lexer::definitions::{Token, TokenType};
use itertools::Itertools;

use crate::{
    Name, ParserError, TokenIt, Type,
    expression::Expression,
    iterator::{ExhaustiveGet, TokenItTrait},
};

#[derive(Clone, Debug, PartialEq)]
pub enum Statement
{
    Return(Option<Expression>),
    Expression(Expression),
    // TODO add another field here. identify and refactor operator assign exprs such as +=
    Assign
    {
        destination: Expression,
        source: Expression,
    },
    Local
    {
        mutable: bool,
        name: Name,
        init: Option<Expression>,
    },
}

impl<I: TokenItTrait> ExhaustiveGet<I> for Statement
{
    fn find_predicate(tokens: &mut TokenIt<I>) -> Result<Self::ParsePredicate, ParserError>
    {
        if tokens.0.peek().is_some_and(|t| t.value == "ret")
        {
            Ok(Self::parse_return)
        }
        else if tokens.0.peek().is_some_and(|t| t.value == "let")
        {
            Ok(Self::parse_local)
        }
        else
        {
            {
                let mut tokens = tokens.clone();
                if Expression::get(&mut tokens).is_ok()
                {
                    if tokens.next(|t| t.value == "=").is_some()
                    {
                        return Ok(Self::parse_assign);
                    }
                    else
                    {
                        return Ok(Self::parse_expression);
                    }
                }
            }

            return Err(ParserError::ExpectedASTStructure { name: "Statement" });
        }
    }
}

// TODO implement concrete error types for everything that may fuck up
// also make error handling more proper with line and col and what not

impl Statement
{
    #[inline]
    fn assert_end<I: TokenItTrait>(
        tokens: &mut TokenIt<I>,
        predicate: impl FnOnce(&mut TokenIt<I>) -> Result<Self, ParserError>,
    ) -> Result<Self, ParserError>
    {
        let value = predicate(tokens)?;

        if let Some(Token {
            r#type: TokenType::Newline, // TokenType::Separator | TokenType::Newline, // shouldn't we only allow newline here?
            ..
        }) = tokens.0.peek()
        {
            Ok(value)
        }
        else
        {
            Err(ParserError::ExpectedNewline)
        }
    }

    #[inline]
    pub fn parse_return<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Result<Self, ParserError>
    {
        Self::assert_end(tokens, |tokens| {
            tokens
                .next(|t| t.value == "ret")
                .ok_or(ParserError::ExpectedTokenValue {
                    value: "ret".into(),
                })?;

            if let Some(Token {
                r#type: TokenType::Newline,
                ..
            }) = tokens.0.peek()
            {
                Ok(Self::Return(None))
            }
            else
            {
                let e = Expression::get(tokens)?;

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
    pub fn parse_expression<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Result<Self, ParserError>
    {
        Self::assert_end(tokens, |tokens| {
            Ok(Self::Expression(Expression::get(tokens)?))
        })
    }

    pub fn parse_assign<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Result<Self, ParserError>
    {
        Self::assert_end(tokens, |tokens| {
            let destination = Expression::get(tokens)?;

            tokens
                .next(|t| t.value == "=")
                .ok_or(ParserError::ExpectedTokenValue { value: "=".into() })?;

            let source = Expression::get(tokens)?;

            Ok(Self::Assign {
                destination,
                source,
            })
        })
    }

    pub fn parse_local<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Result<Self, ParserError>
    {
        Self::assert_end(tokens, |tokens| {
            tokens
                .next(|t| t.value == "let")
                .ok_or(ParserError::ExpectedTokenValue {
                    value: "let".into(),
                })?;

            let identifier = tokens
                .next(|t| t.r#type == TokenType::Identifier)
                .ok_or(ParserError::ExpectedTokenType {
                    r#type: "Identifier",
                })?
                .value;

            let mutable = tokens.next(|t| t.value == "mut").is_some();

            // shouldn't mut always only be intrinsic to the type?
            // No. a variable can be mutable. a type does not have this qualification. a pointer, however, may or may not be mutable.

            let r#type = {
                let r#type = tokens
                    .0
                    .peeking_take_while(|t| t.value != "=" && t.r#type != TokenType::Newline)
                    .map(|t| t.value)
                    .collect::<Vec<_>>();

                if r#type.is_empty()
                {
                    None
                }
                else
                {
                    Some(Type(r#type))
                }
            };

            let init = if tokens.next(|t| t.value == "=").is_some()
            {
                Some(Expression::get(tokens)?)
            }
            else
            {
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
mod tests
{
    use compiler_lexer::definitions::LiteralType;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::Operator;

    // Statement::parse_expression is just a struct wrapper for an already tested function, so we don't test it here

    #[test]
    fn return_passes()
    {
        assert_eq!(
            Statement::parse_return(&mut TokenIt(
                compiler_lexer::tokenize("ret \n").flatten().peekable()
            )),
            Ok(Statement::Return(None))
        );

        assert_eq!(
            Statement::parse_return(&mut TokenIt(
                compiler_lexer::tokenize("ret 42\n").flatten().peekable()
            )),
            Ok(Statement::Return(Some(Expression::Literal {
                value: "42".into(),
                r#type: LiteralType::Int
            })))
        );

        assert_eq!(
            Statement::parse_return(&mut TokenIt(
                compiler_lexer::tokenize("ret ret\n\n").flatten().peekable()
            )),
            Err(ParserError::ExpectedASTStructure { name: "Expression" })
        );
    }

    #[test]
    fn assign_passes()
    {
        assert_eq!(
            Statement::parse_assign(&mut TokenIt(
                // Can lhs of an assign expression ever be something else other than a path?
                compiler_lexer::tokenize("a = 2\n").flatten().peekable()
            )),
            Ok(Statement::Assign {
                destination: Expression::Path(vec!["a".into()].into()),
                source: Expression::Literal {
                    value: "2".into(),
                    r#type: LiteralType::Int
                }
            })
        );

        assert_eq!(
            Statement::parse_assign(&mut TokenIt(
                compiler_lexer::tokenize("*func_to_ptr() = 42\n")
                    .flatten()
                    .peekable()
            )),
            Ok(Statement::Assign {
                destination: Expression::Unary(
                    Operator::Star,
                    Box::new(Expression::Call {
                        path: vec!["func_to_ptr".into()].into(),
                        args: vec![].into()
                    })
                ),
                source: Expression::Literal {
                    value: "42".into(),
                    r#type: LiteralType::Int
                }
            })
        );
    }

    #[test]
    fn local_passes()
    {
        assert_eq!(
            Statement::parse_local(&mut TokenIt(
                compiler_lexer::tokenize("let v\n").flatten().peekable()
            )),
            Ok(Statement::Local {
                mutable: false,
                name: Name("v".into(), None),
                init: None,
            })
        );

        assert_eq!(
            Statement::parse_local(&mut TokenIt(
                compiler_lexer::tokenize("let a = 2\n").flatten().peekable()
            )),
            Ok(Statement::Local {
                mutable: false,
                name: Name("a".into(), None),
                init: Some(Expression::Literal {
                    value: "2".into(),
                    r#type: LiteralType::Int
                })
            })
        );

        assert_eq!(
            Statement::parse_local(&mut TokenIt(
                compiler_lexer::tokenize("let b i32 = 4\n")
                    .flatten()
                    .peekable()
            )),
            Ok(Statement::Local {
                mutable: false,
                name: Name("b".into(), Some(Type(vec!["i32".into()]))),
                init: Some(Expression::Literal {
                    value: "4".into(),
                    r#type: LiteralType::Int
                })
            })
        );

        assert_eq!(
            Statement::parse_local(&mut TokenIt(
                compiler_lexer::tokenize("let b i32\n").flatten().peekable()
            )),
            Ok(Statement::Local {
                mutable: false,
                name: Name("b".into(), Some(Type(vec!["i32".into()]))),
                init: None
            })
        );

        assert_eq!(
            Statement::parse_local(&mut TokenIt(
                compiler_lexer::tokenize("let c *i32\n")
                    .flatten()
                    .peekable()
            )),
            Ok(Statement::Local {
                mutable: false,
                name: Name("c".into(), Some(Type(vec!["*".into(), "i32".into()]))),
                init: None
            })
        );

        assert_eq!(
            Statement::parse_local(&mut TokenIt(
                compiler_lexer::tokenize("let s []i32\n")
                    .flatten()
                    .peekable()
            )),
            Ok(Statement::Local {
                mutable: false,
                name: Name(
                    "s".into(),
                    Some(Type(vec!["[".into(), "]".into(), "i32".into()]))
                ),
                init: None
            })
        );

        // TODO finish tests
    }
}
