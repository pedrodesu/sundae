use std::fmt;

use compiler_lexer::definitions::{Token, TokenType};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Operator {
    Plus,
    Minus,
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

const OPERATOR_MAP: &[(&str, Operator)] = {
    use Operator::*;

    &[
        ("+", Plus),
        ("-", Minus),
        ("*", Star),
        ("/", Div),
        ("and", And),
        ("or", Or),
        ("<", Lt),
        (">", Gt),
        ("<=", Le),
        (">=", Ge),
        ("==", EqEq),
        ("!=", Neq),
        ("<<", Shl),
        (">>", Shr),
        ("&", ShAnd),
        ("|", ShOr),
        ("^", Xor),
    ]
};

// TODO make this const when possible without features
pub fn to_operator(token: &Token) -> Operator {
    if token.r#type != TokenType::Operator {
        panic!("Token isn't an operator");
    }

    OPERATOR_MAP
        .into_iter()
        .copied()
        .find(|&(k, _)| k == token.value.as_str())
        .unwrap()
        .1
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            OPERATOR_MAP
                .into_iter()
                .copied()
                .find(|&(_, v)| v == *self)
                .unwrap()
                .0
        )
    }
}
