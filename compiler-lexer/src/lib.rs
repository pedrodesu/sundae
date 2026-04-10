#![feature(iter_advance_by)]
#![feature(is_ascii_octdigit)]

use std::{
    iter::{Copied, Enumerate, Peekable},
    slice::Iter,
};

use bstr::ByteSlice;
use itertools::Itertools;
use miette::Diagnostic;
use thiserror::Error;

use crate::definitions::*;

pub mod definitions;

// TODO better messages on errors
#[derive(Error, Debug, Diagnostic, PartialEq)]
pub enum LexerError
{
    #[error("Unknown start of token {token}")]
    UnexpectedChar
    {
        #[label("Here")]
        span: Span,
        token: char,
    },
    #[error("Unclosed {}", match *delim {
        RUNE_DELIM => "rune",
        STR_DELIM => "string",
        _ => unreachable!()
    })]
    UnclosedDelim
    {
        #[label("Here")]
        span: Span,
        delim: u8,
    },
    #[error("Missing digits after exponent symbol")]
    MissingExp
    {
        #[label("Here")]
        span: Span,
    },
    #[error("Integer base must be lowercase")]
    IntBaseNotLowercase
    {
        #[label("Here")]
        span: Span,
    },
    #[error("Invalid digit on {}", if *base != 10 { "base {base} integer" } else if *float { "float" } else { "integer" })]
    InvalidDigitOnNumber
    {
        #[label("Here")]
        span: Span,
        float: bool,
        base: u32,
    },
    #[error("Missing digits after integer base")]
    MissingDigitsAfterIntBase
    {
        #[label("Here")]
        span: Span,
    },
    #[error("Invalid escape sequence")]
    InvalidEscapeSequence
    {
        #[label("Here")]
        span: Span,
    },
    #[error("{}", if *len == 0 { "Rune must not be empty" } else { "Rune must have exactly one codepoint" })]
    #[diagnostic(help("If you meant to create a string, use double quotes"))]
    InvalidRune
    {
        #[label("Here")]
        span: Span,
        len: usize,
    },
}

#[derive(Debug, PartialEq)]
pub enum LexerEvent
{
    Token(Token),
    Error(LexerError),
}

type SourceIter<'a> = Peekable<Enumerate<Copied<Iter<'a, u8>>>>;

pub struct Lexer<'a>
{
    source: &'a [u8],
    it: SourceIter<'a>,
}

impl<'a> From<&'a [u8]> for Lexer<'a>
{
    #[inline]
    fn from(bytes: &'a [u8]) -> Self
    {
        Self {
            source: bytes,
            it: bytes.iter().copied().enumerate().peekable(),
        }
    }
}

impl<'a> Iterator for Lexer<'a>
{
    type Item = LexerEvent;

    fn next(&mut self) -> Option<Self::Item>
    {
        self.skip_horizontal_whitespace();

        let (i, c) = self.it.peek().copied()?;

        if c == b'\n'
        {
            self.it.next();
            Some(self.token(TokenType::Newline, Span::single(i)))
        }
        else if let STR_DELIM | RUNE_DELIM = c
        {
            Some(self.lex_string_or_rune(i, c))
        }
        else if let Some(&COMMENT_PREFIX) = self.source[i..].first_chunk()
        {
            Some(self.lex_comment(i))
        }
        else if c == b'_' || c.is_ascii_alphabetic()
        {
            Some(self.lex_identifier_or_keyword(i))
        }
        else if self.source[i].is_ascii_digit()
        {
            Some(self.lex_number(i))
        }
        else if let Some(op) = self.operator_at(i)
        {
            Some(self.lex_operator(i, op))
        }
        else if self
            .it
            .next_if(|&(_, b)| SEPARATORS.binary_search(&b).is_ok())
            .is_some()
        {
            Some(self.token(TokenType::Separator, Span::single(i)))
        }
        else
        {
            Some(self.lex_unexpected_char(i))
        }
    }
}

impl<'a> Lexer<'a>
{
    fn skip_horizontal_whitespace(&mut self)
    {
        let Some((i, _)) = self.it.peek().copied()
        else
        {
            return;
        };

        let n = self.source[i..]
            .find_not_byteset(HORIZONTAL_WHITESPACE)
            .unwrap_or(self.source.len() - i);

        self.it.advance_by(n).unwrap();
    }

    #[inline]
    fn token(&self, r#type: TokenType, span: Span) -> LexerEvent
    {
        LexerEvent::Token(Token { r#type, span })
    }
}

impl<'a> Lexer<'a>
{
    fn lex_string_or_rune(&mut self, start: usize, delim: u8) -> LexerEvent
    {
        self.it.next();

        let Some(end) = self.find_closing_delim(start, delim)
        else
        {
            self.it.by_ref().for_each(drop);

            return LexerEvent::Error(LexerError::UnclosedDelim {
                span: Span::new(start, self.source.len()),
                delim,
            });
        };

        self.it.advance_by(end - start).unwrap();

        let span = Span::inclusive(start, end);
        let inner = &self.source[start + 1..end];

        let decoded_len = match self.decoded_char_len(inner, start + 1)
        {
            Ok(len) => len,
            Err(error) => return LexerEvent::Error(error),
        };

        if delim == RUNE_DELIM
            && let Err(error) = self.validate_rune(decoded_len, span)
        {
            return LexerEvent::Error(error);
        }

        self.token(
            TokenType::Literal(match delim
            {
                STR_DELIM => LiteralType::String,
                RUNE_DELIM => LiteralType::Rune,
                _ => unreachable!(),
            }),
            span,
        )
    }

    fn lex_comment(&mut self, start: usize) -> LexerEvent
    {
        let rem = &self.source[start..];
        let len = rem.find_byte(b'\n').unwrap_or(rem.len());

        self.it.advance_by(len).unwrap();

        self.token(TokenType::Comment, Span::new(start, start + len))
    }

    fn lex_identifier_or_keyword(&mut self, start: usize) -> LexerEvent
    {
        let end = self
            .it
            .peeking_take_while(|&(_, b)| Self::is_identifier_suffix(b))
            .last()
            .unwrap()
            .0;

        self.token(self.classify_ident(start, end), Span::inclusive(start, end))
    }

    fn lex_number(&mut self, start: usize) -> LexerEvent
    {
        let prefix = self.source[start..].first_chunk().copied();

        match prefix
            .map(|s| s.map(|b| b.to_ascii_lowercase()))
            .and_then(Self::special_base)
        {
            Some(base) => self.lex_prefixed_integer(start, prefix.unwrap(), base),
            None => self.lex_decimal_or_float(start),
        }
    }

    fn lex_prefixed_integer(&mut self, start: usize, prefix: [u8; 2], base: u32) -> LexerEvent
    {
        if prefix.map(|b| b.to_ascii_lowercase()) != prefix
        {
            self.it.advance_by(2).unwrap();

            return LexerEvent::Error(LexerError::IntBaseNotLowercase {
                span: Span::single(start + 1),
            });
        }

        self.it.advance_by(2).unwrap();

        let Some((end, _)) = self
            .it
            .peeking_take_while(|&(_, c)| Self::is_digit_in_base(c, base))
            .last()
        else
        {
            return LexerEvent::Error(LexerError::MissingDigitsAfterIntBase {
                span: Span::new(start, start + 2),
            });
        };

        if let Some(error) = self.number_suffix_error(end, base, false)
        {
            return error;
        }

        self.token(
            TokenType::Literal(LiteralType::Int),
            Span::inclusive(start, end),
        )
    }

    fn lex_decimal_or_float(&mut self, start: usize) -> LexerEvent
    {
        let mut end = start;
        let mut has_dot = false;
        let mut has_exp = false;

        while let Some(&(i, b)) = self.it.peek()
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
                        self.it.advance_by(end - start).unwrap();

                        return LexerEvent::Error(LexerError::MissingExp {
                            span: Span::inclusive(start, end),
                        });
                    }
                }
                _ => break,
            }

            end = i;
        }

        let is_float = has_dot || has_exp;

        if let Some(error) = self.number_suffix_error(end, 10, is_float)
        {
            return error;
        }

        self.token(
            TokenType::Literal(
                if is_float
                {
                    LiteralType::Float
                }
                else
                {
                    LiteralType::Int
                },
            ),
            Span::inclusive(start, end),
        )
    }

    fn lex_operator(&mut self, start: usize, op: &[u8]) -> LexerEvent
    {
        self.it.advance_by(op.len()).unwrap();
        self.token(TokenType::Operator, Span::new(start, start + op.len()))
    }

    fn lex_unexpected_char(&mut self, start: usize) -> LexerEvent
    {
        let c = self.source[start..].chars().next().unwrap();
        self.it.advance_by(c.len_utf8()).unwrap();

        LexerEvent::Error(LexerError::UnexpectedChar {
            span: Span::inclusive(start, start + c.len_utf8() - 1),
            token: c,
        })
    }
}

impl<'a> Lexer<'a>
{
    fn find_closing_delim(&self, start: usize, delim: u8) -> Option<usize>
    {
        let mut idx = start + 1;

        loop
        {
            let pos = self.source[idx..].find_byte(delim)?;
            let found_at = idx + pos;

            let backslashes = self.source[..found_at]
                .iter()
                .rev()
                .take_while(|&&b| b == b'\\')
                .count();

            if backslashes % 2 == 0
            {
                return Some(found_at);
            }

            idx = found_at + 1;
        }
    }

    fn validate_rune(&self, len: usize, span: Span) -> Result<(), LexerError>
    {
        if len == 1
        {
            Ok(())
        }
        else
        {
            Err(LexerError::InvalidRune { span, len })
        }
    }
}

impl<'a> Lexer<'a>
{
    fn number_suffix_error(&mut self, end: usize, base: u32, float: bool) -> Option<LexerEvent>
    {
        if !self
            .it
            .peek()
            .is_some_and(|&(_, b)| Self::is_identifier_suffix(b))
        {
            return None;
        }

        self.it.next();

        Some(LexerEvent::Error(LexerError::InvalidDigitOnNumber {
            span: Span::single(end + 1),
            float,
            base,
        }))
    }
}

impl<'a> Lexer<'a>
{
    fn decoded_char_len(&self, bytes: &[u8], offset: usize) -> Result<usize, LexerError>
    {
        let Ok(s) = std::str::from_utf8(bytes)
        else
        {
            return Err(LexerError::InvalidEscapeSequence {
                span: Span::new(offset, offset + bytes.len()),
            });
        };

        let bytes = s.as_bytes();
        let mut i = 0;
        let mut len = 0;

        while i < bytes.len()
        {
            if bytes[i] != b'\\'
            {
                let c = s[i..].chars().next().unwrap();
                i += c.len_utf8();
                len += 1;
                continue;
            }

            match bytes.get(i + 1).copied()
            {
                Some(b'x') =>
                {
                    if Self::take_hex_digits(bytes, i + 2, 2)
                    {
                        i += 4;
                        len += 1;
                    }
                    else
                    {
                        return Err(LexerError::InvalidEscapeSequence {
                            span: Self::invalid_hex_escape_span(offset, i, bytes, 2),
                        });
                    }
                }
                Some(b'u') =>
                {
                    if Self::take_hex_digits(bytes, i + 2, 4)
                    {
                        i += 6;
                        len += 1;
                    }
                    else
                    {
                        return Err(LexerError::InvalidEscapeSequence {
                            span: Self::invalid_hex_escape_span(offset, i, bytes, 4),
                        });
                    }
                }
                Some(b'n' | b'r' | b't' | b'0' | b'\\' | b'\'' | b'`') =>
                {
                    i += 2;
                    len += 1;
                }
                Some(b) =>
                {
                    return Err(LexerError::InvalidEscapeSequence {
                        span: Span::new(offset + i, offset + i + 1 + char::from(b).len_utf8()),
                    });
                }
                None =>
                {
                    return Err(LexerError::InvalidEscapeSequence {
                        span: Span::new(offset + i, offset + i + 1),
                    });
                }
            }
        }

        Ok(len)
    }

    fn take_hex_digits(bytes: &[u8], start: usize, count: usize) -> bool
    {
        bytes[start..]
            .get(..count)
            .is_some_and(|digits| digits.iter().all(u8::is_ascii_hexdigit))
    }

    fn invalid_hex_escape_span(
        offset: usize,
        escape_start: usize,
        bytes: &[u8],
        digits_len: usize,
    ) -> Span
    {
        let end = bytes[escape_start + 2..]
            .iter()
            .take(digits_len)
            .position(|b| !b.is_ascii_hexdigit())
            .map_or(escape_start + 2, |idx| escape_start + 3 + idx);

        Span::new(offset + escape_start, offset + end)
    }
}

impl<'a> Lexer<'a>
{
    #[inline]
    fn is_identifier_suffix(b: u8) -> bool
    {
        b.is_ascii_alphanumeric() || b == b'_' || !b.is_ascii()
    }

    #[inline]
    fn operator_at(&self, i: usize) -> Option<&'static [u8]>
    {
        OPERATORS
            .iter()
            .copied()
            .find(|&op| self.source[i..].starts_with(op))
    }

    #[inline]
    fn special_base(prefix: [u8; 2]) -> Option<u32>
    {
        match prefix
        {
            HEX_PREFIX => Some(16),
            OCT_PREFIX => Some(8),
            BIN_PREFIX => Some(2),
            _ => None,
        }
    }

    #[inline]
    fn is_digit_in_base(c: u8, base: u32) -> bool
    {
        match base
        {
            16 => c.is_ascii_hexdigit(),
            8 => c.is_ascii_octdigit(),
            2 => matches!(c, b'0' | b'1'),
            10 => c.is_ascii_digit(),
            _ => unreachable!(),
        }
    }

    fn classify_ident(&self, start: usize, end: usize) -> TokenType
    {
        let ident = &self.source[start..=end];

        if definitions::KEYWORDS.binary_search(&ident).is_ok()
        {
            TokenType::Keyword
        }
        else if definitions::KEYWORD_LIKE_OPERATORS
            .binary_search(&ident)
            .is_ok()
        {
            TokenType::Operator
        }
        else
        {
            TokenType::Identifier
        }
    }
}

#[inline(always)]
pub fn tokenize<S: AsRef<[u8]> + ?Sized>(source: &S) -> Lexer<'_>
{
    Lexer::from(source.as_ref())
}

#[cfg(test)]
mod tests
{
    use LiteralType::*;
    use TokenType::*;

    use super::*;

    macro_rules! assert_token {
        ($input:expr, $n:expr, $tt:expr) => {
            pretty_assertions::assert_eq!(
                tokenize($input).next(),
                Some(LexerEvent::Token(Token {
                    r#type: $tt,
                    span: (0..$n).into(),
                }))
            );
        };
    }

    macro_rules! assert_err {
        ($input:expr, $err:expr) => {
            pretty_assertions::assert_eq!(tokenize($input).next(), Some(LexerEvent::Error($err)));
        };
    }

    // TODO only need to fix strings and runes atp
    #[test]
    fn strings()
    {
        // Basic string
        assert_token!("\"hello\"", 7, Literal(String));

        // String with escapes
        assert_token!("\"line\\nbreak\"", 13, Literal(String));

        // String with unicode
        assert_token!("\"🚀🚀🚀\"", 14, Literal(String));

        // String with newline
        assert_token!("\"\n\"", 3, Literal(String));

        // Empty string
        assert_token!("\"\"", 2, Literal(String));

        // Unclosed string
        assert_err!(
            "\"unfinished",
            LexerError::UnclosedDelim {
                delim: STR_DELIM,
                span: (0..11).into(),
            }
        );

        // String with invalid escape
        assert_err!(
            "\"\\xZZ\"",
            LexerError::InvalidEscapeSequence {
                span: (1..4).into()
            }
        );

        // String with invalid unicode sequence
        assert_err!(
            "\"\\uZZZZ\"",
            LexerError::InvalidEscapeSequence {
                span: (1..4).into()
            }
        );
    }

    #[test]
    fn runes()
    {
        // Basic rune
        assert_token!("`a`", 3, Literal(Rune));

        // Rune with unicode
        assert_token!("`🚀`", 6, Literal(Rune));

        // Rune with escapes
        assert_token!("`\\u0061`", 8, Literal(Rune));
        assert_token!("`\\x61`", 6, Literal(Rune));

        // Empty rune
        assert_err!(
            "``",
            LexerError::InvalidRune {
                len: 0,
                span: (0..2).into()
            }
        );

        // Rune too long
        assert_err!(
            "`ab`",
            LexerError::InvalidRune {
                len: 2,
                span: (0..4).into()
            }
        );

        // Unclosed rune
        assert_err!(
            "`a",
            LexerError::UnclosedDelim {
                delim: RUNE_DELIM,
                span: (0..2).into(),
            }
        );

        // Rune with invalid escape
        assert_err!(
            "`\\xZZ`",
            LexerError::InvalidEscapeSequence {
                span: (1..4).into()
            }
        );
    }

    #[test]
    fn comments()
    {
        // Simple comment
        assert_token!("// hello", 8, Comment);

        // Comment with newline
        assert_token!("// another comment\n", 18, Comment);
    }

    mod integers
    {
        use super::*;

        #[test]
        fn hexadecimal()
        {
            assert_token!("0x42069FFFff", 12, Literal(Int));
            assert_token!("0xffffffffffffffffffffffffffffffff", 34, Literal(Int));
            assert_token!("0xDEADBEEF", 10, Literal(Int));

            assert_err!(
                "0x",
                LexerError::MissingDigitsAfterIntBase {
                    span: (0..2).into()
                }
            );
            assert_err!(
                "0X42069FFFff",
                LexerError::IntBaseNotLowercase { span: 1.into() }
            );
            assert_err!(
                "0xFFFFFG",
                LexerError::InvalidDigitOnNumber {
                    base: 16,
                    float: false,
                    span: 7.into(),
                }
            );
        }

        #[test]
        fn octal()
        {
            assert_token!("0o01234567", 10, Literal(Int));
            assert_token!("0o777", 5, Literal(Int));

            assert_err!(
                "0o",
                LexerError::MissingDigitsAfterIntBase {
                    span: (0..2).into()
                }
            );
            assert_err!("0O777", LexerError::IntBaseNotLowercase { span: 1.into() });
            assert_err!(
                "0o7778",
                LexerError::InvalidDigitOnNumber {
                    base: 8,
                    float: false,
                    span: 5.into(),
                }
            );
        }

        #[test]
        fn binary()
        {
            assert_token!("0b000101010010101010101", 23, Literal(Int));
            assert_token!("0b1010", 6, Literal(Int));

            assert_err!(
                "0b",
                LexerError::MissingDigitsAfterIntBase {
                    span: (0..2).into()
                }
            );
            assert_err!("0B1010", LexerError::IntBaseNotLowercase { span: 1.into() });
            assert_err!(
                "0b10102",
                LexerError::InvalidDigitOnNumber {
                    base: 2,
                    float: false,
                    span: 6.into(),
                }
            );
        }

        #[test]
        fn decimal()
        {
            assert_token!("0", 1, Literal(Int));
            assert_token!("00", 2, Literal(Int));
            assert_token!("01234", 5, Literal(Int));
            assert_token!("1234", 4, Literal(Int));

            assert_err!(
                "42a",
                LexerError::InvalidDigitOnNumber {
                    base: 10,
                    float: false,
                    span: 2.into(),
                }
            );
        }
    }

    #[test]
    fn floats()
    {
        assert_token!("12.34", 5, Literal(Float));
        assert_token!("01234.00", 8, Literal(Float));
        assert_token!("42.060", 6, Literal(Float));

        // Exponent notation
        assert_token!("12.34e5", 7, Literal(Float));
        assert_token!("12.34E5", 7, Literal(Float));
        assert_token!("12.34e+5", 8, Literal(Float));
        assert_token!("12.34e-5", 8, Literal(Float));

        // Only prefix
        assert_token!("64.", 3, Literal(Float));
        assert_token!("00.", 3, Literal(Float));

        assert_err!(
            "42.0a",
            LexerError::InvalidDigitOnNumber {
                base: 10,
                float: true,
                span: 4.into(),
            }
        );
    }

    #[test]
    fn identifiers()
    {
        assert_token!("abc", 3, Identifier);
        assert_token!("abc_def", 7, Identifier);
        assert_token!("_abc_", 5, Identifier);
        assert_token!("abc23", 5, Identifier);
        assert_token!("_", 1, Identifier);
    }

    #[test]
    fn operators()
    {
        for &op in OPERATORS
        {
            let op = str::from_utf8(op).unwrap();
            assert_token!(op, op.len(), Operator);
        }
    }

    #[test]
    fn separators()
    {
        for &sep in SEPARATORS
        {
            let slice = &[sep];
            let sep = str::from_utf8(slice).unwrap();
            assert_token!(sep, sep.len(), Separator);
        }
    }
}
