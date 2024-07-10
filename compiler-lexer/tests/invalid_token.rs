const SOURCE: &str = r#"func abc() {
    call(42)
    »
}
"#;

#[test]
fn lexer_passes() {
    assert!(compiler_lexer::tokenize(SOURCE)
        .collect::<Result<Vec<_>, _>>()
        .is_err())
}
