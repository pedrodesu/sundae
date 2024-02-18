use std::{borrow::Cow, fmt};

const KEYWORDS: &[&str] = &["const", "func", "ret", "mut"];

const OPERATORS: &[&str] = &[
    "and", "or", "+", "-", "*", "/", "+=", "-=", "*=", "/=", "<", ">", "<=", ">=", "=", "==", "!",
    "!=", "<<", ">>", "<<=", ">>=", "&", "|", "^", "&=", "|=", "^=", ":=",
];

const SEPARATORS: &[&str] = &["(", ")", "[", "]", "{", "}", ",", ";", "."];

const STR_DELIM: char = '"';
const RUNE_DELIM: char = '`';

#[inline]
pub(super) fn allow_type_transmutation(
    (_, curr_type): (&str, TokenType),
    (next, next_type): (&str, Option<TokenType>),
) -> bool {
    (curr_type == TokenType::Identifier && matches!(next_type, Some(TokenType::Keyword)))
        || (curr_type == TokenType::Literal(LiteralType::Int)
            && matches!(next_type, Some(TokenType::Literal(LiteralType::Float))))
        || next == "//"
        || matches!(next, "0x" | "0X" | "0o" | "0O" | "0b" | "0B")
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum LiteralType {
    String,
    Rune,
    Int,
    Float,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TokenType {
    Keyword,
    Identifier,
    Operator,
    Literal(LiteralType),
    Separator,
    Comment,
    Newline,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub value: String,
    pub r#type: TokenType,
}

impl fmt::Display for Token {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = &self.value;
        write!(
            f,
            "{}",
            match self.r#type {
                TokenType::Keyword => Cow::from(format!("KEYW\t{v}")),
                TokenType::Identifier => Cow::from(format!("IDENT\t{v}")),
                TokenType::Operator => Cow::from(format!("OP\t{v}")),
                TokenType::Literal(_) => Cow::from(format!("LIT\t{v}")),
                TokenType::Separator => Cow::from(format!("SEP\t{v}")),
                TokenType::Comment => Cow::from("COMM"),
                TokenType::Newline => Cow::from("NEWL"),
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

impl TokenType {
    #[inline]
    pub fn eval(expr: &str, is_end_of_comment: bool) -> Option<Self> {
        let is_delim =
            |delim: char| expr.starts_with(delim) && expr.ends_with(delim) && expr.len() > 1;

        if KEYWORDS.contains(&expr) {
            Some(TokenType::Keyword)
        } else if expr
            .chars()
            .all(|c| matches!(c, '_' | 'a'..='z' | 'A'..='Z' | '0'..='9'))
            && expr.starts_with(|c| !matches!(c, '0'..='9'))
        {
            Some(TokenType::Identifier)
        } else if is_delim(RUNE_DELIM) {
            Some(TokenType::Literal(LiteralType::Rune))
        } else if Self::is_hex_int(expr)
            || Self::is_dec_int(expr)
            || Self::is_oct_int(expr)
            || Self::is_bin_int(expr)
        {
            Some(TokenType::Literal(LiteralType::Int))
        } else if Self::is_float(expr) {
            Some(TokenType::Literal(LiteralType::Float))
        } else if OPERATORS.contains(&expr) {
            Some(TokenType::Operator)
        } else if SEPARATORS.contains(&expr) {
            Some(TokenType::Separator)
        } else if is_delim(STR_DELIM) {
            Some(TokenType::Literal(LiteralType::String))
        } else if expr.starts_with("//") && is_end_of_comment {
            Some(TokenType::Comment)
        } else if expr == "\n" {
            Some(TokenType::Newline)
        } else {
            None
        }
    }
}
