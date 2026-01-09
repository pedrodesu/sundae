use miette::SourceSpan;

// These constants must remain sorted accordingly - Rust is still a bit dumb and doesn't provide easy sorting at compile time (yet).

// This array is binary searched. It must be sorted by Ord.
pub const KEYWORDS: &[&[u8]] = &[b"const", b"func", b"if", b"let", b"mut", b"ret"];

// This array is searched first against the longest match. It must be sorted by descending length.
pub const OPERATORS: &[&[u8]] = &[
    b"<<=", b">>=", b"+=", b"-=", b"*=", b"/=", b"!=", b"&=", b"|=", b"^=", b"<<", b">>", b"==",
    b"<=", b">=", b"+", b"-", b"*", b"/", b"!", b"&", b"|", b"^", b"<", b">", b"and", b"or",
];

// This array is binary searched. It must be sorted by Ord.
pub const KEYWORD_LIKE_OPERATORS: &[&[u8]] = &[b"and", b"or"];

// This array is binary searched. It must be sorted by Ord.
pub const SEPARATORS: &[u8] = &[b'(', b')', b',', b'.', b'=', b'[', b']', b'{', b'}'];

pub const STR_DELIM: u8 = b'"';
pub const RUNE_DELIM: u8 = b'`';
pub const COMMENT_PREFIX: [u8; 2] = *b"//";

pub const HEX_PREFIX: [u8; 2] = *b"0x";
pub const OCT_PREFIX: [u8; 2] = *b"0o";
pub const BIN_PREFIX: [u8; 2] = *b"0b";

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
    pub span: SourceSpan,
}
