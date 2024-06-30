#![feature(iter_next_chunk)]
#![feature(let_chains)]

use std::{iter::Peekable, mem, str::Chars};

use anyhow::{anyhow, Result};
use itertools::Itertools;

use self::definitions::*;

pub mod definitions;

struct Lexer<'a> {
    iterator: Peekable<Chars<'a>>,
}

impl Iterator for Lexer<'_> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator
            .peeking_take_while(|&c| c.is_ascii_whitespace() && c != '\n')
            .for_each(drop);

        if let Some(&c) = self.iterator.peek()
            && [definitions::STR_DELIM, definitions::RUNE_DELIM].contains(&c)
        {
            let Some(read) = self.iterator.clone().skip(1).position(|x| x == c) else {
                return Some(Err(anyhow!("Unclosed {c}")));
            };

            return Some(Ok(Token {
                value: self.iterator.by_ref().take(read + 2).collect(),
                r#type: TokenType::Literal(if c == definitions::STR_DELIM {
                    LiteralType::String
                } else {
                    LiteralType::Rune
                }),
            }));
        }

        if let Ok(rem) = self.iterator.clone().next_chunk::<COMMENT_PREFIX_LEN>()
            && String::from_iter(rem) == COMMENT_PREFIX
        {
            let value = self
                .iterator
                .by_ref()
                .peeking_take_while(|&c| c != '\n')
                .collect();

            return Some(Ok(Token {
                value,
                r#type: TokenType::Comment,
            }));
        }

        let mut acc = String::new();

        while let Some(c) = self.iterator.next() {
            acc.push(c);

            if let Some(t) = TokenType::eval(acc.as_str()) {
                if let Some(&next) = self.iterator.peek() {
                    let next_acc = acc.clone() + next.encode_utf8(&mut [0; 4]);
                    let next_t = TokenType::eval(next_acc.as_str());

                    // if next_t.is_some() {
                    if next_t
                        .is_some_and(|next_t| mem::discriminant(&t) == mem::discriminant(&next_t))
                        || allow_type_transmutation((acc.as_str(), t), (next_acc.as_str(), next_t))
                    {
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
pub fn tokenize(input: &str) -> Result<Vec<Token>> {
    Lexer {
        iterator: input.chars().peekable(),
    }
    .collect()
}
