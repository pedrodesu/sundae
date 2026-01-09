mod tests
{
    use compiler_lexer::definitions::{LiteralType::*, TokenType::*};
    use miette::SourceSpan;
    use pretty_assertions::assert_eq;

    const SOURCE: &str = r#"func function() {
        let value = 42 // comment
        let float f64 = 2.45
        let spec u8 = 0b010
        let a_rune rune
        let a_str []rune = "bruh"

        call(number)
    }

    // this is another comment"#;

    #[inline]
    fn span_to_range(span: SourceSpan) -> std::ops::Range<usize>
    {
        let offset = span.offset();
        let len = span.len();
        offset..offset + len
    }

    #[test]
    fn lexer_passes()
    {
        let source = SOURCE.as_bytes();
        assert_eq!(
            compiler_lexer::tokenize(SOURCE)
                .map(|t| t.map(|t| (
                    std::str::from_utf8(&source[span_to_range(t.span)]).unwrap(),
                    t.r#type,
                )))
                .collect::<Result<Vec<_>, _>>(),
            Ok(vec![
                ("func", Keyword),
                ("function", Identifier),
                ("(", Separator),
                (")", Separator),
                ("{", Separator),
                ("\n", Newline),
                ("let", Keyword),
                ("value", Identifier),
                ("=", Separator),
                ("42", Literal(Int)),
                ("// comment", Comment),
                ("\n", Newline),
                ("let", Keyword),
                ("float", Identifier),
                ("f64", Identifier),
                ("=", Separator),
                ("2.45", Literal(Float)),
                ("\n", Newline),
                ("let", Keyword),
                ("spec", Identifier),
                ("u8", Identifier),
                ("=", Separator),
                ("0b010", Literal(Int)),
                ("\n", Newline),
                ("let", Keyword),
                ("a_rune", Identifier),
                ("rune", Identifier),
                ("\n", Newline),
                ("let", Keyword),
                ("a_str", Identifier),
                ("[", Separator),
                ("]", Separator),
                ("rune", Identifier),
                ("=", Separator),
                ("\"bruh\"", Literal(String)),
                ("\n", Newline),
                ("\n", Newline),
                ("call", Identifier),
                ("(", Separator),
                ("number", Identifier),
                (")", Separator),
                ("\n", Newline),
                ("}", Separator),
                ("\n", Newline),
                ("\n", Newline),
                ("// this is another comment", Comment)
            ]),
        );
    }
}
