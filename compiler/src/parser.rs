use std::{iter::Peekable, vec};

use crate::lexer::{LiteralType, Token, TokenType};

pub mod binary;

#[derive(Debug)]
pub struct AST(pub Vec<Item>);

#[derive(Debug)]
pub struct Name(pub String, pub String);

#[derive(Clone, Debug)]
pub enum Expression {
    Literal(String, LiteralType),
    Path(Vec<String>),
    Binary(binary::Node),
    Call {
        path: Vec<String>,
        args: Vec<Expression>,
    },
    If {
        condition: Box<Expression>,
        block: Vec<Statement>,
        r#else: Option<Vec<Statement>>,
    },
}

type Tokens<'a> = &'a mut Peekable<vec::IntoIter<Token>>;

#[inline]
pub fn test_any<A, B, C>(it: A, tokens: &mut C) -> Option<B>
where
    A: IntoIterator,
    A::Item: Fn(&mut C) -> Option<B>,
    C: IntoIterator<Item = Token> + Clone,
{
    it.into_iter().find(|f| f(&mut tokens.clone()).is_some())?(tokens)
}

impl Expression {
    pub const PARSE_OPTIONS: &'static [fn(Tokens) -> Option<Self>] = &[
        Self::parse_if,
        Self::parse_binary,
        Self::parse_literal,
        Self::parse_call,
        Self::parse_path,
    ];

    #[inline]
    fn parse_literal(tokens: Tokens) -> Option<Self> {
        let (lit, lit_type) = tokens.next().and_then(|t| {
            if let TokenType::Literal(lit_type) = t.r#type {
                Some((t.value, lit_type))
            } else {
                None
            }
        })?;
        Some(Self::Literal(lit, lit_type))
    }

    fn parse_path(tokens: Tokens) -> Option<Self> {
        let mut path = Vec::new();

        while path.is_empty() || tokens.next_if(|t| t.value == ".").is_some() {
            let segment = assert_token(tokens, |t| t.r#type == TokenType::Identifier)?;
            path.push(segment);
        }

        Some(Self::Path(path))
    }

    #[inline]
    fn parse_binary(tokens: Tokens) -> Option<Self> {
        Some(Self::Binary(binary::parse(tokens)?))
    }

    fn parse_call(tokens: Tokens) -> Option<Self> {
        let Expression::Path(path) = Self::parse_path(tokens)? else {
            unreachable!()
        };

        assert_token(tokens, |t| t.value == "(")?;

        let args = {
            let mut buffer = Vec::new();

            while tokens.peek()?.value != ")" {
                if !buffer.is_empty() {
                    assert_token(tokens, |t| t.value == ",")?;
                }

                let expr = test_any(Expression::PARSE_OPTIONS, tokens)?;
                buffer.push(expr);
            }

            buffer
        };

        assert_token(tokens, |t| t.value == ")")?;

        Some(Self::Call { path, args })
    }

    fn parse_if(tokens: Tokens) -> Option<Self> {
        assert_token(tokens, |t| t.value == "if")?;

        let condition = test_any(Expression::PARSE_OPTIONS, tokens)?;

        assert_token(tokens, |t| t.value == "{")?;

        let block = {
            let mut buffer = Vec::new();

            while tokens.peek()?.value != "}" {
                let stmt = test_any(Statement::PARSE_OPTIONS, tokens)?;
                buffer.push(stmt);
            }

            buffer
        };

        assert_token(tokens, |t| t.value == "}")?;

        let r#else = if tokens.next_if(|t| t.value == "else").is_some() {
            assert_token(tokens, |t| t.value == "{")?;

            let block = {
                let mut buffer = Vec::new();

                while tokens.peek()?.value != "}" {
                    let stmt = test_any(Statement::PARSE_OPTIONS, tokens)?;
                    buffer.push(stmt);
                }

                buffer
            };

            assert_token(tokens, |t| t.value == "}")?;

            Some(block)
        } else {
            None
        };

        Some(Self::If {
            condition: Box::new(condition),
            block,
            r#else,
        })
    }
}

#[derive(Debug)]
pub struct Signature {
    pub name: (String, Option<String>),
    pub arguments: Vec<Name>,
}

#[inline]
fn assert_token(tokens: Tokens, predicate: impl Fn(&Token) -> bool) -> Option<String> {
    tokens
        .next()
        .and_then(|t| if predicate(&t) { Some(t.value) } else { None })
}

#[derive(Debug, Clone)]
pub enum Statement {
    Return(Option<Expression>),
    Expression(Expression),
}

impl Statement {
    const PARSE_OPTIONS: &'static [fn(Tokens) -> Option<Self>] =
        &[Self::parse_return, Self::parse_expression];

    fn parse_return(tokens: Tokens) -> Option<Self> {
        assert_token(tokens, |t| t.value == "ret")?;

        let expr = if tokens.peek()?.value != ";" {
            Some(test_any(Expression::PARSE_OPTIONS, tokens)?)
        } else {
            None
        };

        assert_token(tokens, |t| t.value == ";")?;

        Some(Self::Return(expr))
    }

    #[inline]
    fn parse_expression(tokens: Tokens) -> Option<Self> {
        let expr = test_any(Expression::PARSE_OPTIONS, tokens)?;

        assert_token(tokens, |t| t.value == ";")?;

        Some(Self::Expression(expr))
    }
}

#[derive(Debug)]
pub enum Item {
    Function {
        signature: Signature,
        body: Vec<Statement>,
    },
}

impl Item {
    fn parse_function(tokens: Tokens) -> Option<Self> {
        assert_token(tokens, |t| t.value == "func")?;
        let identifier = assert_token(tokens, |t| t.r#type == TokenType::Identifier)?;
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
                buffer.push(test_any(Statement::PARSE_OPTIONS, tokens)?);
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
pub fn parse(input: Vec<Token>) -> AST {
    let mut iterator = input.into_iter().peekable();
    let mut items = Vec::new();

    while iterator.peek().is_some() {
        let x = Item::parse_function(&mut iterator).unwrap();
        items.push(x);
    }

    AST(items)
}
