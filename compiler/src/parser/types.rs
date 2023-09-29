use crate::lexer::definitions::{LiteralType, TokenType};

use super::{TokenIt, TokenItBaseExt};

#[derive(Debug, Clone)]
pub enum BaseType {
    Array { r#type: String, size: usize },
    Scalar { r#type: String },
}

#[derive(Debug, Clone)]
pub enum Modifiers {
    Const,
    Mut,
    Ref,
    MutRef,
}

#[derive(Debug, Clone)]
pub struct ParserType {
    pub base: BaseType,
    pub modifier: Option<Modifiers>,
}

pub(super) trait TokenItTypeExt {
    fn get_modifier(self) -> Option<Modifiers>;

    fn get_type(self) -> Option<ParserType>;
}

impl TokenItTypeExt for TokenIt<'_> {
    fn get_modifier(self) -> Option<Modifiers> {
        if self.consume(|t| t.value == "const").is_some() {
            Some(Modifiers::Const)
        } else if self.consume(|t| t.value == "mut").is_some() {
            Some(Modifiers::Mut)
        } else if self.consume(|t| t.value == "&").is_some() {
            if self.consume(|t| t.value == "mut").is_some() {
                Some(Modifiers::MutRef)
            } else {
                Some(Modifiers::Ref)
            }
        } else {
            None
        }
    }

    fn get_type(self) -> Option<ParserType> {
        let modifier = self.get_modifier();

        if let Some(r#type) = self.consume(|t| t.r#type == TokenType::Identifier) {
            Some(ParserType {
                base: BaseType::Scalar { r#type },
                modifier,
            })
        } else if self.consume(|t| t.value == "[").is_some() {
            let size = self
                .consume(|t| t.r#type == TokenType::Literal(LiteralType::Int))?
                .parse()
                .ok()?;
            self.consume(|t| t.value == "]")?;
            let r#type: String = self.consume(|t| t.r#type == TokenType::Identifier)?;

            Some(ParserType {
                base: BaseType::Array { r#type, size },
                modifier,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArgumentName(pub String, pub ParserType);

#[derive(Debug, Clone)]
pub struct Name(pub String, pub Option<ParserType>);
