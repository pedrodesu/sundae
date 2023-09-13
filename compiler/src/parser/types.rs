use crate::lexer::definitions::{LiteralType, TokenType};

use super::{consume, TokenIt};

#[derive(Debug, Clone)]
pub enum BaseType {
    Array(String, u32),
    Scalar(String),
}

#[derive(Debug, Clone)]
pub enum Modifiers {
    Const,
    Mut,
    Ref,
    MutRef,
}

#[derive(Debug, Clone)]
pub struct Type {
    base: BaseType,
    modifier: Option<Modifiers>,
}

pub(super) fn get_type(tokens: TokenIt<'_>) -> Option<Type> {
    if let Some(r#type) = tokens
        .next_if(|t| t.r#type == TokenType::Identifier)
        .map(|t| t.value)
    {
        Some(Type {
            base: BaseType::Scalar(r#type),
            modifier: None,
        })
    } else {
        consume(tokens, |t| t.value == "[")?;
        let size = consume(tokens, |t| t.r#type == TokenType::Literal(LiteralType::Int))?
            .parse()
            .unwrap();
        consume(tokens, |t| t.value == "]")?;
        let r#type: String = consume(tokens, |t| t.r#type == TokenType::Identifier)?;

        Some(Type {
            base: BaseType::Array(r#type, size),
            modifier: None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Name(pub String, pub Option<Type>);

// TODO solve question of extracting attributes on types (const, mut, &mut and &)
