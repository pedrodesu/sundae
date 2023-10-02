use anyhow::{anyhow, bail, Result};
use inkwell::values::{BasicValue, BasicValueEnum, IntValue};

use crate::{
    codegen::Codegen,
    components::codegen_types::{Function, Type, Value},
    lexer::definitions::LiteralType,
};

use super::Expression;

impl<'ctx> Codegen<'ctx> {
    #[inline]
    pub fn assure_int_expr(expr: Result<Value<'ctx>>) -> Result<IntValue<'ctx>> {
        if let BasicValueEnum::IntValue(v) = expr?.inner {
            Ok(v)
        } else {
            bail!("Expression asks for an int")
        }
    }

    #[inline]
    pub fn gen_non_void_expression(
        &self,
        func: &mut Function<'ctx>,
        expression: Expression,
    ) -> Result<Value<'ctx>> {
        self.gen_expression(func, expression)
            .and_then(|e| e.ok_or(anyhow!("Using a void value as expression")))
    }

    pub fn gen_expression(
        &self,
        func: &mut Function<'ctx>,
        expression: Expression,
    ) -> Result<Option<Value<'ctx>>> {
        Ok(match expression {
            Expression::Literal { value, r#type } => Some(match r#type {
                LiteralType::String => {
                    let bytes = value[1..value.len() - 1].as_bytes();

                    Value {
                        inner: self.ctx.const_string(bytes, true).into(),
                        r#type: Type::Array {
                            scalar: Box::new(Type::Integer {
                                width: 8,
                                signed: true,
                            }),
                            size: bytes.len(),
                        },
                    }
                }
                LiteralType::Rune => Value {
                    inner: self
                        .ctx
                        .i8_type()
                        .const_int(value.as_bytes()[1].into(), false)
                        .into(),
                    r#type: Type::Integer {
                        width: 8,
                        signed: true,
                    },
                },
                LiteralType::Int => Value {
                    inner: self
                        .ctx
                        .i32_type()
                        .const_int(value.parse().unwrap(), false)
                        .into(),
                    r#type: Type::Integer {
                        width: 32,
                        signed: true,
                    },
                },
                LiteralType::Float => Value {
                    inner: self
                        .ctx
                        .f64_type()
                        .const_float(value.parse().unwrap())
                        .into(),
                    r#type: Type::Float(64),
                },
            }),
            Expression::Reference(expr) => {
                let expr = self.gen_non_void_expression(func, *expr)?;

                let alloc = self
                    .builder
                    .build_alloca(expr.r#type.get_basic_type(self.ctx)?, "ref");
                self.builder.build_store(alloc, expr.inner);

                Some(Value {
                    inner: alloc.as_basic_value_enum(),
                    r#type: Type::Pointer(Box::new(expr.r#type)),
                })
            }
            Expression::Dereference(expr) => {
                let expr = self.gen_non_void_expression(func, *expr)?;

                let pointee_type = if let Type::Pointer(inner) = expr.r#type {
                    *inner
                } else {
                    bail!("Trying to dereference non-reference")
                };

                let value = self.builder.build_load(
                    pointee_type.clone().get_basic_type(self.ctx)?,
                    expr.inner.into_pointer_value(),
                    "deref",
                );

                Some(Value {
                    inner: value,
                    r#type: pointee_type,
                })
            }
            Expression::Path(path) => {
                let name = path.last().unwrap();

                let lookup = func
                    .stack
                    .get(name)
                    .ok_or_else(|| anyhow!("Identifier `{}` not found", name))?
                    .clone();

                Some(Value {
                    inner: self.builder.build_load(
                        lookup.r#type.get_basic_type(self.ctx)?,
                        lookup.inner.into_pointer_value(),
                        name,
                    ),
                    r#type: lookup.r#type,
                })
            }
            Expression::Binary(n) => Some(self.gen_binary(func, n)?),
            Expression::Call { path, args } => {
                let name = path.last().unwrap();

                let function = self
                    .module
                    .get_function(name)
                    .ok_or_else(|| anyhow!("Function `{}` not found", name))?;

                let ret = self.builder.build_call(
                    function,
                    args.into_iter()
                        .map(|e| {
                            self.gen_non_void_expression(func, e)
                                .map(|v| v.inner.into())
                        })
                        .collect::<Result<Vec<_>>>()?
                        .as_slice(),
                    "call",
                );

                let ret = ret.try_as_basic_value();

                // TODO make this shit better, fucking lifetimes lol

                if ret.is_left() {
                    Some(Value {
                        inner: ret.unwrap_left(),
                        r#type: self.functions.get(name).unwrap().clone(),
                    })
                } else {
                    None
                }
            }
            Expression::If {
                condition,
                block,
                else_block,
            } => {
                let then = self.ctx.append_basic_block(func.inner, "then");
                let r#else = self.ctx.append_basic_block(func.inner, "else");

                // TODO optimise else conds
                let r#continue = if else_block.is_none() {
                    Some(self.ctx.append_basic_block(func.inner, "continue"))
                } else {
                    None
                };

                self.builder.position_at_end(then);

                for statement in block {
                    self.gen_statement(func, statement)?;
                }

                if let Some(r#continue) = r#continue {
                    self.builder.build_unconditional_branch(r#continue);
                }

                self.builder.position_at_end(r#else);

                if let Some(r#continue) = r#continue {
                    self.builder.build_unconditional_branch(r#continue);
                } else {
                    else_block
                        .unwrap()
                        .into_iter()
                        .map(|s| self.gen_statement(func, s))
                        .collect::<Result<_>>()?;
                }

                self.builder
                    .position_at_end(func.inner.get_first_basic_block().unwrap());

                self.builder.build_conditional_branch(
                    Self::assure_int_expr(self.gen_non_void_expression(func, *condition))?,
                    then,
                    r#else,
                );

                if let Some(r#continue) = r#continue {
                    self.builder.position_at_end(r#continue);
                }

                None
            }
        })
    }
}
