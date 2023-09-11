use std::{
    collections::HashMap,
    ffi::CString,
    ops::Deref,
    sync::{LazyLock, OnceLock},
};

use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::{AnyType, BasicType, BasicTypeEnum},
    values::IntValue,
};

use crate::{
    lexer::LiteralType,
    parser::{
        binary::{Node, Operator},
        Expression, Item, Name, Signature, Statement, AST,
    },
};

struct Codegen<'ctx> {
    ctx: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
}

// LLVM makes no difference between signed and unsigned types, only wants the amount of bits
mod r#type {
    use inkwell::types::{ArrayType, BasicType, FloatType, IntType, VoidType};

    use super::Codegen;

    // implement complex types, such as arrays and custom structs

    #[inline]
    fn int_type<'ctx>(value: &str, codegen: &Codegen<'ctx>) -> Option<IntType<'ctx>> {
        match value {
            "bool" => Some(codegen.ctx.bool_type()),
            "u8" | "i8" => Some(codegen.ctx.i8_type()),
            "u16" | "i16" => Some(codegen.ctx.i16_type()),
            "u32" | "i32" => Some(codegen.ctx.i32_type()),
            "u64" | "i64" => Some(codegen.ctx.i64_type()),
            "u128" | "i128" => Some(codegen.ctx.i128_type()),
            _ => {
                let (signedness, bits) = value.split_at(1);
                if matches!(signedness, "u" | "i") {
                    Some(codegen.ctx.custom_width_int_type(bits.parse::<u32>().ok()?))
                } else {
                    None
                }
            }
        }
    }

    #[inline]
    fn float_type<'ctx>(value: &str, codegen: &Codegen<'ctx>) -> Option<FloatType<'ctx>> {
        match value {
            "f16" => Some(codegen.ctx.f16_type()),
            "f32" => Some(codegen.ctx.f32_type()),
            "f64" => Some(codegen.ctx.f64_type()),
            "f128" => Some(codegen.ctx.f128_type()),
            _ => None,
        }
    }

    #[inline]
    fn array_type<'ctx>(value: &str, codegen: &Codegen<'ctx>) -> Option<ArrayType<'ctx>> {
        let rhs = value.find(']')?;
        let sub = value.get(value.find('[')? + 1..rhs - 1).unwrap();
        let size = sub.parse::<u32>().ok()?;

        let r#type = self::basic_type(value.get(rhs + 1..)?, codegen)?;

        Some(r#type.array_type(size))
    }

    #[inline]
    fn void_type<'ctx>(value: &str, codegen: &Codegen<'ctx>) -> Option<VoidType<'ctx>> {
        if value == "void" {
            Some(codegen.ctx.void_type())
        } else {
            None
        }
    }

    #[inline]
    pub fn basic_type<'ctx>(
        value: &str,
        codegen: &Codegen<'ctx>,
    ) -> Option<Box<dyn BasicType<'ctx> + 'ctx>> {
        // TODO can't use array as they have different types (return impl AnyType), find better way
        if let Some(t) = self::int_type(value, codegen) {
            Some(Box::new(t))
        } else if let Some(t) = self::float_type(value, codegen) {
            Some(Box::new(t))
        } else if let Some(t) = self::array_type(value, codegen) {
            Some(Box::new(t))
        } else {
            None
        }
    }
}

impl Item {
    fn gen(self, codegen: &Codegen) {
        match self {
            Item::Const { name, value } => {
                let r#type = r#type::basic_type(&name.1, codegen).unwrap();
                let r#const = codegen
                    .module
                    .add_global(r#type.as_basic_type_enum(), None, &name.0);

                let value = r#type.as_basic_type_enum().const_zero();

                // TODO find best way to store types.
                r#const.set_initializer(&value);

                /*
                r#type
                    .as_any_type_enum()
                    .into_int_type()
                    .const_int(42, true);
                */

                // r#const.set_initializer(&codegen.ctx.const_string(b"sex", false));
            }
            Item::Function { signature, body } => {
                //
            }
        }
    }
}

pub fn gen(module: &str, ast: AST) {
    let ctx = Context::create();

    let codegen = Codegen {
        ctx: &ctx,
        module: ctx.create_module(module),
        builder: ctx.create_builder(),
    };

    // add library?
    /*
    unsafe {
        println!(
            "{}",
            LLVMLoadLibraryPermanently("target/debug/libsundae_library.so\0".as_ptr() as _)
        );
        let func_type = unsafe {
            LLVMFunctionType(
                LLVMVoidType(),
                [LLVMPointerTypeInContext(*CTX, 4)].as_mut_ptr(),
                1,
                false as _,
            )
        };
        LLVMAddFunction(r#mod(), "println\0".as_ptr() as _, func_type);
    }
    */

    ast.0.into_iter().for_each(|i| Item::gen(i, &codegen));

    codegen.module.print_to_file("output.ll").unwrap();
}
