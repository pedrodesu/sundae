use crate::lexer::definitions::LiteralType;

use super::Statement;

pub mod binary;
pub mod codegen;
pub mod parse;

#[derive(Clone, Debug)]
pub enum Expression {
    Literal {
        value: String,
        r#type: LiteralType,
    },
    Path(Vec<String>),
    Reference(Box<Expression>),
    Dereference(Box<Expression>),
    Binary(binary::Node),
    Call {
        path: Vec<String>,
        args: Vec<Expression>,
    },
    If {
        condition: Box<Expression>,
        block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
    },
}
