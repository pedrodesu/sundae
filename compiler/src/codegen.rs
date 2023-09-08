use std::{collections::HashMap, ffi::CString, ops::Deref};

use itertools::Itertools;
use llvm_sys::{
    core::*, support::LLVMAddSymbol, target::LLVM_InitializeNativeTarget, LLVMBuilder, LLVMContext,
    LLVMModule, LLVMRealPredicate, LLVMType, LLVMValue,
};

use crate::{
    lexer::LiteralType,
    parser::{
        binary::{Node, Operator},
        Expression, Item, Signature, Statement, AST,
    },
};

// LLVM makes no difference between signed and unsigned types, only wants the amount of bits
mod r#type {
    use llvm_sys::{core::*, LLVMContext, LLVMType};

    #[inline]
    fn get_fixed_int_type(ctx: *mut LLVMContext, value: &str) -> Option<*mut LLVMType> {
        unsafe {
            match value {
                "u8" | "i8" => Some(LLVMInt8TypeInContext(ctx)),
                "u16" | "i16" => Some(LLVMInt16TypeInContext(ctx)),
                "u32" | "i32" => Some(LLVMInt32TypeInContext(ctx)),
                "u64" | "i64" => Some(LLVMInt64TypeInContext(ctx)),
                "u128" | "i128" => Some(LLVMInt128TypeInContext(ctx)),
                _ => None,
            }
        }
    }

    #[inline]
    fn get_int_type(ctx: *mut LLVMContext, value: &str) -> Option<*mut LLVMType> {
        unsafe {
            if let Some(literal) = value.as_ascii() {
                let (signedness, bits) = (literal[0], literal[1..].as_str());
                if matches!(signedness.to_char(), 'u' | 'i') {
                    if let Ok(bits) = bits.parse::<u32>() {
                        return Some(LLVMIntTypeInContext(ctx, bits));
                    }
                }
            }
            None
        }
    }

    #[inline]
    fn get_bool_type(ctx: *mut LLVMContext, value: &str) -> Option<*mut LLVMType> {
        unsafe {
            if value == "bool" {
                Some(LLVMInt1TypeInContext(ctx))
            } else {
                None
            }
        }
    }

    #[inline]
    fn get_float_type(ctx: *mut LLVMContext, value: &str) -> Option<*mut LLVMType> {
        unsafe {
            match value {
                "f16" => Some(LLVMHalfTypeInContext(ctx)),
                "f32" => Some(LLVMFloatTypeInContext(ctx)),
                "f64" => Some(LLVMDoubleTypeInContext(ctx)),
                "f128" => Some(LLVMFP128TypeInContext(ctx)),
                _ => None,
            }
        }
    }

    #[inline]
    fn get_void_type(ctx: *mut LLVMContext, value: &str) -> Option<*mut LLVMType> {
        unsafe {
            if value == "void" {
                Some(LLVMVoidTypeInContext(ctx))
            } else {
                None
            }
        }
    }

    pub fn get_type(ctx: *mut LLVMContext, value: &str) -> Option<*mut LLVMType> {
        [
            self::get_fixed_int_type,
            self::get_int_type,
            self::get_bool_type,
            self::get_float_type,
            self::get_void_type,
        ]
        .into_iter()
        .find_map(|f| f(ctx, value))
    }
}

fn put_function_decl(
    ctx: *mut LLVMContext,
    r#mod: *mut LLVMModule,
    signature: &Signature,
    functions: &mut HashMap<String, (*mut LLVMType, *mut LLVMValue)>,
) -> *mut LLVMValue {
    unsafe {
        let ret_type =
            r#type::get_type(ctx, signature.name.1.as_deref().unwrap_or("void")).unwrap();

        let mut params = signature
            .arguments
            .iter()
            .map(|a| r#type::get_type(ctx, a.1.as_str()).unwrap())
            .collect_vec();

        let func_type =
            LLVMFunctionType(ret_type, params.as_mut_ptr(), params.len() as _, false as _);

        let func = LLVMAddFunction(r#mod, signature.name.0.as_ptr() as _, func_type);

        functions.insert(signature.name.0.clone(), (func_type, func));

        func
    }
}

fn eval_binary(
    ctx: *mut LLVMContext,
    r#mod: *mut LLVMModule,
    builder: *mut LLVMBuilder,
    params: &HashMap<String, *mut LLVMValue>,
    functions: &HashMap<String, (*mut LLVMType, *mut LLVMValue)>,
    node: &Node,
) -> *mut LLVMValue {
    unsafe {
        match node {
            Node::Scalar(s) => eval_expression(ctx, r#mod, builder, params, functions, &*s),
            Node::Compound(l, op, r) => {
                let l = match l.deref() {
                    Node::Scalar(l) => eval_expression(ctx, r#mod, builder, params, functions, &*l),
                    c @ Node::Compound(..) => {
                        eval_binary(ctx, r#mod, builder, params, functions, &c)
                    }
                };
                let r = match r.deref() {
                    Node::Scalar(r) => eval_expression(ctx, r#mod, builder, params, functions, &*r),
                    c @ Node::Compound(..) => {
                        eval_binary(ctx, r#mod, builder, params, functions, &c)
                    }
                };

                // TODO read abt difference on mul and div variants and what not
                // and on E/U abt NaN
                match op {
                    Operator::Sum => LLVMBuildAdd(builder, l, r, "sum\0".as_ptr() as _),
                    Operator::Sub => LLVMBuildSub(builder, l, r, "sub\0".as_ptr() as _),
                    Operator::Star => LLVMBuildMul(builder, l, r, "mul\0".as_ptr() as _),
                    Operator::Div => LLVMBuildFDiv(builder, l, r, "div\0".as_ptr() as _),
                    // LLVM doesn't implement && or ||, rather they work like & and | to bools (i1)
                    Operator::And | Operator::ShAnd => {
                        LLVMBuildAnd(builder, l, r, "and\0".as_ptr() as _)
                    }
                    Operator::Or | Operator::ShOr => {
                        LLVMBuildOr(builder, l, r, "or\0".as_ptr() as _)
                    }
                    Operator::Shl => LLVMBuildShl(builder, l, r, "shl\0".as_ptr() as _),
                    // Logical (LShr) vs Arithmetic (AShr) Right Shift
                    Operator::Shr => LLVMBuildLShr(builder, l, r, "shr\0".as_ptr() as _),
                    Operator::Xor => LLVMBuildXor(builder, l, r, "xor\0".as_ptr() as _),
                    Operator::Lt => LLVMBuildFCmp(
                        builder,
                        LLVMRealPredicate::LLVMRealOLT,
                        l,
                        r,
                        "lt\0".as_ptr() as _,
                    ),
                    Operator::Gt => LLVMBuildFCmp(
                        builder,
                        LLVMRealPredicate::LLVMRealOGT,
                        l,
                        r,
                        "gt\0".as_ptr() as _,
                    ),
                    Operator::Le => LLVMBuildFCmp(
                        builder,
                        LLVMRealPredicate::LLVMRealOLE,
                        l,
                        r,
                        "le\0".as_ptr() as _,
                    ),
                    Operator::Ge => LLVMBuildFCmp(
                        builder,
                        LLVMRealPredicate::LLVMRealOGE,
                        l,
                        r,
                        "ge\0".as_ptr() as _,
                    ),
                    Operator::EqEq => LLVMBuildFCmp(
                        builder,
                        LLVMRealPredicate::LLVMRealOEQ,
                        l,
                        r,
                        "eqeq\0".as_ptr() as _,
                    ),
                    Operator::Neq => LLVMBuildFCmp(
                        builder,
                        LLVMRealPredicate::LLVMRealONE,
                        l,
                        r,
                        "neq\0".as_ptr() as _,
                    ),
                }
            }
        }
    }
}

fn eval_statement(
    ctx: *mut LLVMContext,
    r#mod: *mut LLVMModule,
    builder: *mut LLVMBuilder,
    params: &HashMap<String, *mut LLVMValue>,
    functions: &HashMap<String, (*mut LLVMType, *mut LLVMValue)>,
    stmt: &Statement,
) -> *mut LLVMValue {
    unsafe {
        match stmt {
            Statement::Expression(expr) => {
                eval_expression(ctx, r#mod, builder, params, functions, expr)
            }
            Statement::Return(expr) => match expr {
                Some(expr) => LLVMBuildRet(
                    builder,
                    eval_expression(ctx, r#mod, builder, params, functions, expr),
                ),
                None => LLVMBuildRetVoid(builder),
            },
        }
    }
}

fn eval_expression(
    ctx: *mut LLVMContext,
    r#mod: *mut LLVMModule,
    builder: *mut LLVMBuilder,
    params: &HashMap<String, *mut LLVMValue>,
    functions: &HashMap<String, (*mut LLVMType, *mut LLVMValue)>,
    expr: &Expression,
) -> *mut LLVMValue {
    unsafe {
        match expr {
            Expression::Literal(lit, lit_type) => match lit_type {
                LiteralType::String => {
                    let sub = &lit[1..lit.len() - 1];
                    LLVMConstStringInContext(ctx, sub.as_ptr() as _, sub.len() as _, true as _)
                }
                LiteralType::Rune => {
                    LLVMConstInt(LLVMInt8TypeInContext(ctx), lit.parse().unwrap(), true as _)
                }
                LiteralType::Int => {
                    LLVMConstInt(LLVMInt32TypeInContext(ctx), lit.parse().unwrap(), true as _)
                }
                LiteralType::Float => LLVMConstInt(
                    LLVMDoubleTypeInContext(ctx),
                    lit.parse().unwrap(),
                    true as _,
                ),
                LiteralType::Bool => match lit.as_str() {
                    "true" => LLVMConstInt(LLVMInt1TypeInContext(ctx), 1, false as _),
                    "false" => LLVMConstInt(LLVMInt1TypeInContext(ctx), 0, false as _),
                    _ => unreachable!(),
                },
            },
            Expression::Path(p) => *params.get(&p[0]).unwrap(),
            Expression::Binary(n) => match n {
                Node::Scalar(s) => eval_expression(ctx, r#mod, builder, params, functions, &*s),
                n @ Node::Compound(..) => eval_binary(ctx, r#mod, builder, params, functions, n),
            },
            Expression::Call { path, args } => LLVMBuildCall2(
                builder,
                functions.get(&path[0]).unwrap().0,
                functions.get(&path[0]).unwrap().1,
                args.iter()
                    .map(|a| eval_expression(ctx, r#mod, builder, params, functions, a))
                    .collect_vec()
                    .as_mut_ptr(),
                args.len() as _,
                "call\0".as_ptr() as _,
            ),
            Expression::If { condition, block } => {
                let if_block = LLVMCreateBasicBlockInContext(ctx, "if\0".as_ptr() as _);
                LLVMPositionBuilderAtEnd(builder, if_block);

                block.into_iter().for_each(|s| {
                    eval_statement(ctx, r#mod, builder, params, functions, s);
                });

                LLVMBuildCondBr(
                    builder,
                    eval_expression(ctx, r#mod, builder, params, functions, &*condition),
                    if_block,
                    std::ptr::null_mut(),
                )
            }
        }
    }
}

fn put_function(
    ctx: *mut LLVMContext,
    r#mod: *mut LLVMModule,
    builder: *mut LLVMBuilder,
    functions: &mut HashMap<String, (*mut LLVMType, *mut LLVMValue)>,
    item: Item,
) {
    unsafe {
        let Item::Function { signature, body } = item;

        let func = put_function_decl(ctx, r#mod, &signature, functions);
        let func_body = LLVMAppendBasicBlockInContext(ctx, func, "entry\0".as_ptr() as _);
        LLVMPositionBuilderAtEnd(builder, func_body);

        let params = signature
            .arguments
            .into_iter()
            .enumerate()
            .map(|(pos, arg)| (arg.0, LLVMGetParam(func, pos as _)))
            .collect::<HashMap<_, _>>();

        body.into_iter().for_each(|stmt| {
            eval_statement(ctx, r#mod, builder, &params, &functions, &stmt);
        })
    }
}

pub fn gen(module: &str, ast: AST) {
    unsafe {
        let module = CString::new(module).unwrap();

        let ctx = LLVMContextCreate();
        let r#mod = LLVMModuleCreateWithNameInContext(module.as_ptr(), ctx);
        let builder = LLVMCreateBuilderInContext(ctx);

        let mut functions = HashMap::<String, (*mut LLVMType, *mut LLVMValue)>::new();

        // TODO still not sure if this makes sense
        LLVMAddSymbol("println\0".as_ptr() as _, sundae_library::println as _);

        for item in ast.0 {
            match item {
                Item::Function { .. } => put_function(ctx, r#mod, builder, &mut functions, item),
            }
        }

        LLVMPrintModuleToFile(r#mod, "output.ll\0".as_ptr() as _, std::ptr::null_mut());

        LLVM_InitializeNativeTarget();

        LLVMDisposeBuilder(builder);
        LLVMDisposeModule(r#mod);
        LLVMContextDispose(ctx);
    }
}
