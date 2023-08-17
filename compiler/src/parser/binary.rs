use std::{iter::Peekable, vec};

use super::Expression;

const TERMS: &[Operator] = {
    use Operator::*;
    &[
        Sum, Sub, And, Or, Lt, Gt, Le, Ge, EqEq, Neq, Shl, Shr, ShAnd, ShOr, Xor,
    ]
};

const FACTORS: &[Operator] = {
    use Operator::*;
    &[Star, Div]
};

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
    #[inline]
    fn is_term(&self) -> bool {
        TERMS.contains(self)
    }

    #[inline]
    fn is_factor(&self) -> bool {
        FACTORS.contains(self)
    }
}

impl TryFrom<&super::Token> for Operator {
    type Error = ();

    #[inline]
    fn try_from(token: &super::Token) -> Result<Self, Self::Error> {
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
pub enum Node {
    Scalar(Box<Expression>),
    Compound(Box<Node>, Operator, Box<Node>),
}

struct Parser<'a> {
    it: &'a mut Peekable<vec::IntoIter<super::Token>>,
}

impl<'a> From<&'a mut Peekable<vec::IntoIter<super::Token>>> for Parser<'a> {
    #[inline]
    fn from(it: &'a mut Peekable<vec::IntoIter<super::Token>>) -> Self {
        Self { it }
    }
}

impl Parser<'_> {
    pub fn parse_exp(&mut self) -> Option<Node> {
        let mut acc = self.factor()?;
        while let Some(t) = self
            .it
            .next_if(|t| Operator::try_from(t).is_ok_and(|op| op.is_term()))
        {
            let next = self.factor()?;
            acc = Node::Compound(Box::new(acc), Operator::try_from(&t).ok()?, Box::new(next));
        }
        if let Node::Compound(..) = acc {
            Some(acc)
        } else {
            None
        }
    }

    fn factor(&mut self) -> Option<Node> {
        let mut acc = self.term()?;
        while let Some(t) = self
            .it
            .next_if(|t| Operator::try_from(t).is_ok_and(|op| op.is_factor()))
        {
            let next = self.term()?;
            acc = Node::Compound(Box::new(acc), Operator::try_from(&t).ok()?, Box::new(next));
        }
        Some(acc)
    }

    fn term(&mut self) -> Option<Node> {
        let other_predicates = Expression::PARSE_OPTIONS
            .into_iter()
            .filter(|&&f| f != Expression::parse_binary);

        if let Some(expr) = super::test_any(other_predicates, self.it) {
            Some(Node::Scalar(Box::new(expr)))
        } else if matches!(self.it.next()?, super::Token { value, .. } if value == "(") {
            let exp = self.parse_exp();
            assert!(matches!(self.it.next(), Some(super::Token { value, .. }) if value == ")"));
            exp
        } else {
            None
        }
    }
}

// TODO change whole Option API to Result
#[inline]
pub fn parse(tokens: &mut Peekable<vec::IntoIter<super::Token>>) -> Option<Node> {
    Parser::from(tokens).parse_exp()
}
