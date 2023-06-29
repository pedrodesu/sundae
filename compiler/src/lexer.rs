use std::{fmt, iter::Peekable, str::Chars};

use itertools::Itertools;
use tabled::Tabled;

const KEYWORDS: &[&str] = &["const", "let", "ret"];

const OPERATORS: &[&str] = &[
    "and", "or", "+", "-", "*", "/", "+=", "-=", "*=", "/=",
    "<", ">", "<=", ">=", "=", "==", "!", "!=", "<<", ">>", "<<=", ">>=", "&", "|", "^", "&=",
    "|=", "^=",
];

const SEPARATORS: &[&str] = &["(", ")", "{", "}", ",", ";", "."];

const STR_DELIM: char = '"';
const RUNE_DELIM: char = '`';

#[derive(PartialEq, Clone, Copy)]
pub enum TokenType {
    Keyword,
    Identifier,
    Operator,
    Literal,
    Separator
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
                TokenType::Literal => "Literal",
                TokenType::Separator => "Separator",
            }
        )
    }
}

impl TokenType {
    #[inline]
    fn from(expression: &str) -> Option<TokenType> {
        let is_delim = #[inline]
        |delim: char| {
            expression.starts_with(delim) && expression.ends_with(delim) && expression.len() > 1
        };

        if KEYWORDS.contains(&expression) {
            Some(TokenType::Keyword)
        } else if expression
            .chars()
            .all(|c| matches!(c, '_' | 'a'..='z' | 'A'..='Z' | '0'..='9'))
            && expression.starts_with(|c| !matches!(c, '0'..='9'))
        {
            Some(TokenType::Identifier)
        } else if OPERATORS.contains(&expression) {
            Some(TokenType::Operator)
        } else if SEPARATORS.contains(&expression) {
            Some(TokenType::Separator)
        } else if is_delim(STR_DELIM) || is_delim(RUNE_DELIM) || expression.parse::<f64>().is_ok()
        // TODO implement other number forms and use own checking
        // implement remaining literal patterns
        {
            Some(TokenType::Literal)
        } else {
            None
        }
    }
}

#[derive(Tabled, Clone)]
pub struct Token {
    pub value: String,
    pub r#type: TokenType,
}

struct Lexer<'a> {
    iterator: Peekable<Chars<'a>>,
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let mut acc = String::new();

        let _ = self
            .iterator
            .by_ref()
            .peeking_take_while(|&c| c.is_ascii_whitespace())
            .for_each(drop);

        while let Some(c) = self.iterator.next() {
            acc.push(c);

            let r#type = TokenType::from(&acc);
            if let Some(r#type) = r#type {
                if let Some(&next) = self.iterator.peek() {
                    let next = {
                        let mut tmp_buf = [0u8; 4];
                        let next_acc = acc.clone() + next.encode_utf8(&mut tmp_buf);
                        TokenType::from(&next_acc)
                    };

                    if !matches!(next, Some(t) if t == r#type) {
                        if !matches!(
                            (r#type, next),
                            (TokenType::Identifier, Some(TokenType::Keyword))
                        ) {
                            return Some(Token { value: acc, r#type });
                        }
                    }
                }
            }
        }

        None
    }
}

#[inline(always)]
pub fn tokenize(input: &str) -> Vec<Token> {
    Lexer {
        iterator: input.chars().peekable(),
    }
    .collect()
}
