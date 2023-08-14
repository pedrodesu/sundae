use std::{iter::Peekable, vec};

use crate::lexer::{Token, TokenType};

#[derive(Debug)]
pub struct Name(String, String);

#[derive(Debug)]
pub enum Expression {
    Literal(String),
    Path(Vec<String>),
    Return(Box<Expression>),
    Binary {
        lhs: String,
        op: String,
        rhs: String,
    },
    Call {
        path: Vec<String>,
        args: Vec<Expression>,
    },
}

impl Expression {
    const PARSE_OPTIONS: &[fn(&mut Peekable<vec::IntoIter<Token>>) -> Option<Self>] = &[
        Self::parse_binary,
        Self::parse_literal,
        Self::parse_call,
        Self::parse_path,
        Self::parse_return,
    ];

    #[inline]
    pub fn parse_any(tokens: &mut Peekable<vec::IntoIter<Token>>) -> Option<Self> {
        Self::PARSE_OPTIONS
            .into_iter()
            .find(|&&f| f(&mut tokens.clone()).is_some())?(tokens)
    }

    #[inline]
    fn parse_literal(tokens: &mut Peekable<vec::IntoIter<Token>>) -> Option<Self> {
        Some(Self::Literal(assert_token(tokens, |t| {
            t.r#type == TokenType::Literal
        })?))
    }

    fn parse_path(tokens: &mut Peekable<vec::IntoIter<Token>>) -> Option<Self> {
        let mut path = Vec::new();

        while path.is_empty() || tokens.next_if(|t| t.value == ".").is_some() {
            let segment = assert_token(tokens, |t| t.r#type == TokenType::Identifier)?;
            path.push(segment);
        }

        Some(Self::Path(path))
    }

    fn parse_return(tokens: &mut Peekable<vec::IntoIter<Token>>) -> Option<Self> {
        assert_token(tokens, |t| t.value == "ret")?;

        let expr = Box::new(Self::parse_any(tokens)?);
        Some(Self::Return(expr))
    }

    fn parse_binary(tokens: &mut Peekable<vec::IntoIter<Token>>) -> Option<Self> {
        // TODO parse_any instead of identifier or literal, binary fn overflow
        let lhs = assert_token(tokens, |t: &Token| {
            matches!(t.r#type, TokenType::Identifier | TokenType::Literal)
        })?;
        let op = assert_token(tokens, |t: &Token| matches!(t.r#type, TokenType::Operator))?;
        let rhs = assert_token(tokens, |t: &Token| {
            matches!(t.r#type, TokenType::Identifier | TokenType::Literal)
        })?;

        Some(Self::Binary { lhs, op, rhs })
    }

    fn parse_call(tokens: &mut Peekable<vec::IntoIter<Token>>) -> Option<Self> {
        let Expression::Path(path) = Self::parse_path(tokens)? else { unreachable!()};

        assert_token(tokens, |t| t.value == "(")?;

        let args = {
            let mut buffer = Vec::new();

            while tokens.peek()?.value != ")" {
                if !buffer.is_empty() {
                    assert_token(tokens, |t| t.value == ",")?;
                }

                let expr = Expression::parse_any(tokens)?;
                buffer.push(expr);
            }

            buffer
        };

        assert_token(tokens, |t| t.value == ")")?;

        Some(Self::Call { path, args })
    }
}

#[derive(Debug)]
pub struct Signature {
    name: (String, Option<String>),
    arguments: Vec<Name>,
}

#[inline]
fn assert_token(
    tokens: &mut Peekable<vec::IntoIter<Token>>,
    predicate: impl Fn(&Token) -> bool,
) -> Option<String> {
    tokens
        .next()
        .and_then(|t| if predicate(&t) { Some(t.value) } else { None })
}

#[derive(Debug)]
pub enum Item {
    Function {
        signature: Signature,
        body: Vec<Expression>,
    },
}

impl Item {
    fn parse_function(tokens: &mut Peekable<vec::IntoIter<Token>>) -> Option<Self> {
        assert_token(tokens, |t| t.value == "let")?;
        let identifier = assert_token(tokens, |t| t.r#type == TokenType::Identifier)?;
        assert_token(tokens, |t| t.value == "=")?;
        assert_token(tokens, |t| t.value == "(")?;
        let arguments = {
            let mut buffer = Vec::new();

            while tokens.peek()?.value != ")" {
                if !buffer.is_empty() {
                    assert_token(tokens, |t| t.value == ",")?;
                }

                let identifier = assert_token(tokens, |t| t.r#type == TokenType::Identifier)?;
                let r#type = assert_token(tokens, |t| t.r#type == TokenType::Identifier)?;
                buffer.push(Name(identifier, r#type));
            }

            buffer
        };
        assert_token(tokens, |t| t.value == ")")?;
        let r#type = tokens
            .next_if(|t| t.r#type == TokenType::Identifier)
            .map(|t| t.value);
        assert_token(tokens, |t| t.value == "{")?;
        let body = {
            let mut buffer = Vec::new();

            while tokens.peek()?.value != "}" {
                buffer.push(Expression::parse_any(tokens)?);
                assert_token(tokens, |t| t.value == ";")?;
            }

            buffer
        };
        assert_token(tokens, |t| t.value == "}")?;

        Some(Self::Function {
            signature: Signature {
                name: (identifier, r#type),
                arguments,
            },
            body,
        })
    }
}

#[inline(always)]
pub fn parse(input: Vec<Token>) -> Vec<Item> {
    let mut iterator = input.into_iter().peekable();
    let mut items = Vec::new();

    while !iterator.is_empty() {
        let x = Item::parse_function(&mut iterator).unwrap();
        items.push(x);
    }

    items
}
