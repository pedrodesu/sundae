#![feature(iter_next_chunk)]
#![feature(iter_advance_by)]
#![feature(is_ascii_octdigit)]
#![feature(assert_matches)]

use std::{
    borrow::Cow,
    iter::{Enumerate, Peekable},
    str::Bytes,
};

use bstr::ByteSlice;
use itertools::Itertools;
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::definitions::*;

pub mod definitions;

const HORIZONTAL_WHITESPACE: &[u8] = b" \t\x0b\x0c\r";

// TODO better messages on errors
#[derive(Error, Debug, Diagnostic, PartialEq)]
#[error(transparent)]
pub enum LexerError<'a>
{
    #[error("Unknown start of token {token}")]
    UnknownTokenStart
    {
        #[source_code]
        src: Cow<'a, str>,

        #[label("Here")]
        span: SourceSpan,

        token: char,
    },
    #[error("Unclosed {}", match *delim {
        RUNE_DELIM => "rune",
        STR_DELIM => "string",
        _ => unreachable!()
    })]
    UnclosedDelim
    {
        delim: u8,

        #[source_code]
        src: Cow<'a, str>,

        #[label("Here")]
        span: SourceSpan,
    },
    #[error("Missing digits after exponent symbol")]
    MissingExp
    {
        #[source_code]
        src: Cow<'a, str>,

        #[label("Here")]
        span: SourceSpan,
    },
    #[error("Integer base must be lowercase")]
    IntBaseNotLowercase
    {
        #[source_code]
        src: Cow<'a, str>,

        #[label("Here")]
        span: SourceSpan,
    },
    #[error("Invalid digit on {}", if *base != 10 { "base {base} integer" } else if *float { "float" } else { "integer" })]
    InvalidDigitOnNumber
    {
        float: bool,

        base: u32,

        #[source_code]
        src: Cow<'a, str>,

        #[label("Here")]
        span: SourceSpan,
    },
    #[error("Missing digits after integer base")]
    MissingDigitsAfterIntBase
    {
        #[source_code]
        src: Cow<'a, str>,

        #[label("Here")]
        span: SourceSpan,
    },
    #[error("{}", if *len == 0 { "Rune must not be empty" } else { "Rune must have exactly one codepoint" })]
    #[diagnostic(help("If you meant to create a string, use double quotes"))]
    InvalidRune
    {
        len: usize,

        #[source_code]
        src: Cow<'a, str>,

        #[label("Here")]
        span: SourceSpan,
    },
}

type LexerResult<'a> = Result<Token, LexerError<'a>>;

pub struct Lexer<'a>
{
    source: &'a [u8],
    it: Peekable<Enumerate<Bytes<'a>>>,
}

impl<'a> From<&'a str> for Lexer<'a>
{
    #[inline]
    fn from(source: &'a str) -> Self
    {
        Self {
            source: source.as_bytes(),
            it: source.bytes().enumerate().peekable(),
        }
    }
}

impl<'a> Iterator for Lexer<'a>
{
    type Item = LexerResult<'a>;

    fn next(&mut self) -> Option<Self::Item>
    {
        let (i, _) = self.it.peek().copied()?;

        let n = self.source[i..]
            .find_not_byteset(HORIZONTAL_WHITESPACE)
            .unwrap_or(self.source.len() - i);

        self.it.advance_by(n).unwrap();

        let (i, c) = self.it.peek().copied()?;

        let is_invalid_suffix = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || !b.is_ascii();

        // Newlines
        if c == b'\n'
        {
            self.it.next();
            Some(Ok(Token {
                r#type: TokenType::Newline,
                span: (i..=i).into(),
            }))
        }
        // Strings
        // Runes
        else if let STR_DELIM | RUNE_DELIM = c
        {
            let (mut last, delim) = self.it.next().unwrap();

            while let Some((j, c)) = self.it.next()
            {
                last = j;
                match c
                {
                    b'\\' =>
                    {
                        if self.it.next().is_none()
                        {
                            break;
                        }
                    }
                    c if c == delim =>
                    {
                        // TODO handle rune checking
                        // if delim == RUNE_DELIM && j - i != 2
                        // {
                        //     return Some(Err(LexerError::InvalidRune {
                        //         src: self.source,
                        //         span: (i..=j).into(),
                        //         len: j - i - 1,
                        //     }));
                        // }

                        return Some(Ok(Token {
                            r#type: TokenType::Literal(match delim
                            {
                                STR_DELIM => LiteralType::String,
                                RUNE_DELIM => LiteralType::Rune,
                                _ => unreachable!(),
                            }),
                            span: (i..=j).into(),
                        }));
                    }
                    _ =>
                    {}
                }
            }

            Some(Err(LexerError::UnclosedDelim {
                delim,
                src: self.source.to_str_lossy(),
                span: (i..=last).into(),
            }))
        }
        // Comments
        else if let Ok(COMMENT_PREFIX) = self.it.clone().map(|(_, b)| b).next_chunk()
        {
            let n = self.source[i..].find_byte(b'\n').unwrap_or_default();

            self.it.advance_by(n).unwrap();

            Some(Ok(Token {
                r#type: TokenType::Comment,
                span: (i..=i + n).into(),
            }))
        }
        // Keyword
        // Identifier
        else if c == b'_' || c.is_ascii_alphabetic()
        {
            let (j, _) = self
                .it
                .peeking_take_while(|&(_, b)| is_invalid_suffix(b))
                .last()
                .unwrap();

            Some(Ok(Token {
                r#type: if definitions::KEYWORDS
                    .binary_search(&&self.source[i..=j])
                    .is_ok()
                {
                    TokenType::Keyword
                }
                else if definitions::KEYWORD_LIKE_OPERATORS
                    .binary_search(&&self.source[i..=j])
                    .is_ok()
                {
                    TokenType::Operator
                }
                else
                {
                    TokenType::Identifier
                },
                span: (i..=j).into(),
            }))
        }
        // Integers
        // Floats
        else if c.is_ascii_digit()
            || self
                .it
                .clone()
                .map(|(_, b)| b)
                .next_chunk::<2>()
                .is_ok_and(|chunk| chunk[0] == b'.' && chunk[1].is_ascii_digit())
        {
            let special_base = |s| match s
            {
                HEX_PREFIX => Some(16),
                OCT_PREFIX => Some(8),
                BIN_PREFIX => Some(2),
                _ => None,
            };

            let sub = self.it.clone().map(|(_, b)| b).next_chunk().ok();
            let base = sub.and_then(special_base).unwrap_or(10);

            if base != 10
            {
                self.it.advance_by(2).unwrap();

                let Some((j, _)) = self
                    .it
                    .peeking_take_while(|&(_, c)| match base
                    {
                        16 => c.is_ascii_hexdigit(),
                        8 => c.is_ascii_octdigit(),
                        2 => matches!(c, b'0' | b'1'),
                        10 => c.is_ascii_digit(),
                        _ => unreachable!(),
                    })
                    .last()
                else
                {
                    return Some(Err(LexerError::MissingDigitsAfterIntBase {
                        src: self.source.to_str_lossy(),
                        span: (i..i + 2).into(),
                    }));
                };

                if self.it.peek().is_some_and(|&(_, b)| is_invalid_suffix(b))
                {
                    return Some(Err(LexerError::InvalidDigitOnNumber {
                        base,
                        float: false,
                        src: self.source.to_str_lossy(),
                        span: (j + 1).into(),
                    }));
                }

                Some(Ok(Token {
                    r#type: TokenType::Literal(LiteralType::Int),
                    span: (i..=j).into(),
                }))
            }
            else if sub
                .and_then(|s| special_base(s.map(|b| b.to_ascii_lowercase())))
                .is_some()
            {
                Some(Err(LexerError::IntBaseNotLowercase {
                    src: self.source.to_str_lossy(),
                    span: (i + 1).into(),
                }))
            }
            else
            {
                let mut last = i;
                let mut has_dot = false;
                let mut has_exp = false;

                while let Some(&(j, b)) = self.it.peek()
                {
                    match b
                    {
                        b'0'..=b'9' =>
                        {
                            self.it.next();
                        }
                        b'.' if !has_dot && !has_exp =>
                        {
                            self.it.next();
                            has_dot = true;
                        }
                        b'e' | b'E' if !has_exp =>
                        {
                            self.it.next();
                            has_exp = true;

                            self.it.next_if(|&(_, b)| matches!(b, b'+' | b'-'));

                            if !matches!(self.it.peek(), Some(&(_, b)) if b.is_ascii_digit())
                            {
                                return Some(Err(LexerError::MissingExp {
                                    src: self.source.to_str_lossy(),
                                    span: (i..=last).into(),
                                }));
                            }
                        }
                        _ => break,
                    }
                    last = j;
                }

                // TODO this happens twice, maybe wrap in a nice function?
                if self
                    .it
                    .peek()
                    .is_some_and(|&(_, b)| b.is_ascii_alphanumeric() || b == b'_' || !b.is_ascii())
                {
                    return Some(Err(LexerError::InvalidDigitOnNumber {
                        base: 10,
                        float: has_dot || has_exp,
                        src: self.source.to_str_lossy(),
                        span: (last + 1).into(),
                    }));
                }

                Some(Ok(Token {
                    r#type: if has_dot || has_exp
                    {
                        TokenType::Literal(LiteralType::Float)
                    }
                    else
                    {
                        TokenType::Literal(LiteralType::Int)
                    },
                    span: (i..=last).into(),
                }))
            }
        }
        // Operator
        else if let Some(op) = OPERATORS.iter().copied().find(|&op| {
            let end = i + op.len();
            self.source.get(i..end).is_some_and(|v| v == op)
        })
        {
            self.it.advance_by(op.len()).unwrap();
            Some(Ok(Token {
                r#type: TokenType::Operator,
                span: (i..i + op.len()).into(),
            }))
        }
        // Separator
        else if let Some((i, _)) = self
            .it
            .next_if(|&(_, b)| SEPARATORS.binary_search(&b).is_ok())
        {
            Some(Ok(Token {
                r#type: TokenType::Separator,
                span: (i..=i).into(),
            }))
        }
        else
        {
            self.it.by_ref().for_each(drop);

            let c = self.source[i..].chars().next().unwrap();

            Some(Err(LexerError::UnknownTokenStart {
                src: self.source.to_str_lossy(),
                span: i.into(),
                token: c,
            }))
        }
    }
}

#[inline(always)]
pub fn tokenize<'a>(source: &'a str) -> Lexer<'a>
{
    Lexer::from(source)
}

#[cfg(test)]
mod tests
{
    use std::assert_matches::assert_matches;

    use super::*;

    macro_rules! assert_token {
        ($input:expr, $n:expr, $tt:expr) => {
            pretty_assertions::assert_eq!(
                tokenize($input).next(),
                Some(Ok(Token {
                    r#type: $tt,
                    span: (0..$n).into(),
                }))
            );
        };
    }

    macro_rules! assert_err {
        ($input:expr, $range:expr, $variant:path { $($fields:tt)* }) => {
            assert_matches!(
                tokenize($input).next(),
                Some(Err($variant {
                    span: s,
                    $($fields)*
                })) if s == ($range).into()
            );
        };
    }

    // TODO only need to fix strings and runes atp
    #[test]
    fn strings()
    {
        // Basic string
        assert_token!("\"hello\"", 7, TokenType::Literal(LiteralType::String));

        // String with escapes
        assert_token!(
            "\"line\\nbreak\"",
            13,
            TokenType::Literal(LiteralType::String)
        );

        // String with unicode
        assert_token!("\"🚀🚀🚀\"", 14, TokenType::Literal(LiteralType::String));

        // String with newline
        assert_token!("\"\n\"", 3, TokenType::Literal(LiteralType::String));

        // Empty string
        assert_token!("\"\"", 2, TokenType::Literal(LiteralType::String));

        // Unclosed string
        assert_err!(
            "\"unfinished",
            (0..11),
            LexerError::UnclosedDelim {
                delim: STR_DELIM,
                ..
            }
        );

        // TODO we need to validate unicode and escape sequences inside strings and runes
        // String with invalid escape
        // pretty_assertions::assert_eq!(
        //     Some(Ok(Token {
        //         value: "\"\\xZZ\"".into(),
        //         r#type: TokenType::Literal(LiteralType::String),
        //         span: (0, 6),
        //     })),
        //     tokenize("\"\\xZZ\"").get_str_or_rune()
        // );

        // String with invalid unicode sequence
        // pretty_assertions::assert_eq!(
        //     Some(Ok(Token {
        //         value: "\"\\uZZZZ\"".into(),
        //         r#type: TokenType::Literal(LiteralType::String),
        //         span: (0, 8),
        //     })),
        //     tokenize("\"\\uZZZZ\"").get_str_or_rune()
        // );
    }

    #[test]
    fn runes()
    {
        // Basic rune
        assert_token!("`a`", 3, TokenType::Literal(LiteralType::Rune));

        // Rune with unicode
        assert_token!("`🚀`", 6, TokenType::Literal(LiteralType::Rune));

        // Rune with escapes
        assert_token!("`\\u0061`", 8, TokenType::Literal(LiteralType::Rune));
        assert_token!("`\\x61`", 6, TokenType::Literal(LiteralType::Rune));

        // Empty rune
        // assert_err!("``", 2, LexerError::InvalidRune { len: 0, .. });

        // Rune too long
        // assert_err!("`ab`", 4, LexerError::InvalidRune { len: 2, .. });

        // Unclosed rune
        assert_err!(
            "`a",
            (0..2),
            LexerError::UnclosedDelim {
                delim: RUNE_DELIM,
                ..
            }
        );

        // Rune with invalid escape
        // pretty_assertions::assert_eq!(
        //     Some(Err(LexerError::InvalidRune {
        //         len: 4,
        //         src: "`\\xZZ`".into(),
        //         span: (0, 6).into(),
        //     })),
        //     tokenize("`\\xZZ`").get_str_or_rune()
        // );
    }

    #[test]
    fn comments()
    {
        // Simple comment
        assert_token!("// hello", 8, TokenType::Comment);

        // Comment with newline
        assert_token!("// another comment\n", 18, TokenType::Comment);
    }

    mod integers
    {
        use super::*;

        #[test]
        fn hexadecimal()
        {
            assert_token!("0x42069FFFff", 12, TokenType::Literal(LiteralType::Int));
            assert_token!(
                "0xffffffffffffffffffffffffffffffff",
                34,
                TokenType::Literal(LiteralType::Int)
            );
            assert_token!("0xDEADBEEF", 10, TokenType::Literal(LiteralType::Int));

            assert_err!("0x", (0..2), LexerError::MissingDigitsAfterIntBase { .. });
            assert_err!("0X42069FFFff", 1, LexerError::IntBaseNotLowercase { .. });
            assert_err!(
                "0xFFFFFG",
                7,
                LexerError::InvalidDigitOnNumber {
                    base: 16,
                    float: false,
                    ..
                }
            );
        }

        #[test]
        fn octal()
        {
            assert_token!("0o01234567", 10, TokenType::Literal(LiteralType::Int));
            assert_token!("0o777", 5, TokenType::Literal(LiteralType::Int));

            assert_err!("0o", (0..2), LexerError::MissingDigitsAfterIntBase { .. });
            assert_err!("0O777", 1, LexerError::IntBaseNotLowercase { .. });
            assert_err!(
                "0o7778",
                5,
                LexerError::InvalidDigitOnNumber {
                    base: 8,
                    float: false,
                    ..
                }
            );
        }

        #[test]
        fn binary()
        {
            assert_token!(
                "0b000101010010101010101",
                23,
                TokenType::Literal(LiteralType::Int)
            );
            assert_token!("0b1010", 6, TokenType::Literal(LiteralType::Int));

            assert_err!("0b", (0..2), LexerError::MissingDigitsAfterIntBase { .. });
            assert_err!("0B1010", 1, LexerError::IntBaseNotLowercase { .. });
            assert_err!(
                "0b10102",
                6,
                LexerError::InvalidDigitOnNumber {
                    base: 2,
                    float: false,
                    ..
                }
            );
        }

        #[test]
        fn decimal()
        {
            assert_token!("0", 1, TokenType::Literal(LiteralType::Int));
            assert_token!("00", 2, TokenType::Literal(LiteralType::Int));
            assert_token!("01234", 5, TokenType::Literal(LiteralType::Int));
            assert_token!("1234", 4, TokenType::Literal(LiteralType::Int));

            assert_err!(
                "42a",
                2,
                LexerError::InvalidDigitOnNumber {
                    base: 10,
                    float: false,
                    ..
                }
            );
        }
    }

    #[test]
    fn floats()
    {
        assert_token!("12.34", 5, TokenType::Literal(LiteralType::Float));
        assert_token!("01234.00", 8, TokenType::Literal(LiteralType::Float));
        assert_token!("42.060", 6, TokenType::Literal(LiteralType::Float));

        // Exponent notation
        assert_token!("12.34e5", 7, TokenType::Literal(LiteralType::Float));
        assert_token!("12.34E5", 7, TokenType::Literal(LiteralType::Float));
        assert_token!("12.34e+5", 8, TokenType::Literal(LiteralType::Float));
        assert_token!("12.34e-5", 8, TokenType::Literal(LiteralType::Float));

        // Only prefix
        assert_token!("64.", 3, TokenType::Literal(LiteralType::Float));
        assert_token!("00.", 3, TokenType::Literal(LiteralType::Float));

        // Only suffix
        assert_token!(".4", 2, TokenType::Literal(LiteralType::Float));
        assert_token!(".0", 2, TokenType::Literal(LiteralType::Float));

        assert_err!(
            "42.0a",
            4,
            LexerError::InvalidDigitOnNumber {
                base: 10,
                float: true,
                ..
            }
        );
    }

    #[test]
    fn identifiers()
    {
        assert_token!("abc", 3, TokenType::Identifier);
        assert_token!("abc_def", 7, TokenType::Identifier);
        assert_token!("_abc_", 5, TokenType::Identifier);
        assert_token!("abc23", 5, TokenType::Identifier);
        assert_token!("_", 1, TokenType::Identifier);
    }

    #[test]
    fn operators()
    {
        for &op in OPERATORS
        {
            let op = str::from_utf8(op).unwrap();
            assert_token!(op, op.len(), TokenType::Operator);
        }
    }

    #[test]
    fn separators()
    {
        for &sep in SEPARATORS
        {
            let slice = &[sep];
            let sep = str::from_utf8(slice).unwrap();
            assert_token!(sep, sep.len(), TokenType::Separator);
        }
    }
}

/*
TODO

Since your tests use assert_err! and look for a single error, you are currently in a "fail-fast" mode. If you want to make this "Industrial Grade," you can change the else block (Unknown Token) to:
Record the error.
self.it.next() (skip just one byte).
Continue lexing.
This allows your users to see 5 typos in one compile instead of fixing one, re-running, and finding the next.
*/
