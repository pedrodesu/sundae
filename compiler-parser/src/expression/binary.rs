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
    const TERMS: &'static [Self] = {
        use Operator::*;

        &[
            Sum, Sub, And, Or, Lt, Gt, Le, Ge, EqEq, Neq, Shl, Shr, ShAnd, ShOr, Xor,
        ]
    };

    const FACTORS: &'static [Self] = {
        use Operator::*;

        &[Star, Div]
    };
}

impl TryFrom<&Token> for Operator {
    type Error = ();

    #[inline]
    fn try_from(token: &Token) -> Result<Self, Self::Error> {
        use Operator::*;

        match token.value.as_str() {
            "+" => Ok(Sum),
            "-" => Ok(Sub),
            "*" => Ok(Star),
            "/" => Ok(Div),
            "and" => Ok(And),
            "or" => Ok(Or),
            "<" => Ok(Lt),
            ">" => Ok(Gt),
            "<=" => Ok(Le),
            ">=" => Ok(Ge),
            "==" => Ok(EqEq),
            "!=" => Ok(Neq),
            "<<" => Ok(Shl),
            ">>" => Ok(Shr),
            "&" => Ok(ShAnd),
            "|" => Ok(ShOr),
            "^" => Ok(Xor),
            _ => Err(()),
        }
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
        while let Some(t) = tokens.0.next_if(|t| {
            Operator::try_from(t)
                .ok()
                .is_some_and(|op| Operator::FACTORS.contains(&op))
        }) {
            let next = Self::term(tokens)?;
            acc = Self::Compound(Box::new(acc), Operator::try_from(&t).ok()?, Box::new(next));
        }
        Some(acc)
    }

    fn consume(tokens: &mut TokenIt) -> Option<Self> {
        let mut acc = Self::factor(tokens)?;
        // TODO manage whitespace between binary (and multiple other instances, such as assign)
        while let Some(t) = tokens.0.next_if(|t| {
            Operator::try_from(t)
                .ok()
                .is_some_and(|op| Operator::TERMS.contains(&op))
        }) {
            let next = Self::factor(tokens)?;
            acc = Self::Compound(Box::new(acc), Operator::try_from(&t).ok()?, Box::new(next));
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
