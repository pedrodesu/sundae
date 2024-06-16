const KEYWORDS: &[&str] = &["const", "func", "ret", "mut"];

const OPERATORS: &[&str] = &[
    "and", "or", "+", "-", "*", "/", "+=", "-=", "*=", "/=", "<", ">", "<=", ">=", "=", "==", "!",
    "!=", "<<", ">>", "<<=", ">>=", "&", "|", "^", "&=", "|=", "^=", ":=",
];

const SEPARATORS: &[&str] = &["(", ")", "[", "]", "{", "}", ",", "."];

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
        || matches!(next, "0x" | "0o" | "0b")
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

impl TokenType {
    #[inline]
    fn is_hex_int(expression: &str) -> bool {
        if expression.len() <= 2 {
            false
        } else {
            let (prefix, rem) = expression.split_at(2);
            prefix == "0x"
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
        if expression.len() <= 2 {
            false
        } else {
            let (prefix, rem) = expression.split_at(2);
            prefix == "0o" && rem.chars().all(|c| matches!(c, '0'..='7'))
        }
    }

    #[inline]
    fn is_bin_int(expression: &str) -> bool {
        if expression.len() <= 2 {
            false
        } else {
            let (prefix, rem) = expression.split_at(2);
            prefix == "0b" && rem.chars().all(|c| matches!(c, '0' | '1'))
        }
    }

    #[inline]
    fn is_float(expression: &str) -> bool {
        expression
            .chars()
            .all(|c| c.is_ascii_digit() || c == '-' || c == '.')
            && ((expression.matches('-').count() == 1 && expression.starts_with('-'))
                || expression.matches('-').count() == 0)
            && (expression.matches('.').count() == 1 && !expression.starts_with('.'))
    }

    #[inline]
    fn is_identifier(expression: &str) -> bool {
        expression
            .chars()
            .all(|c| matches!(c, '_' | 'a'..='z' | 'A'..='Z' | '0'..='9'))
            && expression.starts_with(|c| !matches!(c, '0'..='9'))
    }

    #[inline]
    pub fn eval(expr: &str, is_end_of_comment: bool) -> Option<Self> {
        let is_delim =
            |delim: char| expr.starts_with(delim) && expr.ends_with(delim) && expr.len() > 1;

        if KEYWORDS.contains(&expr) {
            Some(TokenType::Keyword)
        } else if Self::is_identifier(expr) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hexadecimal_is_correct() {
        assert!(TokenType::is_hex_int("0x42069FFFff"));
        assert!(TokenType::is_hex_int("0xffffffffffffffffffffffffffffffff"));

        assert!(!TokenType::is_hex_int("0x"));
        assert!(!TokenType::is_hex_int("0X42069FFFff"));
        assert!(!TokenType::is_hex_int("12345"));
        assert!(!TokenType::is_hex_int("0xFFFFFG"));
    }

    #[test]
    fn octal_is_correct() {
        assert!(TokenType::is_oct_int("0o01234567"));
        assert!(TokenType::is_oct_int("0o777"));

        assert!(!TokenType::is_oct_int("0o"));
        assert!(!TokenType::is_oct_int("0O777"));
        assert!(!TokenType::is_oct_int("12345"));
        assert!(!TokenType::is_oct_int("0o7778"));
    }

    #[test]
    fn binary_is_correct() {
        assert!(TokenType::is_bin_int("0b000101010010101010101"));
        assert!(TokenType::is_bin_int("0b1010"));

        assert!(!TokenType::is_bin_int("0b"));
        assert!(!TokenType::is_bin_int("0B1010"));
        assert!(!TokenType::is_bin_int("12345"));
        assert!(!TokenType::is_bin_int("0b10102"));
    }

    #[test]
    fn decimal_is_correct() {
        assert!(TokenType::is_dec_int("1234"));
        assert!(TokenType::is_dec_int("01234"));
        assert!(TokenType::is_dec_int("-42"));

        assert!(!TokenType::is_dec_int("42.0"));
        assert!(!TokenType::is_dec_int("42a"));
    }

    #[test]
    fn float_is_correct() {
        assert!(TokenType::is_float("12.34"));
        assert!(TokenType::is_float("01234.00"));
        assert!(TokenType::is_float("-42.060"));
        assert!(TokenType::is_float("-64."));

        assert!(!TokenType::is_float("42"));
        assert!(!TokenType::is_float("42.0a"));
        assert!(!TokenType::is_float("42a.0"));
        assert!(!TokenType::is_float("42.4.2"));
        assert!(!TokenType::is_float(".0"));
    }

    #[test]
    fn identifier_is_correct() {
        assert!(TokenType::is_identifier("abc"));
        assert!(TokenType::is_identifier("abc_def"));
        assert!(TokenType::is_identifier("_abc_"));
        assert!(TokenType::is_identifier("abc23"));
        assert!(TokenType::is_identifier("_"));

        assert!(!TokenType::is_identifier("23abc"));
        assert!(!TokenType::is_identifier("abc-23"));
    }
}
