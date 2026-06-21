use compiler_lexer::{LexerEvent, definitions::LiteralType};
use compiler_parser::{AST, Expression, Item, Name, Statement, Type, item::FunctionSignature};
use pretty_assertions::assert_eq;

const SOURCE: &str = r#"func function() {
    let value mut = 42
    let float f64 = 2.45
    let spec u8 = 0b010
    let a_rune rune
    let a_str []rune = "bruh"

    call(number)
}
"#;

#[test]
fn parser_passes() {
    assert_eq!(
        compiler_parser::parse(
            SOURCE,
            compiler_lexer::tokenize(SOURCE).filter_map(|event| match event {
                LexerEvent::Token(token) => Some(token),
                LexerEvent::Error(_) => None,
            })
        ),
        Ok(AST([Item::Function {
            signature: FunctionSignature {
                name: (b"function", None),
                arguments: [].into()
            },
            body: [
                Statement::Local {
                    name: Name(b"value", None),
                    mutable: true,
                    init: Some(Expression::Literal {
                        value: b"42",
                        r#type: LiteralType::Int
                    })
                },
                Statement::Local {
                    name: Name(b"float", Some(Type([b"f64" as _].into()))),
                    mutable: false,
                    init: Some(Expression::Literal {
                        value: b"2.45",
                        r#type: LiteralType::Float
                    })
                },
                Statement::Local {
                    name: Name(b"spec", Some(Type([b"u8" as _].into()))),
                    mutable: false,
                    init: Some(Expression::Literal {
                        value: b"0b010",
                        r#type: LiteralType::Int
                    })
                },
                Statement::Local {
                    name: Name(b"a_rune", Some(Type([b"rune" as _].into()))),
                    mutable: false,
                    init: None
                },
                Statement::Local {
                    name: Name(b"a_str", Some(Type([b"[" as &[_], b"]", b"rune"].into()))),
                    mutable: false,
                    init: Some(Expression::Literal {
                        value: b"\"bruh\"",
                        r#type: LiteralType::String
                    })
                },
                Statement::Expression(Expression::Call {
                    path: [b"call" as _].into(),
                    args: [Expression::Path([b"number" as _].into())].into()
                })
            ]
            .into()
        }]
        .into()))
    );
}
