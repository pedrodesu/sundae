#![feature(iter_next_chunk)]
#![feature(iter_intersperse)]

use std::iter::{self, Peekable};

use ecow::{EcoString, EcoVec};
use either::Either;
use itertools::Itertools;
use snafu::prelude::*;

use self::definitions::*;

pub mod definitions;
mod iterator;

use iterator::DynClonableIterator;

#[derive(Debug, Snafu, PartialEq)]
pub enum LexerError
{
    #[snafu(display("`{token}` is an invalid token"))]
    InvalidToken
    {
        token: String
    },
    #[snafu(display("Unclosed `{delim}`"))]
    UnclosedDelimiter
    {
        delim: char
    },
}

type LexerResult = Result<Token, LexerError>;

#[derive(Clone)]
pub struct Lexer<'a>
{
    iterator: Peekable<DynClonableIterator<'a>>,
}

impl Lexer<'_>
{
    fn get_str_or_rune(&mut self) -> Option<LexerResult>
    {
        let (init_pos, delim) = self
            .iterator
            .next_if(|&(_, c)| matches!(c, definitions::STR_DELIM | definitions::RUNE_DELIM))?;

        let elements = self
            .iterator
            .by_ref()
            .take_while_inclusive(|&(_, c)| c != delim)
            .collect::<Vec<_>>();

        let padded_value = iter::once(delim)
            .chain(elements.iter().map(|&(_, c)| c))
            .collect::<EcoString>();

        if elements.is_empty() || !padded_value.ends_with(delim)
        {
            return Some(Err(LexerError::UnclosedDelimiter { delim }));
        }

        let final_pos = elements.last().unwrap().0;

        let literal_type = match delim
        {
            definitions::STR_DELIM => LiteralType::String,
            definitions::RUNE_DELIM => LiteralType::Rune,
            _ => unreachable!(),
        };

        Some(Ok(Token {
            value: padded_value,
            r#type: TokenType::Literal(literal_type),
            span: Span {
                from: init_pos,
                to: final_pos,
            },
        }))
    }

    fn get_comment(&mut self) -> Option<Token>
    {
        let Ok(start) = self.iterator.clone().next_chunk::<COMMENT_PREFIX_LEN>()
        else
        {
            return None;
        };

        if start.map(|(_, c)| c).into_iter().ne(COMMENT_PREFIX.chars())
        {
            return None;
        }

        let from = start.first().unwrap().0;
        let iter_rem = self
            .iterator
            .peeking_take_while(|&(_, c)| c != '\n')
            .collect::<EcoVec<_>>();

        let to = iter_rem.last().unwrap().0;

        let value = iter_rem.into_iter().map(|(_, c)| c).collect();

        Some(Token {
            value,
            r#type: TokenType::Comment,
            span: Span { from, to },
        })
    }
}

impl Iterator for Lexer<'_>
{
    type Item = LexerResult;

    fn next(&mut self) -> Option<Self::Item>
    {
        self.iterator
            .peeking_take_while(|&(_, c)| c.is_ascii_whitespace() && c != '\n')
            .for_each(drop);

        if let Some(delim) = self
            .get_str_or_rune()
            .or_else(|| self.get_comment().map(|v| Ok(v)))
        {
            return Some(delim);
        }

        let mut acc = EcoString::new();
        let mut from = None;

        while let Some((s, c)) = self.iterator.next()
        {
            if from.is_none()
            {
                from = Some(s);
            }
            acc.push(c);

            if !c.is_ascii()
            {
                return Some(Err(LexerError::InvalidToken { token: acc.into() }));
            }
            else if let Some(t) = TokenType::eval(acc.as_str())
            {
                if let Some(&(_, next)) = self.iterator.peek()
                {
                    let next_acc = format!("{acc}{next}");
                    let next_t = TokenType::eval(next_acc.as_str());

                    // TODO quick fix, this should integrate flawlessly in the lexer
                    if next_t.is_some()
                        || (matches!(next_acc.as_str(), HEX_PREFIX | OCT_PREFIX | BIN_PREFIX))
                    {
                        continue;
                    }
                }

                return Some(Ok(Token {
                    value: acc,
                    r#type: t,
                    span: Span {
                        from: from.unwrap(),
                        to: s,
                    },
                }));
            }
        }

        // We should probably do something if `acc` is not empty at this point
        assert_eq!(acc, EcoString::new());

        None
    }
}

#[inline(always)]
pub fn tokenize(input: &'_ str) -> Lexer<'_>
{
    let n_lines = input.lines().count();

    let base_it = DynClonableIterator::new(Box::new(input.lines().enumerate().flat_map(
        move |(row, line)| {
            let it = if n_lines != row + 1
            {
                Either::Left(line.chars().chain(std::iter::once('\n')))
            }
            else
            {
                Either::Right(line.chars())
            };

            it.enumerate().map(move |(col, c)| ((row, col), c))
        },
    )))
    .peekable();

    Lexer { iterator: base_it }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn string_passes()
    {
        assert_eq!(
            Some(Ok(Token {
                value: "\"this is a string!\"".into(),
                r#type: TokenType::Literal(LiteralType::String),
                span: Span {
                    from: (0, 0),
                    to: (0, 18)
                }
            })),
            tokenize("\"this is a string!\"").get_str_or_rune()
        );

        assert_eq!(
            Some(Ok(Token {
                value: "\"this is 🚀 another \\x61\\u0061 string!\"".into(),
                r#type: TokenType::Literal(LiteralType::String),
                span: Span {
                    from: (0, 0),
                    to: (0, 37)
                }
            })),
            tokenize("\"this is 🚀 another \\x61\\u0061 string!\")").get_str_or_rune()
        );

        assert_eq!(
            Some(Err(LexerError::UnclosedDelimiter {
                delim: definitions::STR_DELIM
            })),
            tokenize("\"unfinished string").get_str_or_rune()
        );

        assert_eq!(
            Some(Err(LexerError::UnclosedDelimiter {
                delim: definitions::STR_DELIM
            })),
            tokenize("\"").get_str_or_rune()
        );

        assert_eq!(
            Some(Err(LexerError::UnclosedDelimiter {
                delim: definitions::STR_DELIM
            })),
            tokenize("\"\n}\n").get_str_or_rune()
        );
    }

    #[test]
    fn rune_passes()
    {
        assert_eq!(
            Some(Ok(Token {
                value: "`c`".into(),
                r#type: TokenType::Literal(LiteralType::Rune),
                span: Span {
                    from: (0, 0),
                    to: (0, 2)
                }
            })),
            tokenize("`c`").get_str_or_rune()
        );

        assert_eq!(
            Some(Ok(Token {
                value: "`🚀`".into(),
                r#type: TokenType::Literal(LiteralType::Rune),
                span: Span {
                    from: (0, 0),
                    to: (0, 2)
                }
            })),
            tokenize("`🚀`").get_str_or_rune()
        );

        assert_eq!(
            Some(Ok(Token {
                value: "`\\x61`".into(),
                r#type: TokenType::Literal(LiteralType::Rune),
                span: Span {
                    from: (0, 0),
                    to: (0, 5)
                }
            })),
            tokenize("`\\x61`").get_str_or_rune()
        );

        assert_eq!(
            Some(Ok(Token {
                value: "`\\u0061`".into(),
                r#type: TokenType::Literal(LiteralType::Rune),
                span: Span {
                    from: (0, 0),
                    to: (0, 7)
                }
            })),
            tokenize("`\\u0061`").get_str_or_rune()
        );

        assert_eq!(
            Some(Err(LexerError::UnclosedDelimiter {
                delim: definitions::RUNE_DELIM
            })),
            tokenize("`a").get_str_or_rune()
        );
    }

    #[test]
    fn comment_passes()
    {
        assert_eq!(
            Some(Token {
                value: "// a comment".into(),
                r#type: TokenType::Comment,
                span: Span {
                    from: (0, 0),
                    to: (0, 11)
                }
            }),
            tokenize("// a comment").get_comment()
        );

        assert_eq!(
            Some(Token {
                value: "// another comment".into(),
                r#type: TokenType::Comment,
                span: Span {
                    from: (0, 0),
                    to: (0, 17)
                }
            }),
            tokenize("// another comment\n\n").get_comment()
        );
    }
}
