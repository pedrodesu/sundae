use super::{
    parser_types::{ArgumentName, Name, ParserType},
    Expression, Statement,
};

mod codegen;
mod parse;

#[derive(Debug)]
pub struct Signature {
    pub name: (String, Option<ParserType>),
    pub arguments: Vec<ArgumentName>,
}

#[derive(Debug)]
pub enum Item {
    Const {
        name: Name,
        value: Expression,
    },
    Function {
        signature: Signature,
        body: Vec<Statement>,
    },
}
