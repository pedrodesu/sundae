use std::fmt;

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
    BitAnd,
    BitOr,
    Xor,
}

const OPERATOR_MAP: &[(Operator, &[u8])] = {
    use Operator::*;

    &[
        (Plus, b"+"),
        (Minus, b"-"),
        (Star, b"*"),
        (Div, b"/"),
        (And, b"and"),
        (Or, b"or"),
        (Lt, b"<"),
        (Gt, b">"),
        (Le, b"<="),
        (Ge, b">="),
        (EqEq, b"=="),
        (Neq, b"!="),
        (Shl, b"<<"),
        (Shr, b">>"),
        (BitAnd, b"&"),
        (BitOr, b"|"),
        (Xor, b"^"),
    ]
};

impl Operator {
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        OPERATOR_MAP
            .iter()
            .copied()
            .find(|&(_, b)| b == bytes)
            .map(|(op, _)| op)
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = OPERATOR_MAP
            .iter()
            .copied()
            .find(|&(op, _)| op == *self)
            .map(|(_, b)| b)
            .unwrap();

        // SAFETY: We define the values of `OPERATOR_MAP` ourselves. We know that they are always valid UTF-8.
        let s = unsafe { std::str::from_utf8_unchecked(value) };
        f.write_str(s)
    }
}
