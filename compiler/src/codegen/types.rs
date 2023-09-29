use std::collections::HashMap;

use anyhow::{anyhow, Result};
use inkwell::{
    context::Context,
    types::{BasicType, BasicTypeEnum, VoidType},
    values::{BasicValueEnum, FunctionValue},
    AddressSpace,
};

use crate::parser::types::{BaseType, Modifiers, ParserType};

#[derive(Clone)]
pub enum Type {
    Integer { width: u32, signed: bool },
    Float(u32),
    Void,
    Pointer(Box<Type>),
    Array { scalar: Box<Type>, size: usize },
}

#[derive(Clone)]
pub struct Value<'ctx> {
    pub inner: BasicValueEnum<'ctx>,
    pub r#type: Type,
}

pub struct Function<'ctx> {
    pub inner: FunctionValue<'ctx>,
    pub stack: HashMap<String, Value<'ctx>>,
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

impl<'ctx> Type {
    pub fn get_basic_type(&self, ctx: &'ctx Context) -> Result<BasicTypeEnum<'ctx>> {
        match self {
            Self::Integer { width, .. } => Ok(match *width {
                8 => ctx.i8_type(),
                16 => ctx.i16_type(),
                32 => ctx.i32_type(),
                64 => ctx.i64_type(),
                128 => ctx.i128_type(),
                n => ctx.custom_width_int_type(n),
            }
            .into()),
            Self::Float(width) => Ok(match *width {
                16 => ctx.f16_type(),
                32 => ctx.f32_type(),
                64 => ctx.f64_type(),
                128 => ctx.f128_type(),
                _ => unreachable!(),
            }
            .into()),
            Self::Pointer(inner) => Ok(inner
                .get_basic_type(ctx)?
                .ptr_type(AddressSpace::default())
                .into()),
            Self::Void => Err(anyhow!(
                "Trying to use void in a place that requires a non-zero type"
            )),
            Self::Array { .. } => todo!(),
        }
    }

    pub fn into_void_type(self, ctx: &'ctx Context) -> Option<VoidType<'ctx>> {
        if matches!(self, Self::Void) {
            return Some(ctx.void_type());
        } else {
            None
        }
    }
}
