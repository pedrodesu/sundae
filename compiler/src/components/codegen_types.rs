use anyhow::{anyhow, Result};
use llvm_sys::{
    core::{
        LLVMDoubleTypeInContext, LLVMFP128TypeInContext, LLVMFloatTypeInContext,
        LLVMHalfTypeInContext, LLVMInt128TypeInContext, LLVMInt16TypeInContext,
        LLVMInt32TypeInContext, LLVMInt64TypeInContext, LLVMInt8TypeInContext,
        LLVMIntTypeInContext, LLVMPointerType, LLVMVoidTypeInContext,
    },
    prelude::{LLVMContextRef, LLVMTypeRef},
};

use super::parser_types::Type as ParserType;

#[derive(Clone, PartialEq, Debug)]
pub enum Type {
    Integer { width: u32, signed: bool },
    Float(u32),
    Void,
    Array { scalar: Box<Type>, size: usize },
    Ref(Box<Type>),
    MutRef(Box<Type>),
}

impl Default for Type {
    #[inline]
    fn default() -> Self {
        Self::Void
    }
}

impl TryFrom<ParserType> for Type {
    type Error = anyhow::Error;

    fn try_from(value: ParserType) -> Result<Self, Self::Error> {
        let r#type = &value.0[0]; // TODO assume for now

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

impl Type {
    pub fn get_type(&self, ctx: LLVMContextRef) -> LLVMTypeRef {
        unsafe {
            match self {
                Self::Integer { width, .. } => match width {
                    8 => LLVMInt8TypeInContext(ctx),
                    16 => LLVMInt16TypeInContext(ctx),
                    32 => LLVMInt32TypeInContext(ctx),
                    64 => LLVMInt64TypeInContext(ctx),
                    128 => LLVMInt128TypeInContext(ctx),
                    &n => LLVMIntTypeInContext(ctx, n),
                },
                Self::Float(width) => match width {
                    16 => LLVMHalfTypeInContext(ctx),
                    32 => LLVMFloatTypeInContext(ctx),
                    64 => LLVMDoubleTypeInContext(ctx),
                    128 => LLVMFP128TypeInContext(ctx),
                    _ => unreachable!(),
                },
                Self::Void => LLVMVoidTypeInContext(ctx),
                Self::Array { .. } => todo!(),
                Self::Ref(inner) | Self::MutRef(inner) => {
                    LLVMPointerType(inner.get_type(ctx), Default::default())
                }
            }
        }
    }
}
