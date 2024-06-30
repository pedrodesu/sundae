const SOURCE: &str = r#"func abc() {
    call(42)
    »
}
"#;

#[test]
fn lexer_passes() {
    compiler_lexer::tokenize(SOURCE).unwrap_err();
}
