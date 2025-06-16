use compiler_lexer::definitions::LiteralType;
use compiler_parser::{AST, Expression, Item, Name, Statement, Type, item::FunctionSignature};
use pretty_assertions::assert_eq;

const SOURCE: &str = r#"func function() {
    val value mut = 42
    val float f64 = 2.45
    val spec u8 = 0b010
    val a_rune rune
    val a_str []rune = "bruh"

    call(number)
}
"#;

#[test]
fn parser_passes()
{
    assert_eq!(
        compiler_parser::parse(compiler_lexer::tokenize(SOURCE).flatten()),
        Ok(AST(vec![Item::Function {
            signature: FunctionSignature {
                name: ("function".into(), None),
                arguments: vec![].into()
            },
            body: vec![
                Statement::Local {
                    name: Name("value".into(), None),
                    mutable: true,
                    init: Some(Expression::Literal {
                        value: "42".into(),
                        r#type: LiteralType::Int
                    })
                },
                Statement::Local {
                    name: Name("float".into(), Some(Type(vec!["f64".into()]))),
                    mutable: false,
                    init: Some(Expression::Literal {
                        value: "2.45".into(),
                        r#type: LiteralType::Float
                    })
                },
                Statement::Local {
                    name: Name("spec".into(), Some(Type(vec!["u8".into()]))),
                    mutable: false,
                    init: Some(Expression::Literal {
                        value: "0b010".into(),
                        r#type: LiteralType::Int
                    })
                },
                Statement::Local {
                    name: Name("a_rune".into(), Some(Type(vec!["rune".into()]))),
                    mutable: false,
                    init: None
                },
                Statement::Local {
                    name: Name(
                        "a_str".into(),
                        Some(Type(vec!["[".into(), "]".into(), "rune".into()]))
                    ),
                    mutable: false,
                    init: Some(Expression::Literal {
                        value: "\"bruh\"".into(),
                        r#type: LiteralType::String
                    })
                },
                Statement::Expression(Expression::Call {
                    path: vec!["call".into()].into(),
                    args: vec![Expression::Path(vec!["number".into()].into())].into()
                })
            ]
            .into()
        }]))
    );
}
