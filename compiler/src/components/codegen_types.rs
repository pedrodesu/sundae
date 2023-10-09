use std::collections::HashMap;

use anyhow::{anyhow, Result};
use llvm_sys::{
    core::{
        LLVMDoubleTypeInContext, LLVMFP128TypeInContext, LLVMFloatTypeInContext,
        LLVMHalfTypeInContext, LLVMInt128TypeInContext, LLVMInt16TypeInContext,
        LLVMInt32TypeInContext, LLVMInt64TypeInContext, LLVMInt8TypeInContext,
        LLVMIntTypeInContext, LLVMPointerType, LLVMVoidTypeInContext,
    },
    prelude::{LLVMContextRef, LLVMTypeRef, LLVMValueRef},
};

use super::parser_types::{BaseType, Modifiers, ParserType};

#[derive(Clone)]
pub enum Type {
    Integer { width: u32, signed: bool },
    Float(u32),
    Void,
    Pointer(Box<Type>),
    Array { scalar: Box<Type>, size: usize },
}

#[derive(Clone)]
pub struct Value {
    pub inner: LLVMValueRef,
    pub r#type: Type,
}

pub struct Function {
    pub inner: LLVMValueRef,
    pub stack: HashMap<String, Value>,
}

impl TryFrom<ParserType> for Type {
    type Error = anyhow::Error;

    fn try_from(value: ParserType) -> Result<Self, Self::Error> {
        let is_ptr = matches!(value.modifier, Some(Modifiers::Ref | Modifiers::MutRef));

        let base = match value.base {
            BaseType::Array { r#type, size } => {
                let scalar = Box::new(Self::try_from(ParserType {
                    base: BaseType::Scalar { r#type },
                    modifier: None,
                })?);
                Ok(Self::Array { scalar, size })
            }
            BaseType::Scalar { r#type } => {
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
        }?;

        if !is_ptr {
            Ok(base)
        } else {
            Ok(Self::Pointer(Box::new(base)))
        }
    }
}

impl Type {
    pub fn get_type(&self, ctx: LLVMContextRef) -> LLVMTypeRef {
        unsafe {
            match self {
                Self::Integer { width, .. } => match *width {
                    8 => LLVMInt8TypeInContext(ctx),
                    16 => LLVMInt16TypeInContext(ctx),
                    32 => LLVMInt32TypeInContext(ctx),
                    64 => LLVMInt64TypeInContext(ctx),
                    128 => LLVMInt128TypeInContext(ctx),
                    n => LLVMIntTypeInContext(ctx, n),
                },
                Self::Float(width) => match *width {
                    16 => LLVMHalfTypeInContext(ctx),
                    32 => LLVMFloatTypeInContext(ctx),
                    64 => LLVMDoubleTypeInContext(ctx),
                    128 => LLVMFP128TypeInContext(ctx),
                    _ => unreachable!(),
                },
                Self::Pointer(inner) => LLVMPointerType(inner.get_type(ctx), 0),
                Self::Void => LLVMVoidTypeInContext(ctx),
                Self::Array { .. } => todo!(),
            }
        }
    }
}
