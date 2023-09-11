use std::{fmt, iter::Peekable, str::Chars};

use anyhow::{anyhow, Result};
use itertools::Itertools;
use tabled::Tabled;

const KEYWORDS: &[&str] = &["const", "let", "func", "ret"];

const OPERATORS: &[&str] = &[
    "and", "or", "+", "-", "*", "/", "+=", "-=", "*=", "/=", "<", ">", "<=", ">=", "=", "==", "!",
    "!=", "<<", ">>", "<<=", ">>=", "&", "|", "^", "&=", "|=", "^=",
];

const SEPARATORS: &[&str] = &["(", ")", "[", "]", "{", "}", ",", ";", "."];

const STR_DELIM: char = '"';
const RUNE_DELIM: char = '`';

const BOOL_VALUES: &[&str] = &["true", "false"];

const COMMENT_PAIRS: &[(&str, &str)] = &[("//", "\n"), ("/*", "*/")];

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum LiteralType {
    String,
    Rune,
    Int,
    Float,
    Bool,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TokenType {
    Keyword,
    Identifier,
    Operator,
    Literal(LiteralType),
    Separator,
    Comment,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                TokenType::Keyword => "Keyword",
                TokenType::Identifier => "Identifier",
                TokenType::Operator => "Operator",
                TokenType::Literal(_) => "Literal",
                TokenType::Separator => "Separator",
                TokenType::Comment => "Comment",
            }
        )
    }
}

impl TokenType {
    #[inline]
    fn is_hex_int(expression: &str) -> bool {
        if !(expression.len() > 2) {
            false
        } else {
            let (prefix, rem) = expression.split_at(2);
            (prefix == "0x" || prefix == "0X")
                && rem
                    .chars()
                    .all(|c| matches!(c, '0'..='9' | 'a'..='f' | 'A'..='F'))
        }
    }

    #[inline]
    fn is_dec_int(expression: &str) -> bool {
        expression.chars().all(|c| c.is_ascii_digit() || c == '-')
            && ((expression.matches('-').count() == 1 && expression.starts_with('-'))
                || expression.matches('-').count() == 0)
    }

    #[inline]
    fn is_oct_int(expression: &str) -> bool {
        if !(expression.len() > 2) {
            false
        } else {
            let (prefix, rem) = expression.split_at(2);
            (prefix == "0o" || prefix == "0O") && rem.chars().all(|c| matches!(c, '0'..='6'))
        }
    }

    #[inline]
    fn is_bin_int(expression: &str) -> bool {
        if !(expression.len() > 2) {
            false
        } else {
            let (prefix, rem) = expression.split_at(2);
            (prefix == "0b" || prefix == "0B") && rem.chars().all(|c| matches!(c, '0' | '1'))
        }
    }

    #[inline]
    fn is_float(expression: &str) -> bool {
        expression
            .chars()
            .all(|c| c.is_ascii_digit() || c == '-' || c == '.')
            && ((expression.matches('-').count() == 1 && expression.starts_with('-'))
                || expression.matches('-').count() == 0)
            && expression.matches('.').count() == 1
    }
}

impl TryFrom<&str> for TokenType {
    type Error = ();

    #[inline]
    fn try_from(expr: &str) -> Result<Self, Self::Error> {
        let is_delim =
            |delim: char| expr.starts_with(delim) && expr.ends_with(delim) && expr.len() > 1;

        if KEYWORDS.contains(&expr) {
            Ok(TokenType::Keyword)
        } else if expr
            .chars()
            .all(|c| matches!(c, '_' | 'a'..='z' | 'A'..='Z' | '0'..='9'))
            && expr.starts_with(|c| !matches!(c, '0'..='9'))
        {
            Ok(TokenType::Identifier)
        } else if is_delim(RUNE_DELIM) {
            Ok(TokenType::Literal(LiteralType::Rune))
        } else if Self::is_hex_int(expr)
            || Self::is_dec_int(expr)
            || Self::is_oct_int(expr)
            || Self::is_bin_int(expr)
        {
            Ok(TokenType::Literal(LiteralType::Int))
        } else if Self::is_float(expr) {
            Ok(TokenType::Literal(LiteralType::Float))
        } else if BOOL_VALUES.contains(&expr) {
            Ok(TokenType::Literal(LiteralType::Bool))
        } else if OPERATORS.contains(&expr) {
            Ok(TokenType::Operator)
        } else if SEPARATORS.contains(&expr) {
            Ok(TokenType::Separator)
        } else if is_delim(STR_DELIM) {
            Ok(TokenType::Literal(LiteralType::String))
        } else if COMMENT_PAIRS
            .into_iter()
            .any(|p| expr.starts_with(p.0) && expr.ends_with(p.1))
        {
            Ok(TokenType::Comment)
        } else {
            Err(())
        }
    }
}

#[derive(Tabled, Debug, Clone)]
pub struct Token {
    pub value: String,
    pub r#type: TokenType,
}

struct Lexer<'a> {
    iterator: Peekable<Chars<'a>>,
}

impl Iterator for Lexer<'_> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut acc = String::new();

        let _ = self
            .iterator
            .by_ref()
            .peeking_take_while(|&c| c.is_ascii_whitespace())
            .for_each(drop);

        while let Some(c) = self.iterator.next() {
            acc.push(c);

            if let Ok(r#type) = TokenType::try_from(&*acc) {
                if let Some(&next) = self.iterator.peek() {
                    let next_acc = acc.clone() + next.encode_utf8(&mut [0u8; 4]);
                    let next_type = TokenType::try_from(&*next_acc);

                    if !next_type.is_ok_and(|t| t == r#type) {
                        if !((r#type == TokenType::Identifier
                            && matches!(next_type, Ok(TokenType::Keyword)))
                            || (r#type == TokenType::Literal(LiteralType::Int)
                                && matches!(next_type, Ok(TokenType::Literal(LiteralType::Float))))
                            || (COMMENT_PAIRS.into_iter().any(|&p| next_acc == p.0))
                            || (matches!(&*next_acc, "0x" | "0X" | "0o" | "0O" | "0b" | "0B")))
                        {
                            return Some(Ok(Token { value: acc, r#type }));
                        }
                    }
                } else {
                    return Some(Ok(Token { value: acc, r#type }));
                }
            }
        }

        if !acc.is_empty() {
            return Some(Err(anyhow!(
                "Incomplete expression ({acc:?} is an invalid expression)"
            )));
        }

        None
    }
}

#[inline(always)]
pub fn tokenize(input: &str) -> Result<Vec<Token>> {
    Lexer {
        iterator: input.chars().peekable(),
    }
    .filter(|t| !t.as_ref().is_ok_and(|v| v.r#type == TokenType::Comment))
    .collect()
}
