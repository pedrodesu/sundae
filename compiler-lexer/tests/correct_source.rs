use compiler_lexer::{
    LexerEvent,
    definitions::{LiteralType::*, TokenType::*},
};
use itertools::{Either, Itertools};
use pretty_assertions::assert_eq;

const SOURCE: &str = r#"func function() {
        let value = 42 // comment
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
            ("function", Identifier),
            ("(", Separator),
            (")", Separator),
            ("{", Separator),
            ("\n", Newline),
            ("let", Keyword),
            ("value", Identifier),
            ("=", Separator),
            ("42", Literal(Int)),
            ("// comment", Comment),
            ("\n", Newline),
            ("let", Keyword),
            ("float", Identifier),
            ("f64", Identifier),
            ("=", Separator),
            ("2.45", Literal(Float)),
            ("\n", Newline),
            ("let", Keyword),
            ("spec", Identifier),
            ("u8", Identifier),
            ("=", Separator),
            ("0b010", Literal(Int)),
            ("\n", Newline),
            ("let", Keyword),
            ("a_rune", Identifier),
            ("rune", Identifier),
            ("\n", Newline),
            ("let", Keyword),
            ("a_str", Identifier),
            ("[", Separator),
            ("]", Separator),
            ("rune", Identifier),
            ("=", Separator),
            ("\"bruh\"", Literal(String)),
            ("\n", Newline),
            ("\n", Newline),
            ("call", Identifier),
            ("(", Separator),
            ("number", Identifier),
            (")", Separator),
            ("\n", Newline),
            ("}", Separator),
            ("\n", Newline),
            ("\n", Newline),
            ("// this is another comment", Comment)
        ]
    );

    assert_eq!(errors, []);
}
