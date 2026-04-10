use compiler_lexer::{
    LexerError, LexerEvent,
    definitions::{LiteralType::*, TokenType::*},
};
use itertools::{Either, Itertools};
use pretty_assertions::assert_eq;

const SOURCE: &str = r#"func abc() {
        call(42)

        "
    }
    "#;

#[test]
fn unclosed_delim()
{
    let (tokens, errors) = compiler_lexer::tokenize(SOURCE)
        .partition_map::<Vec<_>, Vec<_>, _, _, _>(|e| match e
        {
            LexerEvent::Token(token) => Either::Left((token.span.source(SOURCE), token.r#type)),
            LexerEvent::Error(error) => Either::Right(error),
        });

    assert_eq!(
        tokens,
        [
            ("func", Keyword),
            ("abc", Identifier),
            ("(", Separator),
            (")", Separator),
            ("{", Separator),
            ("\n", Newline),
            ("call", Identifier),
            ("(", Separator),
            ("42", Literal(Int)),
            (")", Separator),
            ("\n", Newline),
            ("\n", Newline),
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
