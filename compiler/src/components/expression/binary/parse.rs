use crate::{
    components::Expression,
    lexer::definitions::Token,
    parser::{Component, TokenIt},
};

use super::{Node, Operator};

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

struct Parser<'a> {
    it: TokenIt<'a>,
}

impl<'a> From<TokenIt<'a>> for Parser<'a> {
    #[inline]
    fn from(it: TokenIt<'a>) -> Self {
        Self { it }
    }
}

impl Parser<'_> {
    pub fn consume(&mut self) -> Option<Node> {
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

        let test = other_predicates
            .into_iter()
            .find(|f| f(&mut self.it.clone()).is_some())?(self.it);

        if let Some(expr) = test {
            Some(Node::Scalar(Box::new(expr)))
        } else if matches!(self.it.next()?, Token { value, .. } if value == "(") {
            let exp = self.consume();
            if matches!(self.it.next(), Some(Token { value, .. }) if value == ")") {
                exp
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[inline]
pub fn parse(tokens: TokenIt) -> Option<Node> {
    Parser::from(tokens).consume()
}
