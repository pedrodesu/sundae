use compiler_lexer::{
    LexerError, LexerEvent,
    definitions::{LiteralType::*, TokenType::*},
};
use pretty_assertions::assert_eq;

const SOURCE: &[u8] = r#"func abc() {
        call(42)

        "
    }
    "#
.as_bytes();

#[test]
fn unclosed_delim() {
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
            (b"abc", Identifier),
            (b"(", Separator),
            (b")", Separator),
            (b"{", Separator),
            (b"\n", Newline),
            (b"call", Identifier),
            (b"(", Separator),
            (b"42", Literal(Int)),
            (b")", Separator),
            (b"\n", Newline),
            (b"\n", Newline),
        ]
    );

    assert_eq!(
        errors,
        [LexerError::UnclosedDelim {
            delim: b'"',
            span: (39..=50).into(),
        }]
    );
}
