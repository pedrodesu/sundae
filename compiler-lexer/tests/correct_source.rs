use compiler_lexer::definitions::{LiteralType::*, Span, Token, TokenType::*};
use pretty_assertions::assert_eq;

const SOURCE: &str = r#"func function() {
    let value mut = 42 // comment
    let float f64 = 2.45
    let spec u8 = 0b010
    let a_rune rune
    let a_str []rune = "bruh"
    
    call(number)
}

// this is another comment"#;

#[test]
fn lexer_passes()
{
    assert_eq!(
        compiler_lexer::tokenize(SOURCE).collect::<Result<Vec<_>, _>>(),
        Ok(vec![
            Token {
                value: "func".into(),
                r#type: Keyword,
                span: Span {
                    from: (0, 0),
                    to: (0, 3)
                }
            },
            Token {
                value: "function".into(),
                r#type: Identifier,
                span: Span {
                    from: (0, 5),
                    to: (0, 12)
                }
            },
            Token {
                value: "(".into(),
                r#type: Separator,
                span: Span {
                    from: (0, 13),
                    to: (0, 13)
                }
            },
            Token {
                value: ")".into(),
                r#type: Separator,
                span: Span {
                    from: (0, 14),
                    to: (0, 14)
                }
            },
            Token {
                value: "{".into(),
                r#type: Separator,
                span: Span {
                    from: (0, 16),
                    to: (0, 16)
                }
            },
            Token {
                value: "\n".into(),
                r#type: Newline,
                span: Span {
                    from: (0, 17),
                    to: (0, 17)
                }
            },
            Token {
                value: "let".into(),
                r#type: Keyword,
                span: Span {
                    from: (1, 4),
                    to: (1, 6)
                }
            },
            Token {
                value: "value".into(),
                r#type: Identifier,
                span: Span {
                    from: (1, 8),
                    to: (1, 12)
                }
            },
            Token {
                value: "mut".into(),
                r#type: Keyword,
                span: Span {
                    from: (1, 14),
                    to: (1, 16)
                }
            },
            Token {
                value: "=".into(),
                r#type: Separator,
                span: Span {
                    from: (1, 18),
                    to: (1, 18)
                }
            },
            Token {
                value: "42".into(),
                r#type: Literal(Int),
                span: Span {
                    from: (1, 20),
                    to: (1, 21)
                }
            },
            Token {
                value: "// comment".into(),
                r#type: Comment,
                span: Span {
                    from: (1, 23),
                    to: (1, 32)
                }
            },
            Token {
                value: "\n".into(),
                r#type: Newline,
                span: Span {
                    from: (1, 33),
                    to: (1, 33)
                }
            },
            Token {
                value: "let".into(),
                r#type: Keyword,
                span: Span {
                    from: (2, 4),
                    to: (2, 6)
                }
            },
            Token {
                value: "float".into(),
                r#type: Identifier,
                span: Span {
                    from: (2, 8),
                    to: (2, 12)
                }
            },
            Token {
                value: "f64".into(),
                r#type: Identifier,
                span: Span {
                    from: (2, 14),
                    to: (2, 16)
                }
            },
            Token {
                value: "=".into(),
                r#type: Separator,
                span: Span {
                    from: (2, 18),
                    to: (2, 18)
                }
            },
            Token {
                value: "2.45".into(),
                r#type: Literal(Float),
                span: Span {
                    from: (2, 20),
                    to: (2, 23)
                }
            },
            Token {
                value: "\n".into(),
                r#type: Newline,
                span: Span {
                    from: (2, 24),
                    to: (2, 24)
                }
            },
            Token {
                value: "let".into(),
                r#type: Keyword,
                span: Span {
                    from: (3, 4),
                    to: (3, 6)
                }
            },
            Token {
                value: "spec".into(),
                r#type: Identifier,
                span: Span {
                    from: (3, 8),
                    to: (3, 11)
                }
            },
            Token {
                value: "u8".into(),
                r#type: Identifier,
                span: Span {
                    from: (3, 13),
                    to: (3, 14)
                }
            },
            Token {
                value: "=".into(),
                r#type: Separator,
                span: Span {
                    from: (3, 16),
                    to: (3, 16)
                }
            },
            Token {
                value: "0b010".into(),
                r#type: Literal(Int),
                span: Span {
                    from: (3, 18),
                    to: (3, 22)
                }
            },
            Token {
                value: "\n".into(),
                r#type: Newline,
                span: Span {
                    from: (3, 23),
                    to: (3, 23)
                }
            },
            Token {
                value: "let".into(),
                r#type: Keyword,
                span: Span {
                    from: (4, 4),
                    to: (4, 6)
                }
            },
            Token {
                value: "a_rune".into(),
                r#type: Identifier,
                span: Span {
                    from: (4, 8),
                    to: (4, 13)
                }
            },
            Token {
                value: "rune".into(),
                r#type: Identifier,
                span: Span {
                    from: (4, 15),
                    to: (4, 18)
                }
            },
            Token {
                value: "\n".into(),
                r#type: Newline,
                span: Span {
                    from: (4, 19),
                    to: (4, 19)
                }
            },
            Token {
                value: "let".into(),
                r#type: Keyword,
                span: Span {
                    from: (5, 4),
                    to: (5, 6)
                }
            },
            Token {
                value: "a_str".into(),
                r#type: Identifier,
                span: Span {
                    from: (5, 8),
                    to: (5, 12)
                }
            },
            Token {
                value: "[".into(),
                r#type: Separator,
                span: Span {
                    from: (5, 14),
                    to: (5, 14)
                }
            },
            Token {
                value: "]".into(),
                r#type: Separator,
                span: Span {
                    from: (5, 15),
                    to: (5, 15)
                }
            },
            Token {
                value: "rune".into(),
                r#type: Identifier,
                span: Span {
                    from: (5, 16),
                    to: (5, 19)
                }
            },
            Token {
                value: "=".into(),
                r#type: Separator,
                span: Span {
                    from: (5, 21),
                    to: (5, 21)
                }
            },
            Token {
                value: "\"bruh\"".into(),
                r#type: Literal(String),
                span: Span {
                    from: (5, 23),
                    to: (5, 28)
                }
            },
            Token {
                value: "\n".into(),
                r#type: Newline,
                span: Span {
                    from: (5, 29),
                    to: (5, 29)
                }
            },
            Token {
                value: "\n".into(),
                r#type: Newline,
                span: Span {
                    from: (6, 4),
                    to: (6, 4)
                }
            },
            Token {
                value: "call".into(),
                r#type: Identifier,
                span: Span {
                    from: (7, 4),
                    to: (7, 7)
                }
            },
            Token {
                value: "(".into(),
                r#type: Separator,
                span: Span {
                    from: (7, 8),
                    to: (7, 8)
                }
            },
            Token {
                value: "number".into(),
                r#type: Identifier,
                span: Span {
                    from: (7, 9),
                    to: (7, 14)
                }
            },
            Token {
                value: ")".into(),
                r#type: Separator,
                span: Span {
                    from: (7, 15),
                    to: (7, 15)
                }
            },
            Token {
                value: "\n".into(),
                r#type: Newline,
                span: Span {
                    from: (7, 16),
                    to: (7, 16)
                }
            },
            Token {
                value: "}".into(),
                r#type: Separator,
                span: Span {
                    from: (8, 0),
                    to: (8, 0)
                }
            },
            Token {
                value: "\n".into(),
                r#type: Newline,
                span: Span {
                    from: (8, 1),
                    to: (8, 1)
                }
            },
            Token {
                value: "\n".into(),
                r#type: Newline,
                span: Span {
                    from: (9, 0),
                    to: (9, 0)
                }
            },
            Token {
                value: "// this is another comment".into(),
                r#type: Comment,
                span: Span {
                    from: (10, 0),
                    to: (10, 25)
                }
            }
        ]),
    );
}
