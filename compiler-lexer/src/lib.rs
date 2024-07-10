#![feature(iter_next_chunk)]

use std::{fmt, iter::Peekable, str::Chars};

use anyhow::{anyhow, Result};
use ecow::EcoString;
use itertools::Itertools;

use self::definitions::*;

pub mod definitions;

#[derive(Debug, Clone)]
struct Lexer<'a> {
    iterator: Peekable<Chars<'a>>,
}

impl Lexer<'_> {
    fn get_str_or_rune(&mut self) -> Option<Result<Token>> {
        let Some(c @ (definitions::STR_DELIM | definitions::RUNE_DELIM)) =
            self.iterator.peek().copied()
        else {
            return None;
        };

        let Some(read) = self.iterator.clone().skip(1).position(|x| x == c) else {
            return Some(Err(anyhow!("Unclosed {c}")));
        };

        Some(Ok(Token {
            value: self.iterator.by_ref().take(read + 2).collect(),
            r#type: TokenType::Literal(if c == definitions::STR_DELIM {
                LiteralType::String
            } else {
                LiteralType::Rune
            }),
        }))
    }

    fn get_comment(&mut self) -> Option<Token> {
        let Ok(rem) = self.iterator.clone().next_chunk::<COMMENT_PREFIX_LEN>() else {
            return None;
        };

        if EcoString::from_iter(rem) != COMMENT_PREFIX {
            return None;
        }

        let value = self.iterator.peeking_take_while(|&c| c != '\n').collect();

        Some(Token {
            value,
            r#type: TokenType::Comment,
        })
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator
            .peeking_take_while(|&c| c.is_ascii_whitespace() && c != '\n')
            .for_each(drop);

        if let Some(ret) = self.get_str_or_rune() {
            return Some(ret);
        } else if let Some(comment) = self.get_comment() {
            return Some(Ok(comment));
        }

        let mut acc = EcoString::new();

        while let Some(c) = self.iterator.next() {
            acc.push(c);

            if let Some(t) = TokenType::eval(acc.as_str()) {
                if let Some(&next) = self.iterator.peek() {
                    let next_acc = format!("{}{}", acc, next);
                    let next_t = TokenType::eval(next_acc.as_str());

                    if next_t.is_some() {
                        continue;
                    }
                }

                return Some(Ok(Token {
                    value: acc,
                    r#type: t,
                }));
            } else if c.is_ascii_whitespace() {
                return Some(Err(anyhow!(
                    "`{:?}` is an invalid token",
                    &acc[..acc.len() - 1]
                )));
            }
        }

        None
    }
}

#[inline(always)]
pub fn tokenize(input: &str) -> impl Iterator<Item = Result<Token>> + Clone + fmt::Debug + '_ {
    Lexer {
        iterator: input.chars().peekable(),
    }
}
