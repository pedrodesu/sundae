use compiler_lexer::{
    LexerEvent,
    definitions::{LiteralType::*, TokenType::*},
};
use pretty_assertions::assert_eq;

const SOURCE: &[u8] = r#"func function() {
        let value = 42 // comment
        let float f64 = 2.45
        let spec u8 = 0b010
        let a_rune rune
        let a_str []rune = "bruh"

        call(number)
    }

    // this is another comment"#
    .as_bytes();

#[test]
fn lexer_passes() {
    let (tokens, errors) = compiler_lexer::tokenize(SOURCE).fold(
        (Vec::new(), Vec::new()),
        |(mut tokens, mut errors), e| {
            match e {
                LexerEvent::Token(t) => tokens.push((t.span.source(SOURCE), t.r#type)),
                LexerEvent::Error(err) => errors.push(err),
            }
            (tokens, errors)
        },
    );

    assert_eq!(
        tokens,
        [
            (b"func" as &[u8], Keyword),
            (b"function", Identifier),
            (b"(", Separator),
            (b")", Separator),
            (b"{", Separator),
            (b"\n", Newline),
            (b"let", Keyword),
            (b"value", Identifier),
            (b"=", Separator),
            (b"42", Literal(Int)),
            (b"// comment", Comment),
            (b"\n", Newline),
            (b"let", Keyword),
            (b"float", Identifier),
            (b"f64", Identifier),
            (b"=", Separator),
            (b"2.45", Literal(Float)),
            (b"\n", Newline),
            (b"let", Keyword),
            (b"spec", Identifier),
            (b"u8", Identifier),
            (b"=", Separator),
            (b"0b010", Literal(Int)),
            (b"\n", Newline),
            (b"let", Keyword),
            (b"a_rune", Identifier),
            (b"rune", Identifier),
            (b"\n", Newline),
            (b"let", Keyword),
            (b"a_str", Identifier),
            (b"[", Separator),
            (b"]", Separator),
            (b"rune", Identifier),
            (b"=", Separator),
            (b"\"bruh\"", Literal(String)),
            (b"\n", Newline),
            (b"\n", Newline),
            (b"call", Identifier),
            (b"(", Separator),
            (b"number", Identifier),
            (b")", Separator),
            (b"\n", Newline),
            (b"}", Separator),
            (b"\n", Newline),
            (b"\n", Newline),
            (b"// this is another comment", Comment)
        ]
    );

    assert_eq!(errors, []);
}
