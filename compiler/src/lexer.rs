use std::{fmt, iter::Peekable, str::Chars};

use anyhow::{anyhow, Result};
use itertools::Itertools;
use tabled::Tabled;

const KEYWORDS: &[&str] = &["const", "let", "func", "ret"];

const OPERATORS: &[&str] = &[
    "and", "or", "+", "-", "*", "/", "+=", "-=", "*=", "/=", "<", ">", "<=", ">=", "=", "==", "!",
    "!=", "<<", ">>", "<<=", ">>=", "&", "|", "^", "&=", "|=", "^=",
];

const SEPARATORS: &[&str] = &["(", ")", "{", "}", ",", ";", "."];

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

impl TryFrom<&str> for TokenType {
    type Error = ();

    #[inline]
    fn try_from(expression: &str) -> Result<Self, Self::Error> {
        let is_delim = |delim: char| {
            expression.starts_with(delim) && expression.ends_with(delim) && expression.len() > 1
        };

        if KEYWORDS.contains(&expression) {
            Ok(TokenType::Keyword)
        } else if expression
            .chars()
            .all(|c| matches!(c, '_' | 'a'..='z' | 'A'..='Z' | '0'..='9'))
            && expression.starts_with(|c| !matches!(c, '0'..='9'))
        {
            Ok(TokenType::Identifier)
        } else if OPERATORS.contains(&expression) {
            Ok(TokenType::Operator)
        } else if SEPARATORS.contains(&expression) {
            Ok(TokenType::Separator)
        // TODO implement other number forms and use own checking
        // implement remaining literal patterns
        } else if is_delim(STR_DELIM) {
            Ok(TokenType::Literal(LiteralType::String))
        } else if is_delim(RUNE_DELIM) {
            Ok(TokenType::Literal(LiteralType::Rune))
        } else if expression.parse::<i64>().is_ok() {
            Ok(TokenType::Literal(LiteralType::Int))
        } else if expression.parse::<f64>().is_ok() {
            Ok(TokenType::Literal(LiteralType::Float))
        } else if BOOL_VALUES.contains(&expression) {
            Ok(TokenType::Literal(LiteralType::Bool))
        } else if COMMENT_PAIRS
            .into_iter()
            .any(|p| expression.starts_with(p.0) && expression.ends_with(p.1))
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
                            || (COMMENT_PAIRS.into_iter().any(|&p| next_acc == p.0)))
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
