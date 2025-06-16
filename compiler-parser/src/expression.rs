use compiler_lexer::definitions::{LiteralType, Token, TokenType};
use ecow::{EcoString, EcoVec};
use operator::{Operator, to_operator};

use crate::{
    ParserError, TokenIt,
    iterator::{ExhaustiveGet, TokenItTrait},
    statement::Statement,
};

pub mod binary;
pub mod operator;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression
{
    Literal
    {
        value: EcoString,
        r#type: LiteralType,
    },
    Path(EcoVec<EcoString>),
    Binary(Box<binary::Node>),
    Unary(Operator, Box<Expression>),
    Call
    {
        path: EcoVec<EcoString>,
        args: EcoVec<Expression>,
    },
    If
    {
        condition: Box<Expression>,
        block: EcoVec<Statement>,
        else_block: Option<EcoVec<Statement>>,
    },
    Parenthesis(Box<Expression>),
    Tuple(EcoVec<Expression>),
    Array(EcoVec<Expression>),
    // TODO Block
}

impl<I: TokenItTrait> ExhaustiveGet<I> for Expression
{
    fn find_predicate(tokens: &mut TokenIt<I>) -> Result<Self::ParsePredicate, ParserError>
    {
        let base_predicate = Self::shallow_find_predicate(&mut tokens.clone())?;

        base_predicate(tokens)?; // Consume whichever base so we can peek ahead

        if tokens
            .0
            .peek()
            .is_some_and(|t| t.r#type == TokenType::Operator)
        {
            Ok(Self::parse_binary)
        }
        else
        {
            Ok(base_predicate)
        }
    }
}

impl Expression
{
    pub fn shallow_find_predicate<I: TokenItTrait>(
        tokens: &mut TokenIt<I>,
    ) -> Result<<Expression as ExhaustiveGet<I>>::ParsePredicate, ParserError>
    {
        if tokens.0.peek().is_some_and(|t| t.value == "if")
        {
            Ok(Self::parse_if)
        }
        else if tokens
            .0
            .peek()
            .is_some_and(|t| t.r#type == TokenType::Operator)
        {
            Ok(Self::parse_unary)
        }
        else if tokens
            .0
            .peek()
            .is_some_and(|t| matches!(t.r#type, TokenType::Literal(_)))
        {
            Ok(Self::parse_literal)
        }
        else if tokens.0.peek().is_some_and(|t| t.value == "[")
        {
            Ok(Self::parse_array)
        }
        else
        {
            {
                let mut tokens = tokens.clone();

                if tokens.next(|t| t.value == "(").is_some() && Expression::get(&mut tokens).is_ok()
                {
                    // TODO this won't work properly with a leading colon, as probably other things won't either. make a decision on this
                    if tokens.0.peek().is_some_and(|t| t.value == ")")
                    {
                        return Ok(Self::parse_parenthesis);
                    }
                    else
                    {
                        return Ok(Self::parse_tuple);
                    }
                }
            }

            {
                let mut tokens = tokens.clone();

                if Self::parse_path(&mut tokens).is_ok()
                {
                    if tokens.0.peek().is_some_and(|t| t.value == "(")
                    {
                        return Ok(Self::parse_call);
                    }
                    else
                    {
                        return Ok(Self::parse_path);
                    }
                }
            }

            return Err(ParserError::ExpectedASTStructure { name: "Expression" });
        }
    }

    #[inline]
    pub fn parse_literal(tokens: &mut TokenIt<impl TokenItTrait>) -> Result<Self, ParserError>
    {
        let Token {
            value,
            r#type: TokenType::Literal(lit_type),
            ..
        } = tokens
            .next(|t| matches!(t.r#type, TokenType::Literal(_)))
            .ok_or(ParserError::ExpectedTokenType { r#type: "Literal" })?
        else
        {
            unreachable!()
        };

        Ok(Self::Literal {
            value,
            r#type: lit_type,
        })
    }

    pub fn parse_path(tokens: &mut TokenIt<impl TokenItTrait>) -> Result<Self, ParserError>
    {
        let mut path = EcoVec::new();

        while path.is_empty() || tokens.next(|t| t.value == ".").is_some()
        {
            let segment = tokens
                .next(|t| t.r#type == TokenType::Identifier)
                .ok_or(ParserError::ExpectedTokenType {
                    r#type: "Identifier",
                })?
                .value;
            path.push(segment);
        }

        Ok(Self::Path(path))
    }

    #[inline]
    pub fn parse_binary(tokens: &mut TokenIt<impl TokenItTrait>) -> Result<Self, ParserError>
    {
        // TODO RPN should prolly be bettered.
        let node = binary::Node::parse(tokens)?;

        Ok(Self::Binary(Box::new(node)))
    }

    pub fn parse_call(tokens: &mut TokenIt<impl TokenItTrait>) -> Result<Self, ParserError>
    {
        let Self::Path(path) = Self::parse_path(tokens)?
        else
        {
            unreachable!()
        };

        let args = tokens.consume_generic_list(("(", ")"), Expression::get, Some(","))?;

        Ok(Self::Call { path, args })
    }

    pub fn parse_if(tokens: &mut TokenIt<impl TokenItTrait>) -> Result<Self, ParserError>
    {
        tokens
            .next(|t| t.value == "if")
            .ok_or(ParserError::ExpectedTokenValue { value: "if".into() })?;

        // TODO ignore_newlines might not be necessary? if when we get next we always skip newline. is this viable? try and test.
        tokens.ignore_newlines();

        let condition = Expression::get(tokens)?;

        tokens.ignore_newlines();

        let block = tokens.consume_block()?;

        tokens.ignore_newlines();

        let r#else = if tokens.next(|t| t.value == "else").is_some()
        {
            tokens.ignore_newlines();

            Some(tokens.consume_block()?)
        }
        else
        {
            None
        };

        Ok(Self::If {
            condition: Box::new(condition),
            block,
            else_block: r#else,
        })
    }

    pub fn parse_unary(tokens: &mut TokenIt<impl TokenItTrait>) -> Result<Self, ParserError>
    {
        let operator = tokens
            .next(|t| t.r#type == TokenType::Operator)
            .ok_or(ParserError::ExpectedTokenType { r#type: "Operator" })?;
        let operator @ (Operator::Minus | Operator::Star) = to_operator(&operator)
        else
        {
            return Err(ParserError::IllegalUnary { token: operator });
        };

        let e = Expression::get(tokens)?;

        Ok(Self::Unary(operator, Box::new(e)))
        // // Custom order, immediate literal should have priority over another binary expression
        // let expression = [
        //     Self::parse_if,
        //     Self::parse_unary,
        //     Self::parse_call,
        //     Self::parse_path,
        //     Self::parse_literal,
        //     Self::parse_binary,
        // ]
        // .into_iter()
        // // this is ass
        // // also this sort of brute force testing is also used on expression/binary.rs, make this better
        // .find(|&f| matches!(f(&mut tokens.clone()), Some(Ok(_))))?(tokens)
        // .unwrap()
        // .unwrap();

        // Some(Ok(Self::Unary(operator, Box::new(expression))))
    }

    pub fn parse_parenthesis(tokens: &mut TokenIt<impl TokenItTrait>) -> Result<Self, ParserError>
    {
        tokens
            .next(|t| t.value == "(")
            .ok_or(ParserError::ExpectedTokenValue { value: "(".into() })?;

        let e = Expression::get(tokens)?;

        tokens
            .next(|t| t.value == ")")
            .ok_or(ParserError::ExpectedTokenValue { value: ")".into() })?;

        Ok(Self::Parenthesis(Box::new(e)))
    }

    #[inline]
    pub fn parse_tuple(tokens: &mut TokenIt<impl TokenItTrait>) -> Result<Self, ParserError>
    {
        Ok(Self::Tuple(tokens.consume_generic_list(
            ("(", ")"),
            Expression::get,
            Some(","),
        )?))
    }

    #[inline]
    pub fn parse_array(tokens: &mut TokenIt<impl TokenItTrait>) -> Result<Self, ParserError>
    {
        Ok(Self::Array(tokens.consume_generic_list(
            ("[", "]"),
            Expression::get,
            Some(","),
        )?))
    }
}

#[cfg(test)]
mod tests
{
    use compiler_lexer::definitions::Span;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::Node;

    // Expression::Literal and Expression::Binary are mere simple wrappers for already tested features, so we don't test them here

    #[test]
    fn path_passes()
    {
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
    fn call_passes()
    {
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

    #[test]
    fn if_passes()
    {
        assert_eq!(
            Expression::parse_if(&mut TokenIt(
                compiler_lexer::tokenize("if 1 {}").flatten().peekable()
            )),
            Ok(Expression::If {
                condition: Box::new(Expression::Literal {
                    value: "1".into(),
                    r#type: LiteralType::Int
                }),
                block: vec![].into(),
                else_block: None
            })
        );

        assert_eq!(
            Expression::parse_if(&mut TokenIt(
                compiler_lexer::tokenize("if 1 {} else {}")
                    .flatten()
                    .peekable()
            )),
            Ok(Expression::If {
                condition: Box::new(Expression::Literal {
                    value: "1".into(),
                    r#type: LiteralType::Int
                }),
                block: vec![].into(),
                else_block: Some(vec![].into())
            })
        );

        assert_eq!(
            Expression::parse_if(&mut TokenIt(
                compiler_lexer::tokenize("if 2 + 2 {\ncall()\n}")
                    .flatten()
                    .peekable()
            )),
            Ok(Expression::If {
                condition: Box::new(Expression::Binary(Box::new(Node::Compound(Box::new((
                    Node::Scalar(Expression::Literal {
                        value: "2".into(),
                        r#type: LiteralType::Int
                    }),
                    Operator::Plus,
                    Node::Scalar(Expression::Literal {
                        value: "2".into(),
                        r#type: LiteralType::Int
                    })
                )))))),
                block: vec![Statement::Expression(Expression::Call {
                    path: vec!["call".into()].into(),
                    args: vec![].into()
                })]
                .into(),
                else_block: None
            })
        );

        assert_eq!(
            Expression::parse_if(&mut TokenIt(
                compiler_lexer::tokenize("if 1 {call() }")
                    .flatten()
                    .peekable()
            )),
            Ok(Expression::If {
                condition: Box::new(Expression::Literal {
                    value: "1".into(),
                    r#type: LiteralType::Int
                }),
                block: vec![Statement::Expression(Expression::Call {
                    path: vec!["call".into()].into(),
                    args: vec![].into()
                })]
                .into(),
                else_block: None
            })
        );

        assert_eq!(
            Expression::parse_if(&mut TokenIt(
                compiler_lexer::tokenize("if 1{ call() }else { other_call()}")
                    .flatten()
                    .peekable()
            )),
            Ok(Expression::If {
                condition: Box::new(Expression::Literal {
                    value: "1".into(),
                    r#type: LiteralType::Int
                }),
                block: vec![Statement::Expression(Expression::Call {
                    path: vec!["call".into()].into(),
                    args: vec![].into()
                })]
                .into(),
                else_block: Some(
                    vec![Statement::Expression(Expression::Call {
                        path: vec!["other_call".into()].into(),
                        args: vec![].into()
                    })]
                    .into()
                )
            })
        );

        assert_eq!(
            Expression::parse_if(&mut TokenIt(
                compiler_lexer::tokenize(
                    "if\n\n2 + 2\n\n{\n   \n  call  ()\n}\n\t  \nelse\n  \n\n{\n\n42\n\n\n}\n\n"
                )
                .flatten()
                .peekable()
            )),
            Ok(Expression::If {
                condition: Box::new(Expression::Binary(Box::new(Node::Compound(Box::new((
                    Node::Scalar(Expression::Literal {
                        value: "2".into(),
                        r#type: LiteralType::Int
                    }),
                    Operator::Plus,
                    Node::Scalar(Expression::Literal {
                        value: "2".into(),
                        r#type: LiteralType::Int
                    }),
                )))))),
                block: vec![Statement::Expression(Expression::Call {
                    path: vec!["call".into()].into(),
                    args: vec![].into()
                })]
                .into(),
                else_block: Some(
                    vec![Statement::Expression(Expression::Literal {
                        value: "42".into(),
                        r#type: LiteralType::Int
                    })]
                    .into()
                )
            })
        );
    }

    #[test]
    fn unary_passes()
    {
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
