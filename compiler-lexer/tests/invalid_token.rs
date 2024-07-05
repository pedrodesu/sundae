const SOURCE: &str = r#"func abc() {
    call(42)
    »
}
"#;

#[test]
fn lexer_passes() {
    compiler_lexer::tokenize(SOURCE)
        .collect::<Result<Vec<_>, _>>()
        .unwrap_err();
}
