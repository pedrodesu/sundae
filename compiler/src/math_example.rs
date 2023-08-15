use std::{iter::Peekable, vec::IntoIter};

#[derive(Debug)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    LeftBrace,
    RightBrace,
}

#[derive(Debug)]
pub enum Leaf {
    Number(f64),
    Compound(Box<Leaf>, Operator, Box<Leaf>),
}

pub struct Parser {
    pub it: Peekable<IntoIter<Token>>,
}

impl Parser {
    pub fn parse_exp(&mut self) -> Leaf {
        let mut acc = self.factor();
        while let Some(Token::Operator(op)) = self
            .it
            .next_if(|t| matches!(t, Token::Operator(Operator::Add | Operator::Sub)))
        {
            let next = self.factor();
            acc = Leaf::Compound(Box::new(acc), op, Box::new(next));
        }
        acc
    }

    fn factor(&mut self) -> Leaf {
        let mut acc = self.term();
        while let Some(Token::Operator(op)) = self
            .it
            .next_if(|t| matches!(t, Token::Operator(Operator::Mul | Operator::Div)))
        {
            let next = self.term();
            acc = Leaf::Compound(Box::new(acc), op, Box::new(next));
        }
        acc
    }

    // 2*4+(6-9)+10*4-(9/4)
    fn term(&mut self) -> Leaf {
        match self.it.next() {
            Some(Token::Number(n)) => Leaf::Number(n),
            Some(Token::Operator(Operator::LeftBrace)) => {
                let exp = self.parse_exp();
                assert!(matches!(
                    self.it.next(),
                    Some(Token::Operator(Operator::RightBrace))
                ));
                exp
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub enum Token {
    Number(f64),
    Operator(Operator),
}
