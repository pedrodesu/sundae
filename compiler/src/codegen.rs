use std::collections::HashMap;

use inkwell::{builder::Builder, context::Context, module::Module};

use crate::parser::AST;

use anyhow::Result;

use self::types::Type;

mod expression;
mod item;
mod statement;
mod types;

struct Codegen<'ctx> {
    ctx: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    functions: HashMap<String, Type>,
}

pub fn gen(module: &str, ast: AST) -> Result<()> {
    let ctx = Context::create();

    let mut codegen = Codegen {
        ctx: &ctx,
        module: ctx.create_module(module),
        builder: ctx.create_builder(),
        functions: HashMap::new(),
    };

    // TODO how to extern whole lib
    // you don't, just extern everything necessary in lib source code and then call it on my exe?
    codegen.module.add_function(
        "putd",
        codegen
            .ctx
            .void_type()
            .fn_type(&[codegen.ctx.i32_type().into()], false),
        Some(inkwell::module::Linkage::External),
    );

    for item in ast.0 {
        codegen.gen_item(item)?;
    }

    codegen
        .module
        .print_to_file(format!("{module}.ll"))
        .unwrap();

    Ok(())
}

// take some concepts of iconic:
// propagate errors so as to stop/yield execution
// binary operations succeed/fail and return rhs

/*

if -2 < -1 < 0 {

}

this executes, if condition will be sequently evaluated to a 'true' (if block)

//

let i = 10
i = i < find(pat, str)

this executes, i will equal to find output (assign)

*/
