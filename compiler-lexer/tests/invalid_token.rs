const SOURCE: &str = r#"func abc() {
    call(42)
    »
}
"#;

#[test]
#[should_panic]
fn lexer_passes() {
    compiler_lexer::tokenize(SOURCE)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
}
