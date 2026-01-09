#![feature(assert_matches)]

mod tests
{
    use std::assert_matches::assert_matches;

    use compiler_lexer::LexerError;

    const SOURCE: &str = r#"func abc() {
        call(42)
        »
    }
    "#;

    #[test]
    fn lexer_passes()
    {
        let errors = compiler_lexer::tokenize(SOURCE).filter_map(Result::err);

        assert_matches!(
            errors.collect::<Vec<_>>().as_slice(),
            [LexerError::UnknownTokenStart {
                token: '»',
                span: s,
                ..
            }] if *s == 38.into()
        );
    }
}
