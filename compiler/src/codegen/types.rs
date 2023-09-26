use inkwell::{
    context::Context,
    types::{BasicType, BasicTypeEnum, VoidType},
    AddressSpace,
};

use crate::parser::types::{BaseType, Modifiers, Type as ParserType};

pub enum Type {
    Integer { width: u32, signed: bool },
    Float(u32),
    Void,
    Pointer(Box<Type>),
    // TODO support Array()
}

impl TryFrom<ParserType> for Type {
    type Error = ();

    fn try_from(value: ParserType) -> Result<Self, Self::Error> {
        let is_ptr = matches!(value.modifier, Some(Modifiers::Ref | Modifiers::MutRef));

        let base = match value.base {
            BaseType::Array { .. } => todo!(),
            BaseType::Scalar { r#type } => {
                let (signedness, bits) = r#type.split_at(1);
                if matches!(signedness, "u" | "i") {
                    Ok(Self::Integer {
                        width: bits.parse().map_err(|_| ())?,
                        signed: signedness == "i",
                    })
                } else {
                    match r#type.as_str() {
                        "f16" => Ok(Self::Float(16)),
                        "f32" => Ok(Self::Float(32)),
                        "f64" => Ok(Self::Float(64)),
                        "f128" => Ok(Self::Float(128)),
                        "void" => Ok(Self::Void),
                        _ => Err(()),
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
    pub fn into_basic_type(self, ctx: &'ctx Context) -> Option<BasicTypeEnum<'ctx>> {
        match self {
            Self::Integer { width, .. } => Some(
                match width {
                    8 => ctx.i8_type(),
                    16 => ctx.i16_type(),
                    32 => ctx.i32_type(),
                    64 => ctx.i64_type(),
                    128 => ctx.i128_type(),
                    n => ctx.custom_width_int_type(n),
                }
                .into(),
            ),
            Self::Float(width) => Some(
                match width {
                    16 => ctx.f16_type(),
                    32 => ctx.f32_type(),
                    64 => ctx.f64_type(),
                    128 => ctx.f128_type(),
                    _ => unreachable!(),
                }
                .into(),
            ),
            Self::Pointer(inner) => Some(
                inner
                    .into_basic_type(ctx)?
                    .ptr_type(AddressSpace::default())
                    .into(),
            ),
            _ => None,
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
