#![feature(box_patterns)]
#![feature(let_chains)]

use anyhow::{anyhow, bail, ensure, Result};
use compiler_parser::{Type as ParserType, AST};
use ecow::EcoString;
use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
    types::{AnyType, AnyTypeEnum, BasicType, BasicTypeEnum},
    values::{BasicValueEnum, FunctionValue},
    OptimizationLevel,
};
use std::{cell::RefCell, collections::HashMap, fmt, fs, path::PathBuf, process::Command, rc::Rc};

mod expression;
mod item;
mod statement;

#[derive(Clone, PartialEq, Debug)]
pub enum Type {
    Integer { width: u32, signed: bool },
    Float(u32),
    Void,
    Array { scalar: Box<Type>, size: u32 },
    Ref(Box<Type>),
    MutRef(Box<Type>),
}

impl Default for Type {
    #[inline]
    fn default() -> Self {
        Self::Void
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Integer { width, signed } => {
                write!(f, "{}{width}", if *signed { "i" } else { "u" })
            }
            Type::Float(width) => write!(f, "f{width}"),
            Type::Void => write!(f, "()"),
            Type::Array { box scalar, size } => write!(f, "[{size}]{scalar}"),
            Type::Ref(box v) => write!(f, "&{v}"),
            Type::MutRef(box v) => write!(f, "&mut {v}"),
        }
    }
}

impl TryFrom<ParserType> for Type {
    type Error = anyhow::Error;

    fn try_from(value: ParserType) -> Result<Self, Self::Error> {
        if let [a, b, c @ ..] = value.0.as_slice()
            && let ["&", "mut"] = [a.as_str(), b.as_str()]
        {
            return Ok(Self::MutRef(Box::new(ParserType(c.to_vec()).try_into()?)));
        } else if let [a, b @ ..] = value.0.as_slice()
            && let ["&"] = [a.as_str()]
        {
            return Ok(Self::Ref(Box::new(ParserType(b.to_vec()).try_into()?)));
        } else {
            let r#type = &value.0[0]; // TODO assume for now, modify when we have structs (and fields, thus)

            let (signedness, bits) = r#type.split_at(1);
            if matches!(signedness, "u" | "i") {
                Ok(Self::Integer {
                    width: bits
                        .parse()
                        .map_err(|_| anyhow!("Integer with invalid width"))?,
                    signed: signedness == "i",
                })
            } else {
                match r#type.as_str() {
                    "f16" => Ok(Self::Float(16)),
                    "f32" => Ok(Self::Float(32)),
                    "f64" => Ok(Self::Float(64)),
                    "f128" => Ok(Self::Float(128)),
                    "void" => Ok(Self::Void),
                    _ => Err(anyhow!("Unknown type")),
                }
            }
        }
    }
}

impl Type {
    #[inline]
    pub fn as_llvm_any_type<'ctx>(&self, ctx: &'ctx Context) -> AnyTypeEnum<'ctx> {
        match self.as_llvm_basic_type(ctx) {
            Ok(t) => t.as_any_type_enum(),
            Err(_) => match self {
                Type::Void => ctx.void_type().into(),
                _ => unreachable!(),
            },
        }
    }

    #[inline]
    pub fn as_llvm_basic_type<'ctx>(&self, ctx: &'ctx Context) -> Result<BasicTypeEnum<'ctx>> {
        match self {
            Self::Integer { width, .. } => Ok(match width {
                8 => ctx.i8_type().into(),
                16 => ctx.i16_type().into(),
                32 => ctx.i32_type().into(),
                64 => ctx.i64_type().into(),
                128 => ctx.i128_type().into(),
                n => ctx.custom_width_int_type(*n).into(),
            }),
            Self::Float(width) => Ok(match width {
                16 => ctx.f16_type().into(),
                32 => ctx.f32_type().into(),
                64 => ctx.f64_type().into(),
                128 => ctx.f128_type().into(),
                _ => unreachable!(),
            }),
            Self::Array { box scalar, size } => {
                Ok(scalar.as_llvm_basic_type(ctx)?.array_type(*size).into())
            }
            Self::Ref(_) | Self::MutRef(_) => Ok(ctx.ptr_type(Default::default()).into()),
            _ => bail!("type {self:?} can't be converted to a basic type"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Value<'ctx> {
    pub r#type: Type,
    pub inner: BasicValueEnum<'ctx>,
}

/*
pub struct Local<'ctx> {
    mutable: bool,
    value: Value<'ctx>,
}
*/

#[derive(Clone, Debug)]
pub struct Function<'ctx> {
    pub arguments: Vec<(EcoString, Type)>,
    pub return_type: Type,
    pub stack: HashMap<EcoString, Value<'ctx>>,
    pub inner: FunctionValue<'ctx>,
}

impl Function<'_> {
    #[inline]
    pub fn init_block(&mut self, codegen: &Codegen) {
        let block = codegen.ctx.append_basic_block(self.inner, "entry");
        codegen.builder.position_at_end(block);
    }

    #[inline]
    pub fn init_args_stack(&mut self, codegen: &Codegen) -> Result<()> {
        self.arguments
            .clone()
            .into_iter()
            .zip(self.inner.get_param_iter())
            .try_for_each(|((name, r#type), arg)| {
                let ptr = codegen
                    .builder
                    .build_alloca(r#type.as_llvm_basic_type(&codegen.ctx)?, &name)?;

                codegen.builder.build_store(ptr, arg)?;

                self.stack.insert(name, Value { r#type, inner: arg });

                Ok(())
            })
    }
}

#[derive(Default)]
pub struct Runtime<'ctx> {
    pub functions: HashMap<EcoString, Rc<RefCell<Function<'ctx>>>>,
    pub constants: HashMap<EcoString, Value<'ctx>>,
}

pub struct Codegen<'ctx> {
    pub ctx: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub runtime: Rc<RefCell<Runtime<'ctx>>>,
}

pub fn gen(module: &str, ast: AST, ir: bool, output_path: Option<PathBuf>) -> Result<()> {
    let ctx = Context::create();

    let codegen = {
        let r#mod = ctx.create_module(module);
        let builder = ctx.create_builder();
        let runtime = Runtime::default();

        Codegen {
            ctx: &ctx,
            module: r#mod,
            builder,
            runtime: Rc::new(RefCell::new(runtime)),
        }
    };

    // TODO extern whole lib
    // https://llvm.org/docs/tutorial/MyFirstLanguageFrontend/LangImpl03.html
    Target::initialize_native(&InitializationConfig {
        asm_parser: false,
        asm_printer: true,
        base: true,
        disassembler: false,
        info: false,
        machine_code: false,
    })
    .map_err(|m| anyhow!("Couldn't initialise LLVM native target {m}"))?;

    let triple = TargetMachine::get_default_triple();

    let target = Target::from_triple(&triple)
        .map_err(|m| anyhow!("Couldn't create target from triple `{m}`"))?;

    let machine = target
        .create_target_machine(
            &triple,
            "generic",
            "",
            OptimizationLevel::None,
            RelocMode::Default,
            CodeModel::Default,
        )
        .unwrap();

    codegen.runtime.borrow_mut().functions.insert(
        "putd".into(),
        Rc::new(RefCell::new(Function {
            arguments: vec![(
                "n".into(),
                Type::Integer {
                    width: 32,
                    signed: true,
                },
            )],
            return_type: Type::Void,
            stack: Default::default(),
            inner: codegen.module.add_function(
                "putd",
                codegen
                    .ctx
                    .void_type()
                    .fn_type(&[codegen.ctx.i32_type().into()], false),
                None,
            ),
        })),
    );

    ast.0.into_iter().try_for_each(|i| codegen.gen_item(i))?;

    let output_path = {
        let base = output_path.unwrap_or_default();
        // hacky way to check for trailing slash because Components (https://doc.rust-lang.org/stable/std/path/struct.PathBuf.html#method.components)
        if base.as_os_str().is_empty()
            || base.to_string_lossy().ends_with(std::path::MAIN_SEPARATOR)
        {
            base.join(module)
        } else {
            base
        }
    };

    if let Some(root_path) = output_path.parent() {
        fs::create_dir_all(root_path)?;
    }

    if ir {
        let ir_path = output_path.with_extension("ll");

        codegen
            .module
            .print_to_file(ir_path)
            .map_err(|m| anyhow!("Couldn't output LLVM IR: {m}"))?;
    }

    let object_path = output_path.with_extension("o");

    machine
        .write_to_file(&codegen.module, FileType::Object, &object_path)
        .map_err(|m| anyhow!("Couldn't write to object file: {m}"))?;

    // See https://github.com/rust-lang/rust/blob/master/compiler/rustc_codegen_ssa/src/back/link.rs#L1280
    // order of priority on *nix cc -> lld -> ld
    // on msvc is lld -> link.exe
    {
        let output = Command::new("cc")
            .args([
                "-fuse-ld=mold",
                object_path.to_str().unwrap(),
                "-Ltarget/debug",
                "-lsundae_library",
                "-o",
                output_path.to_str().unwrap(),
            ])
            .output()?;

        if !output.stderr.is_empty() {
            Err(anyhow!(String::from_utf8(output.stderr).unwrap()))
        } else {
            Ok(())
        }
    }
    .map_err(|m| anyhow!("Error linking object file: {}", m))?;

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
