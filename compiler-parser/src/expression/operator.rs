use std::fmt;

use compiler_lexer::definitions::{Token, TokenType};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Operator
{
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
    BitAnd,
    BitOr,
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
        ("&", BitAnd),
        ("|", BitOr),
        ("^", Xor),
    ]
};

pub fn to_operator(token: &Token) -> Operator
{
    assert_eq!(token.r#type, TokenType::Operator, "Token isn't an operator");

    OPERATOR_MAP
        .iter()
        .copied()
        .find(|&(k, _)| k == token.value)
        .unwrap()
        .1
}

impl fmt::Display for Operator
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(
            f,
            "{}",
            OPERATOR_MAP
                .iter()
                .copied()
                .find(|&(_, v)| v == *self)
                .unwrap()
                .0
        )
    }
}
