use super::Span;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum LiteralType
{
    String,
    Rune,
    Int,
    Float,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TokenType
{
    Keyword,
    Identifier,
    Operator,
    Literal(LiteralType),
    Separator,
    Comment,
    Newline,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Token
{
    pub r#type: TokenType,
    pub span: Span,
}
