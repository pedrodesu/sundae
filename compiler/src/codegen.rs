use std::{
    collections::HashMap,
    ffi::CString,
    ops::Deref,
    sync::{LazyLock, OnceLock},
};

use itertools::Itertools;
use llvm_sys::{
    core::*,
    prelude::{LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMValueRef},
    LLVMIntPredicate, LLVMRealPredicate,
};

use crate::{
    lexer::LiteralType,
    parser::{
        binary::{Node, Operator},
        Expression, Item, Signature, Statement, AST,
    },
};

static mut CTX: LazyLock<LLVMContextRef> = LazyLock::new(|| unsafe { LLVMContextCreate() });
static mut MOD: OnceLock<LLVMModuleRef> = OnceLock::new();
static mut BUILDER: LazyLock<LLVMBuilderRef> =
    LazyLock::new(|| unsafe { LLVMCreateBuilderInContext(*CTX) });

#[inline]
fn r#mod() -> LLVMModuleRef {
    *unsafe { MOD.get() }.unwrap()
}

// LLVM makes no difference between signed and unsigned types, only wants the amount of bits
mod r#type {
    use llvm_sys::{core::*, prelude::LLVMTypeRef};

    use super::CTX;

    #[inline]
    fn get_fixed_int_type(value: &str) -> Option<LLVMTypeRef> {
        unsafe {
            match value {
                "u8" | "i8" => Some(LLVMInt8TypeInContext(*CTX)),
                "u16" | "i16" => Some(LLVMInt16TypeInContext(*CTX)),
                "u32" | "i32" => Some(LLVMInt32TypeInContext(*CTX)),
                "u64" | "i64" => Some(LLVMInt64TypeInContext(*CTX)),
                "u128" | "i128" => Some(LLVMInt128TypeInContext(*CTX)),
                _ => None,
            }
        }
    }

    #[inline]
    fn get_int_type(value: &str) -> Option<LLVMTypeRef> {
        if let Some(literal) = value.as_ascii() {
            let (signedness, bits) = (literal[0], literal[1..].as_str());
            if matches!(signedness.to_char(), 'u' | 'i') {
                if let Ok(bits) = bits.parse::<u32>() {
                    return Some(unsafe { LLVMIntTypeInContext(*CTX, bits) });
                }
            }
        }
        None
    }

    #[inline]
    fn get_bool_type(value: &str) -> Option<LLVMTypeRef> {
        if value == "bool" {
            Some(unsafe { LLVMInt1TypeInContext(*CTX) })
        } else {
            None
        }
    }

    #[inline]
    fn get_float_type(value: &str) -> Option<LLVMTypeRef> {
        unsafe {
            match value {
                "f16" => Some(LLVMHalfTypeInContext(*CTX)),
                "f32" => Some(LLVMFloatTypeInContext(*CTX)),
                "f64" => Some(LLVMDoubleTypeInContext(*CTX)),
                "f128" => Some(LLVMFP128TypeInContext(*CTX)),
                _ => None,
            }
        }
    }

    #[inline]
    fn get_void_type(value: &str) -> Option<LLVMTypeRef> {
        if value == "void" {
            Some(unsafe { LLVMVoidTypeInContext(*CTX) })
        } else {
            None
        }
    }

    #[inline]
    pub fn get_type(value: &str) -> Option<LLVMTypeRef> {
        [
            self::get_fixed_int_type,
            self::get_int_type,
            self::get_bool_type,
            self::get_float_type,
            self::get_void_type,
        ]
        .into_iter()
        .find_map(|f| f(value))
    }
}

fn put_function_decl(signature: &Signature) -> LLVMValueRef {
    let ret_type = r#type::get_type(signature.name.1.as_deref().unwrap_or("void")).unwrap();

    let mut params = signature
        .arguments
        .iter()
        .map(|a| r#type::get_type(a.1.as_str()).unwrap())
        .collect_vec();

    let func_type =
        unsafe { LLVMFunctionType(ret_type, params.as_mut_ptr(), params.len() as _, false as _) };

    let func_name = CString::new(signature.name.0.clone()).unwrap();
    let func = unsafe { LLVMAddFunction(r#mod(), func_name.as_ptr(), func_type) };

    func
}

fn eval_binary(
    params: &HashMap<String, LLVMValueRef>,
    func: LLVMValueRef,
    node: &Node,
) -> LLVMValueRef {
    match node {
        Node::Scalar(s) => eval_expression(params, func, &*s),
        Node::Compound(l, op, r) => {
            let eval_side = |side: &Box<Node>| match side.deref() {
                Node::Scalar(node) => eval_expression(params, func, &*node),
                node @ Node::Compound(..) => eval_binary(params, func, &node),
            };

            let (l, r) = (eval_side(l), eval_side(r));

            // TODO check for and generate according instructions for fp operands
            // same thing for signed and unsigned predicate types

            // TODO read abt difference on mul and div variants and what not
            // and on E/U abt NaN
            unsafe {
                match op {
                    Operator::Sum => LLVMBuildAdd(*BUILDER, l, r, "sum\0".as_ptr() as _),
                    Operator::Sub => LLVMBuildSub(*BUILDER, l, r, "sub\0".as_ptr() as _),
                    Operator::Star => LLVMBuildMul(*BUILDER, l, r, "mul\0".as_ptr() as _),
                    Operator::Div => LLVMBuildFDiv(*BUILDER, l, r, "div\0".as_ptr() as _),
                    // LLVM doesn't implement && or ||, rather they work like & and | to bools (i1)
                    Operator::And | Operator::ShAnd => {
                        LLVMBuildAnd(*BUILDER, l, r, "and\0".as_ptr() as _)
                    }
                    Operator::Or | Operator::ShOr => {
                        LLVMBuildOr(*BUILDER, l, r, "or\0".as_ptr() as _)
                    }
                    Operator::Shl => LLVMBuildShl(*BUILDER, l, r, "shl\0".as_ptr() as _),
                    // Logical (LShr) vs Arithmetic (AShr) Right Shift
                    Operator::Shr => LLVMBuildLShr(*BUILDER, l, r, "shr\0".as_ptr() as _),
                    Operator::Xor => LLVMBuildXor(*BUILDER, l, r, "xor\0".as_ptr() as _),
                    Operator::Lt => LLVMBuildICmp(
                        *BUILDER,
                        LLVMIntPredicate::LLVMIntULT,
                        l,
                        r,
                        "lt\0".as_ptr() as _,
                    ),
                    Operator::Gt => LLVMBuildICmp(
                        *BUILDER,
                        LLVMIntPredicate::LLVMIntUGT,
                        l,
                        r,
                        "gt\0".as_ptr() as _,
                    ),
                    Operator::Le => LLVMBuildICmp(
                        *BUILDER,
                        LLVMIntPredicate::LLVMIntULE,
                        l,
                        r,
                        "le\0".as_ptr() as _,
                    ),
                    Operator::Ge => LLVMBuildICmp(
                        *BUILDER,
                        LLVMIntPredicate::LLVMIntUGE,
                        l,
                        r,
                        "ge\0".as_ptr() as _,
                    ),
                    Operator::EqEq => LLVMBuildICmp(
                        *BUILDER,
                        LLVMIntPredicate::LLVMIntEQ,
                        l,
                        r,
                        "eqeq\0".as_ptr() as _,
                    ),
                    Operator::Neq => LLVMBuildICmp(
                        *BUILDER,
                        LLVMIntPredicate::LLVMIntNE,
                        l,
                        r,
                        "neq\0".as_ptr() as _,
                    ),
                }
            }
        }
    }
}

// TODO handle params without saving in HashMap, and then implement funcs with From<> to LLVMValueRef

fn eval_statement(
    params: &HashMap<String, LLVMValueRef>,
    func: LLVMValueRef,
    stmt: &Statement,
) -> LLVMValueRef {
    match stmt {
        Statement::Expression(expr) => eval_expression(params, func, expr),
        Statement::Return(expr) => unsafe {
            match expr {
                Some(expr) => LLVMBuildRet(*BUILDER, eval_expression(params, func, expr)),
                None => LLVMBuildRetVoid(*BUILDER),
            }
        },
    }
}

fn eval_expression(
    params: &HashMap<String, LLVMValueRef>,
    func: LLVMValueRef,
    expr: &Expression,
) -> LLVMValueRef {
    match expr {
        Expression::Literal(lit, lit_type) => unsafe {
            match lit_type {
                LiteralType::String => {
                    let sub = &lit[1..lit.len() - 1];
                    LLVMConstStringInContext(*CTX, sub.as_ptr() as _, sub.len() as _, true as _)
                }
                LiteralType::Rune => {
                    LLVMConstInt(LLVMInt8TypeInContext(*CTX), lit.parse().unwrap(), true as _)
                }
                LiteralType::Int => LLVMConstInt(
                    LLVMInt32TypeInContext(*CTX),
                    lit.parse().unwrap(),
                    true as _,
                ),
                LiteralType::Float => LLVMConstInt(
                    LLVMDoubleTypeInContext(*CTX),
                    lit.parse().unwrap(),
                    true as _,
                ),
                LiteralType::Bool => match lit.as_str() {
                    "true" => LLVMConstInt(LLVMInt1TypeInContext(*CTX), 1, false as _),
                    "false" => LLVMConstInt(LLVMInt1TypeInContext(*CTX), 0, false as _),
                    _ => unreachable!(),
                },
            }
        },
        Expression::Path(p) => *params.get(&p[0]).unwrap(),
        Expression::Binary(n) => match n {
            Node::Scalar(s) => eval_expression(params, func, &*s),
            n @ Node::Compound(..) => eval_binary(params, func, n),
        },
        Expression::Call { path, args } => {
            let func_name = CString::new(path[0].clone()).unwrap();
            let (func, func_type) = unsafe {
                let func = LLVMGetNamedFunction(r#mod(), func_name.as_ptr());
                (func, LLVMGlobalGetValueType(func))
            };
            unsafe {
                LLVMBuildCall2(
                    *BUILDER,
                    func_type,
                    func,
                    args.iter()
                        .map(|a| eval_expression(params, func, a))
                        .collect_vec()
                        .as_mut_ptr(),
                    args.len() as _,
                    "call\0".as_ptr() as _,
                )
            }
        }
        Expression::If { condition, block } => {
            let then_block =
                unsafe { LLVMAppendBasicBlockInContext(*CTX, func, "if\0".as_ptr() as _) };
            unsafe { LLVMPositionBuilderAtEnd(*BUILDER, then_block) };

            block.into_iter().for_each(|s| {
                eval_statement(params, func, s);
            });

            // the idea here is to create new block for the rest of the code and point condbr else there

            unsafe {
                LLVMPositionBuilderAtEnd(*BUILDER, LLVMGetEntryBasicBlock(func));
                // LLVMBuildBr (https://github.com/Virtual-Machine/llvm-tutorial-book/blob/master/chap-7-if-else.md)
                LLVMBuildCondBr(
                    *BUILDER,
                    eval_expression(params, func, &*condition),
                    then_block,
                    LLVMCreateBasicBlockInContext(*CTX, "empty\0".as_ptr() as _),
                )
            }
        }
    }
}

fn put_function(item: Item) {
    let Item::Function { signature, body } = item;

    let func = put_function_decl(&signature);
    let func_body = unsafe { LLVMAppendBasicBlockInContext(*CTX, func, "entry\0".as_ptr() as _) };
    unsafe { LLVMPositionBuilderAtEnd(*BUILDER, func_body) };

    let params = signature
        .arguments
        .into_iter()
        .enumerate()
        .map(|(pos, arg)| (arg.0, unsafe { LLVMGetParam(func, pos as _) }))
        .collect::<HashMap<_, _>>();

    body.into_iter().for_each(|stmt| {
        eval_statement(&params, func, &stmt);
    })
}

pub fn gen(module: &str, ast: AST) {
    let module = CString::new(module).unwrap();

    unsafe {
        MOD.set(LLVMModuleCreateWithNameInContext(module.as_ptr(), *CTX))
            .unwrap()
    };

    // add library?
    // LLVMAddSymbol("println\0".as_ptr() as _, sundae_library::println as _);

    for item in ast.0 {
        match item {
            Item::Function { .. } => put_function(item),
        }
    }

    unsafe {
        LLVMPrintModuleToFile(r#mod(), "output.ll\0".as_ptr() as _, std::ptr::null_mut());

        // start compilation?
        // LLVM_InitializeNativeTarget();

        LLVMDisposeBuilder(*BUILDER);
        LLVMDisposeModule(r#mod());
        LLVMContextDispose(*CTX);
    }
}
