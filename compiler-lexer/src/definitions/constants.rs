// These arrays must remain sorted accordingly - Rust doesn't provide easy sorting at compile time (yet).

// This array is binary searched. It must be sorted by Ord.
pub const KEYWORDS: &[&[u8]] = &[b"const", b"func", b"if", b"let", b"mut", b"ret"];

// This array is searched first against the longest match. It must be sorted by descending length.
pub const OPERATORS: &[&[u8]] = &[
    b"<<=", b">>=", b"+=", b"-=", b"*=", b"/=", b"!=", b"&=", b"|=", b"^=", b"<<", b">>", b"==",
    b"<=", b">=", b"+", b"-", b"*", b"/", b"!", b"&", b"|", b"^", b"<", b">",
];

// This array is binary searched. It must be sorted by Ord.
pub const KEYWORD_LIKE_OPERATORS: &[&[u8]] = &[b"and", b"or"];

// This array is binary searched. It must be sorted by Ord.
pub const SEPARATORS: &[u8] = &[b'(', b')', b',', b'.', b'=', b'[', b']', b'{', b'}'];

pub const HORIZONTAL_WHITESPACE: &[u8] = b" \t";

pub const STR_DELIM: u8 = b'"';
pub const RUNE_DELIM: u8 = b'`';
pub const COMMENT_PREFIX: [u8; 2] = *b"//";

pub const HEX_PREFIX: [u8; 2] = *b"0x";
pub const OCT_PREFIX: [u8; 2] = *b"0o";
pub const BIN_PREFIX: [u8; 2] = *b"0b";
