use super::Span;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum LiteralType {
    String,
    Rune,
    Int,
    Float,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TokenType {
    Keyword,
    Identifier,
    Operator,
    Literal(LiteralType),
    Separator,
    Comment,
    Newline,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Token {
    pub r#type: TokenType,
    pub span: Span,
}

impl Token {
    #[inline]
    pub fn value<'a>(&self, source: &'a [u8]) -> &'a [u8] {
        self.span.source(source)
    }

    #[inline]
    pub fn is_value(&self, source: &[u8], value: &str) -> bool {
        self.value(source) == value.as_bytes()
    }
}
