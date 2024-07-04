use compiler_lexer::definitions::{LiteralType, Token, TokenType};

use crate::{iterator::TokenItTrait, statement::Statement, ExhaustiveGet, TokenIt};

use self::binary::BinaryNode;

pub mod binary;

#[derive(Clone, Debug)]
pub enum Expression {
    Literal {
        value: String,
        r#type: LiteralType,
    },
    Path(Vec<String>),
    Binary(BinaryNode),
    Call {
        path: Vec<String>,
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
        Self::parse_literal,
        Self::parse_call,
        Self::parse_path,
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

        while path.is_empty() || tokens.consume(|t| t.value == ".").is_some() {
            let segment = tokens.consume(|t| t.r#type == TokenType::Identifier)?;
            path.push(segment);
        }

        Some(Self::Path(path))
    }

    #[inline]
    pub fn parse_binary<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Option<Self> {
        BinaryNode::parse(tokens).map(Self::Binary)
    }

    pub fn parse_call<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Option<Self> {
        let Self::Path(path) = Self::parse_path(tokens)? else {
            unreachable!()
        };

        let args = tokens.parse_generic_list("(", ")", |t| Self::get(t), Some(","))?;

        Some(Self::Call { path, args })
    }

    pub fn parse_if<I: TokenItTrait>(tokens: &mut TokenIt<I>) -> Option<Self> {
        tokens.consume(|t| t.value == "if")?;

        tokens.ignore_newlines();

        let condition = Self::get(tokens)?;

        let block = tokens.parse_block()?;

        tokens.ignore_newlines();

        let r#else = tokens
            .consume(|t| t.value == "else")
            .and_then(|_| tokens.parse_block());

        Some(Self::If {
            condition: Box::new(condition),
            block,
            else_block: r#else,
        })
    }
}
