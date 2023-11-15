use std::{iter::Peekable, str::Chars};

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

            if let Ok(r#type) = TokenType::try_from(&*acc) {
                if let Some(&next) = self.iterator.peek() {
                    let next_acc = acc.clone() + next.encode_utf8(&mut [0u8; 4]);
                    let next_type = TokenType::try_from(&*next_acc);

                    if !next_type.is_ok_and(|t| t == r#type) {
                        if !allow_type_transmutation(
                            (acc.as_str(), r#type),
                            (next_acc.as_str(), next_type),
                        ) {
                            // TODO re-do lexer ?
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
    // .filter(|t| !t.as_ref().is_ok_and(|v| v.r#type == TokenType::Comment))
    .collect()
}
