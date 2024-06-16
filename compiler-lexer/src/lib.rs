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

            if let Some(t) = TokenType::eval(
                acc.as_str(),
                matches!(self.iterator.peek(), None | Some('\n')),
            ) {
                if let Some(&next) = self.iterator.peek() {
                    let next_acc = acc.clone() + next.encode_utf8(&mut [0u8; 4]);
                    let next_t = TokenType::eval(next_acc.as_str(), false); // end of comment doesn't matter on peek

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
            }
        }

        if !acc.is_empty() {
            Some(Err(anyhow!(
                "Incomplete expression ({acc:?} is an invalid expression)"
            )))
        } else {
            None
        }
    }
}

#[inline(always)]
pub fn tokenize(input: &str) -> Result<Vec<Token>> {
    Lexer {
        iterator: input.chars().peekable(),
    }
    .collect()
}
