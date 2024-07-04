use compiler_lexer::definitions::{LiteralType::*, Token, TokenType::*};
use pretty_assertions::assert_eq;

const SOURCE: &str = r#"func function() {
    number mut := 42 // this is a comment
    float f64 := 2.45
    spec u8 := 0b010
    a_rune rune
    a_str []rune := "bruh"
    
    call(number)
}

// this is another comment"#;

#[test]
fn lexer_passes() {
    assert_eq!(
        compiler_lexer::tokenize(SOURCE)
            .unwrap()
            .collect::<Vec<_>>(),
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
                value: "number".into(),
                r#type: Identifier,
            },
            Token {
                value: "mut".into(),
                r#type: Keyword,
            },
            Token {
                value: ":=".into(),
                r#type: Operator,
            },
            Token {
                value: "42".into(),
                r#type: Literal(Int),
            },
            Token {
                value: "// this is a comment".into(),
                r#type: Comment,
            },
            Token {
                value: "\n".into(),
                r#type: Newline,
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
                value: ":=".into(),
                r#type: Operator,
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
                value: "spec".into(),
                r#type: Identifier,
            },
            Token {
                value: "u8".into(),
                r#type: Identifier,
            },
            Token {
                value: ":=".into(),
                r#type: Operator,
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
                value: ":=".into(),
                r#type: Operator,
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
