#![feature(assert_matches)]

#[cfg(test)]
mod tests
{
    use std::assert_matches::assert_matches;

    use compiler_lexer::LexerError;

    const SOURCE: &str = r#"func abc() {
        call(42)

        "
    }
    "#;

    #[test]
    fn lexer_passes()
    {
        let errors = compiler_lexer::tokenize(SOURCE).filter_map(Result::err);

        assert_matches!(
            errors.collect::<Vec<_>>().as_slice(),
            [LexerError::UnclosedDelim { delim: b'"', span: s, .. }] if *s == (39..51).into()
        );
    }
}
