use ecow::EcoString;

const KEYWORDS: &[&str] = &["const", "func", "ret", "if", "mut", "let"];

const OPERATORS: &[&str] = &[
    "and", "or", "+", "-", "*", "/", "+=", "-=", "*=", "/=", "<", ">", "<=", ">=", "==", "!", "!=",
    "<<", ">>", "<<=", ">>=", "&", "|", "^", "&=", "|=", "^=",
];

const SEPARATORS: &[&str] = &["=", "(", ")", "[", "]", "{", "}", ",", "."];

pub const STR_DELIM: char = '"';
pub const RUNE_DELIM: char = '`';
pub const COMMENT_PREFIX: &str = "//";
pub const COMMENT_PREFIX_LEN: usize = COMMENT_PREFIX.len();

pub const HEX_PREFIX: &str = "0x";
pub const OCT_PREFIX: &str = "0o";
pub const BIN_PREFIX: &str = "0b";

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum LiteralType
{
    String,
    Rune,
    Int,
    Float,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TokenType
{
    Keyword,
    Identifier,
    Operator,
    Literal(LiteralType),
    Separator,
    Comment,
    Newline,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Span
{
    pub from: (usize, usize),
    pub to: (usize, usize),
}

#[derive(PartialEq, Debug, Clone)]
pub struct Token
{
    pub value: EcoString,
    pub r#type: TokenType,
    pub span: Span,
}

impl TokenType
{
    #[inline]
    fn is_special_fmt_int(
        expression: &str,
        prefix: &str,
        char_predicate: impl Fn(char) -> bool,
    ) -> bool
    {
        if expression.len() <= prefix.len()
        {
            false
        }
        else
        {
            let Some((p, rem)) = expression.split_at_checked(prefix.len())
            else
            {
                return false;
            };

            p == prefix && rem.chars().all(char_predicate)
        }
    }

    #[inline]
    fn is_dec_int(expression: &str) -> bool
    {
        expression.chars().all(|c| c.is_ascii_digit())
    }

    #[inline]
    fn is_hex_int(expression: &str) -> bool
    {
        Self::is_special_fmt_int(expression, HEX_PREFIX, |c| c.is_ascii_hexdigit())
    }

    #[inline]
    fn is_oct_int(expression: &str) -> bool
    {
        Self::is_special_fmt_int(expression, OCT_PREFIX, |c| matches!(c, '0'..='7'))
    }

    #[inline]
    fn is_bin_int(expression: &str) -> bool
    {
        Self::is_special_fmt_int(expression, BIN_PREFIX, |c| matches!(c, '0' | '1'))
    }

    #[inline]
    fn is_float(expression: &str) -> bool
    {
        expression
            .chars()
            .all(|c| matches!(c, '0'..='9' | '-' | '.'))
            && (expression.matches('.').count() == 1 && !expression.starts_with('.'))
    }

    #[inline]
    fn is_identifier(expression: &str) -> bool
    {
        expression
            .chars()
            .all(|c| matches!(c, '_' | 'a'..='z' | 'A'..='Z' | '0'..='9'))
            && expression.starts_with(|c: char| !c.is_ascii_digit())
    }

    #[inline]
    pub fn eval(expr: &str) -> Option<Self>
    {
        if KEYWORDS.contains(&expr)
        {
            Some(TokenType::Keyword)
        }
        else if Self::is_identifier(expr)
        {
            Some(TokenType::Identifier)
        }
        else if Self::is_hex_int(expr)
            || Self::is_dec_int(expr)
            || Self::is_oct_int(expr)
            || Self::is_bin_int(expr)
        {
            Some(TokenType::Literal(LiteralType::Int))
        }
        else if Self::is_float(expr)
        {
            Some(TokenType::Literal(LiteralType::Float))
        }
        else if OPERATORS.contains(&expr)
        {
            Some(TokenType::Operator)
        }
        else if SEPARATORS.contains(&expr)
        {
            Some(TokenType::Separator)
        }
        else if expr == "\n"
        {
            Some(TokenType::Newline)
        }
        else
        {
            None
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn hexadecimal_passes()
    {
        assert!(TokenType::is_hex_int("0x42069FFFff"));
        assert!(TokenType::is_hex_int("0xffffffffffffffffffffffffffffffff"));
        assert!(TokenType::is_hex_int("0xDEADBEEF"));

        assert!(!TokenType::is_hex_int("0x"));
        assert!(!TokenType::is_hex_int("0X42069FFFff"));
        assert!(!TokenType::is_hex_int("12345"));
        assert!(!TokenType::is_hex_int("0xFFFFFG"));
    }

    #[test]
    fn octal_passes()
    {
        assert!(TokenType::is_oct_int("0o01234567"));
        assert!(TokenType::is_oct_int("0o777"));

        assert!(!TokenType::is_oct_int("0o"));
        assert!(!TokenType::is_oct_int("0O777"));
        assert!(!TokenType::is_oct_int("12345"));
        assert!(!TokenType::is_oct_int("0o7778"));
    }

    #[test]
    fn binary_passes()
    {
        assert!(TokenType::is_bin_int("0b000101010010101010101"));
        assert!(TokenType::is_bin_int("0b1010"));

        assert!(!TokenType::is_bin_int("0b"));
        assert!(!TokenType::is_bin_int("0B1010"));
        assert!(!TokenType::is_bin_int("12345"));
        assert!(!TokenType::is_bin_int("0b10102"));
    }

    #[test]
    fn decimal_passes()
    {
        assert!(TokenType::is_dec_int("0"));
        assert!(TokenType::is_dec_int("00"));
        assert!(TokenType::is_dec_int("01234"));
        assert!(TokenType::is_dec_int("1234"));

        assert!(!TokenType::is_dec_int("42.0"));
        assert!(!TokenType::is_dec_int("42a"));
    }

    #[test]
    fn float_passes()
    {
        assert!(TokenType::is_float("12.34"));
        assert!(TokenType::is_float("64."));
        assert!(TokenType::is_float("00."));
        assert!(TokenType::is_float("01234.00"));
        assert!(TokenType::is_float("42.060"));

        assert!(!TokenType::is_float("42"));
        assert!(!TokenType::is_float("42.0a"));
        assert!(!TokenType::is_float("42a.0"));
        assert!(!TokenType::is_float("42.4.2"));
        assert!(!TokenType::is_float(".0"));
    }

    #[test]
    fn identifier_passes()
    {
        assert!(TokenType::is_identifier("abc"));
        assert!(TokenType::is_identifier("abc_def"));
        assert!(TokenType::is_identifier("_abc_"));
        assert!(TokenType::is_identifier("abc23"));
        assert!(TokenType::is_identifier("_"));

        assert!(!TokenType::is_identifier("23abc"));
        assert!(!TokenType::is_identifier("abc-23"));
    }
}
