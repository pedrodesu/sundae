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

        tokens.ignore_newlines();

        let condition = Self::get(tokens)?;

        let block = tokens.parse_block()?;

        // FIXME needs to ignore newlines here

        let r#else = tokens
            .next(|t| t.value == "else")
            .and_then(|_| tokens.parse_block());

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

// TODO impl remaining tests
