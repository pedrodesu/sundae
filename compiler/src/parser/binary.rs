use std::{iter::Peekable, vec};

use crate::lexer::{Token, TokenType};

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

#[derive(Debug)]
pub enum Node {
    Scalar(Box<Expression>),
    Compound(Box<Node>, Operator, Box<Node>),
}

struct Parser<'a> {
    it: &'a mut Peekable<vec::IntoIter<Token>>,
}

impl<'a> From<&'a mut Peekable<vec::IntoIter<Token>>> for Parser<'a> {
    #[inline]
    fn from(it: &'a mut Peekable<vec::IntoIter<Token>>) -> Self {
        Self { it }
    }
}

impl Parser<'_> {
    pub fn parse_exp(&mut self) -> Option<Node> {
        let mut acc = self.factor()?;
        // TODO refactor this expression
        while let Some(t) = self
            .it
            .next_if(|t| Operator::try_from(t).is_ok_and(|op| op.is_term()))
        {
            let next = self.factor()?;
            acc = Node::Compound(Box::new(acc), Operator::try_from(&t).ok()?, Box::new(next));
        }
        Some(acc)
    }

    fn factor(&mut self) -> Option<Node> {
        let mut acc = self.term()?;
        // TODO refactor this expression
        while let Some(t) = self
            .it
            .next_if(|t| Operator::try_from(t).is_ok_and(|op| op.is_factor()))
        {
            let next = self.term()?;
            acc = Node::Compound(Box::new(acc), Operator::try_from(&t).ok()?, Box::new(next));
        }
        Some(acc)
    }

    // TODO meter qlqr expression aqui
    fn term(&mut self) -> Option<Node> {
        // Expression::PARSE_OPTIONS;

        match self.it.next()? {
            token @ Token {
                r#type: TokenType::Literal,
                ..
            } => Some(Node::Scalar(Box::new(token))),
            Token { value, .. } if value == "(" => {
                let exp = self.parse_exp();
                assert!(matches!(self.it.next(), Some(Token { value, .. }) if value == ")"));
                exp
            }
            _ => None,
        }
    }
}

// TODO change whole Option API to Result
#[inline]
pub fn parse(tokens: &mut Peekable<vec::IntoIter<Token>>) -> Option<Node> {
    Parser::from(tokens).parse_exp()
}
