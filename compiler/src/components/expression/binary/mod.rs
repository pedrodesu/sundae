use super::Expression;

pub mod codegen;
pub mod parse;

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

#[derive(Clone, Debug)]
pub enum Node {
    Scalar(Box<Expression>),
    Compound(Box<Node>, Operator, Box<Node>),
}
