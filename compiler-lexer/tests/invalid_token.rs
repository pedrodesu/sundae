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

    assert_eq!(
        errors.collect::<Vec<_>>(),
        [LexerError::InvalidToken {
            token: "»".to_owned(),
        }]
    );
}
