// 2*4+(6-9)+10*4-(9/4)

/*

*/

use std::cell::RefCell;
use std::vec::IntoIter;

use itertools::FoldWhile::{Continue, Done};
use itertools::Itertools;

enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    LeftBrace,
    RightBrace,
}

enum Leaf {
    Number(f64),
    Compound(Box<Leaf>, Operator, Box<Leaf>),
}

struct Parser {
    it: IntoIter<Token>,
}

impl Parser {
    fn parse_exp(&mut self) -> Leaf {
        let mut acc = self.term();
        while let Some(Token::Operator(op @ (Operator::Mul | Operator::Div))) = self.it.next() {
            let next = self.term();
            acc = Leaf::Compound(Box::new(acc), op, Box::new(next));
        }
        acc
    }

    fn factor(&mut self) -> Leaf {
        self.it
            .fold_while(self.term(), |acc, n| {
                if let Some(Token::Operator(op @ (Operator::Mul | Operator::Div))) = self.it.next()
                {
                    Continue(Leaf::Compound(Box::new(acc), op, Box::new(self.term())))
                } else {
                    Done(acc)
                }
            })
            .into_inner()
        /*
        let mut acc = self.term();
        while let Some(Token::Operator(op @ (Operator::Mul | Operator::Div))) = self.it.next() {
            let next = self.term();
            acc = Leaf::Compound(Box::new(acc), op, Box::new(next));
        }
        acc
        */
    }

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

impl Iterator for Parser {
    type Item = Leaf;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

enum Token {
    Number(f64),
    Operator(Operator),
}

fn main() {
    Parser {
        it: vec![
            Token::Number(2.0),
            Token::Operator(Operator::Mul),
            Token::Number(4.0),
            Token::Operator(Operator::Add),
            Token::Operator(Operator::LeftBrace),
            Token::Number(6.0),
            Token::Operator(Operator::Sub),
            Token::Number(9.0),
            Token::Operator(Operator::RightBrace),
            Token::Operator(Operator::Add),
            Token::Number(10.0),
            Token::Operator(Operator::Mul),
            Token::Number(4.0),
            Token::Operator(Operator::Sub),
            Token::Operator(Operator::LeftBrace),
            Token::Number(9.0),
            Token::Operator(Operator::Div),
            Token::Number(4.0),
            Token::Operator(Operator::RightBrace),
        ]
        .into_iter(),
    };
}
