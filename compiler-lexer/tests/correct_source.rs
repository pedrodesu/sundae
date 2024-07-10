use compiler_lexer::definitions::{LiteralType::*, Token, TokenType::*};
use pretty_assertions::assert_eq;

const SOURCE: &str = r#"func function() {
    val value mut = 42 // comment
    val float f64 = 2.45
    val spec u8 = 0b010
    val a_rune rune
    val a_str []rune = "bruh"
    
    call(number)
}

// this is another comment"#;

#[test]
fn lexer_passes() {
    assert_eq!(
        compiler_lexer::tokenize(SOURCE)
            .collect::<Result<Vec<_>, _>>()
            .unwrap(),
        vec![
            Token {
                value: "func".into(),
                r#type: Keyword,
            },
            Token {
                value: "function".into(),
                r#type: Identifier,
            },
            Token {
                value: "(".into(),
                r#type: Separator,
            },
            Token {
                value: ")".into(),
                r#type: Separator,
            },
            Token {
                value: "{".into(),
                r#type: Separator,
            },
            Token {
                value: "\n".into(),
                r#type: Newline,
            },
            Token {
                value: "val".into(),
                r#type: Keyword,
            },
            Token {
                value: "value".into(),
                r#type: Identifier,
            },
            Token {
                value: "mut".into(),
                r#type: Keyword,
            },
            Token {
                value: "=".into(),
                r#type: Separator,
            },
            Token {
                value: "42".into(),
                r#type: Literal(Int),
            },
            Token {
                value: "// comment".into(),
                r#type: Comment,
            },
            Token {
                value: "\n".into(),
                r#type: Newline,
            },
            Token {
                value: "val".into(),
                r#type: Keyword,
            },
            Token {
                value: "float".into(),
                r#type: Identifier,
            },
            Token {
                value: "f64".into(),
                r#type: Identifier,
            },
            Token {
                value: "=".into(),
                r#type: Separator,
            },
            Token {
                value: "2.45".into(),
                r#type: Literal(Float),
            },
            Token {
                value: "\n".into(),
                r#type: Newline,
            },
            Token {
                value: "val".into(),
                r#type: Keyword,
            },
            Token {
                value: "spec".into(),
                r#type: Identifier,
            },
            Token {
                value: "u8".into(),
                r#type: Identifier,
            },
            Token {
                value: "=".into(),
                r#type: Separator,
            },
            Token {
                value: "0b010".into(),
                r#type: Literal(Int),
            },
            Token {
                value: "\n".into(),
                r#type: Newline,
            },
            Token {
                value: "val".into(),
                r#type: Keyword,
            },
            Token {
                value: "a_rune".into(),
                r#type: Identifier,
            },
            Token {
                value: "rune".into(),
                r#type: Identifier,
            },
            Token {
                value: "\n".into(),
                r#type: Newline,
            },
            Token {
                value: "val".into(),
                r#type: Keyword,
            },
            Token {
                value: "a_str".into(),
                r#type: Identifier,
            },
            Token {
                value: "[".into(),
                r#type: Separator,
            },
            Token {
                value: "]".into(),
                r#type: Separator,
            },
            Token {
                value: "rune".into(),
                r#type: Identifier,
            },
            Token {
                value: "=".into(),
                r#type: Separator,
            },
            Token {
                value: "\"bruh\"".into(),
                r#type: Literal(String),
            },
            Token {
                value: "\n".into(),
                r#type: Newline,
            },
            Token {
                value: "\n".into(),
                r#type: Newline,
            },
            Token {
                value: "call".into(),
                r#type: Identifier,
            },
            Token {
                value: "(".into(),
                r#type: Separator,
            },
            Token {
                value: "number".into(),
                r#type: Identifier,
            },
            Token {
                value: ")".into(),
                r#type: Separator,
            },
            Token {
                value: "\n".into(),
                r#type: Newline,
            },
            Token {
                value: "}".into(),
                r#type: Separator,
            },
            Token {
                value: "\n".into(),
                r#type: Newline,
            },
            Token {
                value: "\n".into(),
                r#type: Newline,
            },
            Token {
                value: "// this is another comment".into(),
                r#type: Comment,
            },
        ],
    );
}
