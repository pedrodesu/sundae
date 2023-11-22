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
        let mut acc = String::new();

        self.iterator
            .peeking_take_while(|&c| c.is_ascii_whitespace() && c != '\n')
            .for_each(drop);

        while let Some(c) = self.iterator.next() {
            acc.push(c);

            if let Ok(t) = Token::try_from(&*acc) {
                if let Some(&next) = self.iterator.peek() {
                    let next_acc = acc.clone() + next.encode_utf8(&mut [0u8; 4]);
                    let next_t = Token::try_from(&*next_acc);

                    if !next_t
                        .is_ok_and(|next_t| mem::discriminant(&t) == mem::discriminant(&next_t))
                    {
                        if !allow_type_transmutation(t, &next_acc) {
                            // TODO re-do lexer ?
                            return Some(Ok(t));
                        }
                    }
                } else {
                    return Some(Ok(t));
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
    // .filter(|t| !t.as_ref().is_ok_and(|v| v.r#type == TokenType::Comment))
    .collect()
}
