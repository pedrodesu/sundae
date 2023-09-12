use std::{fmt::Debug, iter::Peekable, vec};

use itertools::Itertools;

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
fn ignore_newlines(tokens: Tokens) {
    tokens
        .peeking_take_while(|t| t.r#type == TokenType::Newline)
        .for_each(drop)
}

fn parse_generic_list<T>(
    tokens: Tokens,
    left_bound: &str,
    right_bound: &str,
    predicate: impl Fn(Tokens) -> Option<T>,
    sep_predicate: Option<&str>,
) -> Option<Vec<T>> {
    ignore_newlines(tokens);

    assert_token(tokens, |t| t.value == left_bound)?;

    let mut buffer = Vec::new();

    loop {
        ignore_newlines(tokens);

        if tokens.next_if(|t| t.value == right_bound).is_some() {
            break;
        }

        if let Some(sep_predicate) = sep_predicate {
            if !buffer.is_empty() {
                assert_token(tokens, |t| t.value == sep_predicate)?;
            }
        }

        let value = predicate(tokens)?;
        buffer.push(value);
    }

    Some(buffer)
}

#[inline]
fn parse_block(tokens: Tokens) -> Option<Vec<Statement>> {
    parse_generic_list(
        tokens,
        "{",
        "}",
        |t| test_any(Statement::PARSE_OPTIONS, t),
        None,
    )
}

#[inline]
pub fn test_any<A, B, C>(it: A, tokens: &mut Peekable<C>) -> Option<B>
where
    A: IntoIterator,
    A::Item: Fn(&mut Peekable<C>) -> Option<B>,
    C: Iterator<Item = Token> + Clone,
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
        if let Token {
            value,
            r#type: TokenType::Literal(lit_type),
        } = tokens.next()?
        {
            Some(Self::Literal(value, lit_type))
        } else {
            None
        }
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

        let args = parse_generic_list(
            tokens,
            "(",
            ")",
            |t| test_any(Expression::PARSE_OPTIONS, t),
            Some(","),
        )?;

        Some(Self::Call { path, args })
    }

    fn parse_if(tokens: Tokens) -> Option<Self> {
        assert_token(tokens, |t| t.value == "if")?;

        let condition = test_any(Expression::PARSE_OPTIONS, tokens)?;

        let block = parse_block(tokens)?;

        let r#else = if tokens.next_if(|t| t.value == "else").is_some() {
            parse_block(tokens)
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

        let expr = if tokens.peek()?.r#type != TokenType::Newline {
            Some(test_any(Expression::PARSE_OPTIONS, tokens)?)
        } else {
            None
        };

        assert_token(tokens, |t| t.r#type == TokenType::Newline)?;

        Some(Self::Return(expr))
    }

    #[inline]
    fn parse_expression(tokens: Tokens) -> Option<Self> {
        let expr = test_any(Expression::PARSE_OPTIONS, tokens)?;

        assert_token(tokens, |t| t.r#type == TokenType::Newline)?;

        Some(Self::Expression(expr))
    }
}

#[derive(Debug)]
pub enum Item {
    Const {
        name: Name,
        value: Expression,
    },
    Function {
        signature: Signature,
        body: Vec<Statement>,
    },
}

impl Item {
    const PARSE_OPTIONS: &'static [fn(Tokens) -> Option<Self>] =
        &[Self::parse_const, Self::parse_function];

    fn parse_const(tokens: Tokens) -> Option<Self> {
        assert_token(tokens, |t| t.value == "const")?;

        let identifier = assert_token(tokens, |t| t.r#type == TokenType::Identifier)?;

        let r#type = {
            if let Some(r#type) = tokens
                .next_if(|t| t.r#type == TokenType::Identifier)
                .map(|t| t.value)
            {
                r#type
            } else {
                assert_token(tokens, |t| t.value == "[")?;
                let size =
                    assert_token(tokens, |t| t.r#type == TokenType::Literal(LiteralType::Int))?;
                assert_token(tokens, |t| t.value == "]")?;
                let base_type: String =
                    assert_token(tokens, |t| t.r#type == TokenType::Identifier)?;

                format!("[{size}]{base_type}")
            }
        };

        assert_token(tokens, |t| t.value == "=")?;

        let value = test_any(Expression::PARSE_OPTIONS, tokens)?;

        assert_token(tokens, |t| t.r#type == TokenType::Newline)?;

        Some(Self::Const {
            name: Name(identifier, r#type),
            value,
        })
    }

    fn parse_function(tokens: Tokens) -> Option<Self> {
        assert_token(tokens, |t| t.value == "func")?;

        let identifier = assert_token(tokens, |t| t.r#type == TokenType::Identifier)?;

        let arguments = parse_generic_list(
            tokens,
            "(",
            ")",
            |t| {
                let identifier = assert_token(t, |t| t.r#type == TokenType::Identifier)?;
                let r#type = assert_token(t, |t| t.r#type == TokenType::Identifier)?;

                Some(Name(identifier, r#type))
            },
            Some(","),
        )?;

        let r#type = tokens
            .next_if(|t| t.r#type == TokenType::Identifier)
            .map(|t| t.value);

        let body = parse_block(tokens)?;

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
        if iterator
            .next_if(|t| t.r#type == TokenType::Newline)
            .is_none()
        {
            items.push(test_any(Item::PARSE_OPTIONS, &mut iterator).unwrap());
        }
    }

    AST(items)
}

// TODO refine type extracting and definition
