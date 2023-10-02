use super::{parser_types::Name, Expression};

pub mod codegen;
pub mod parse;

#[derive(Debug, Clone)]
pub enum Statement {
    Return(Option<Expression>),
    Expression(Expression),
    Assign {
        destination: Expression,
        source: Expression,
    },
    Local {
        mutable: bool,
        name: Name,
        init: Option<Expression>,
    },
}
