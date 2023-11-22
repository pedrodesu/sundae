use std::{borrow::Cow, fmt};

const KEYWORDS: &[&str] = &["const", "func", "ret", "mut"];

const OPERATORS: &[&str] = &[
    "and", "or", "+", "-", "*", "/", "+=", "-=", "*=", "/=", "<", ">", "<=", ">=", "=", "==", "!",
    "!=", "<<", ">>", "<<=", ">>=", "&", "|", "^", "&=", "|=", "^=", ":=",
];

const SEPARATORS: &[&str] = &["(", ")", "[", "]", "{", "}", ",", ";", "."];

const STR_DELIM: char = '"';
const RUNE_DELIM: char = '`';

const COMMENT_PAIRS: &[(&str, &str)] = &[("//", "\n"), ("/*", "*/")];

#[inline]
pub(super) fn allow_type_transmutation(curr: Token, next: &str) -> bool {
    matches!(
        (curr, Token::try_from(next)),
        (Token::Identifier(_), Ok(Token::Keyword(_)))
            | (
                Token::Literal(_, LiteralType::Int),
                Ok(Token::Literal(_, LiteralType::Float))
            )
    ) || (COMMENT_PAIRS.into_iter().any(|&p| next == p.0))
        || matches!(next, "0x" | "0X" | "0o")
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum LiteralType {
    String,
    Rune,
    Int,
    Float,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Token {
    Keyword(String),
    Identifier(String),
    Operator(String),
    Literal(String, LiteralType),
    Separator(String),
    Comment,
    Newline,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Self::Keyword(v) => Cow::from(format!("KEYW\t{v}")),
                Self::Identifier(v) => Cow::from(format!("IDENT\t{v}")),
                Self::Operator(v) => Cow::from(format!("OP\t{v}")),
                Self::Literal(v, t) => Cow::from(format!("LIT\t{v}")),
                Self::Separator(v) => Cow::from(format!("SEP\t{v}")),
                Self::Comment => Cow::from("COMM"),
                Self::Newline => Cow::from("NEWL"),
            }
        )
    }
}

impl Token {
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

impl TryFrom<&str> for Token {
    type Error = ();

    #[inline]
    fn try_from(expr: &str) -> Result<Self, Self::Error> {
        let is_delim =
            |delim: char| expr.starts_with(delim) && expr.ends_with(delim) && expr.len() > 1;

        if KEYWORDS.contains(&expr) {
            Ok(Token::Keyword(expr.to_owned()))
        } else if expr
            .chars()
            .all(|c| matches!(c, '_' | 'a'..='z' | 'A'..='Z' | '0'..='9'))
            && expr.starts_with(|c| !matches!(c, '0'..='9'))
        {
            Ok(Token::Identifier(expr.to_owned()))
        } else if is_delim(RUNE_DELIM) {
            Ok(Token::Literal(expr.to_owned(), LiteralType::Rune))
        } else if Self::is_hex_int(expr)
            || Self::is_dec_int(expr)
            || Self::is_oct_int(expr)
            || Self::is_bin_int(expr)
        {
            Ok(Token::Literal(expr.to_owned(), LiteralType::Int))
        } else if Self::is_float(expr) {
            Ok(Token::Literal(expr.to_owned(), LiteralType::Float))
        } else if is_delim(STR_DELIM) {
            Ok(Token::Literal(expr.to_owned(), LiteralType::String))
        } else if OPERATORS.contains(&expr) {
            Ok(Token::Operator(expr.to_owned()))
        } else if SEPARATORS.contains(&expr) {
            Ok(Token::Separator(expr.to_owned()))
        } else if COMMENT_PAIRS
            .into_iter()
            .any(|p| expr.starts_with(p.0) && expr.ends_with(p.1))
        {
            Ok(Token::Comment)
        } else if expr == "\n" {
            Ok(Token::Newline)
        } else {
            Err(())
        }
    }
}
