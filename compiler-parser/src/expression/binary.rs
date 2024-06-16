use compiler_lexer::definitions::Token;

use crate::{ExhaustiveGet, TokenIt};

use super::Expression;

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
    const TERMS: &'static [Operator] = {
        &[
            Self::Sum,
            Self::Sub,
            Self::And,
            Self::Or,
            Self::Lt,
            Self::Gt,
            Self::Le,
            Self::Ge,
            Self::EqEq,
            Self::Neq,
            Self::Shl,
            Self::Shr,
            Self::ShAnd,
            Self::ShOr,
            Self::Xor,
        ]
    };

    const FACTORS: &'static [Operator] = { &[Self::Star, Self::Div] };
}

#[inline]
fn from_string(token: &Token) -> Option<Operator> {
    use Operator::*;

    match token.value.as_str() {
        "+" => Some(Sum),
        "-" => Some(Sub),
        "*" => Some(Star),
        "/" => Some(Div),
        "and" => Some(And),
        "or" => Some(Or),
        "<" => Some(Lt),
        ">" => Some(Gt),
        "<=" => Some(Le),
        ">=" => Some(Ge),
        "==" => Some(EqEq),
        "!=" => Some(Neq),
        "<<" => Some(Shl),
        ">>" => Some(Shr),
        "&" => Some(ShAnd),
        "|" => Some(ShOr),
        "^" => Some(Xor),
        _ => None,
    }
}

#[derive(Clone, Debug)]
pub enum BinaryNode {
    Scalar(Box<Expression>),
    Compound(Box<BinaryNode>, Operator, Box<BinaryNode>),
}

impl BinaryNode {
    fn term(tokens: &mut TokenIt) -> Option<Self> {
        let other_predicates = Expression::PARSE_OPTIONS
            .into_iter()
            .filter(|&&f| f != Expression::parse_binary);

        let test = other_predicates
            .into_iter()
            .find(|f| f(&mut tokens.clone()).is_some())?(tokens);

        if let Some(expr) = test {
            Some(Self::Scalar(Box::new(expr)))
        } else if matches!(tokens.0.next()?, Token { value, .. } if value == "(") {
            let exp = Self::consume(tokens);
            if matches!(tokens.0.next(), Some(Token { value, .. }) if value == ")") {
                exp
            } else {
                None
            }
        } else {
            None
        }
    }

    fn factor(tokens: &mut TokenIt) -> Option<Self> {
        let mut acc = Self::term(tokens)?;
        while let Some(t) = tokens
            .0
            .next_if(|t| from_string(t).is_some_and(|op| Operator::FACTORS.contains(&op)))
        {
            let next = Self::term(tokens)?;
            acc = Self::Compound(Box::new(acc), from_string(&t)?, Box::new(next));
        }
        Some(acc)
    }

    fn consume(tokens: &mut TokenIt) -> Option<Self> {
        let mut acc = Self::factor(tokens)?;
        while let Some(t) = tokens
            .0
            .next_if(|t| from_string(t).is_some_and(|op| Operator::TERMS.contains(&op)))
        {
            let next = Self::factor(tokens)?;
            acc = Self::Compound(Box::new(acc), from_string(&t)?, Box::new(next));
        }
        if let Self::Compound(..) = acc {
            Some(acc)
        } else {
            None
        }
    }

    #[inline]
    pub fn parse(tokens: &mut TokenIt) -> Option<Self> {
        Self::consume(tokens)
    }
}
