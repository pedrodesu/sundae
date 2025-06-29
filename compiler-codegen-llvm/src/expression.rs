use std::{cell::RefCell, rc::Rc};

use anyhow::{Result, anyhow, bail, ensure};
use compiler_lexer::definitions::LiteralType;
use compiler_parser::{Expression, Operator};
use inkwell::values::BasicValue;

use crate::{Codegen, Function, Type, Value};

mod binary;

impl<'ctx> Codegen<'ctx>
{
    #[inline]
    pub fn ref_cast(&self, from_value: Value<'ctx>, to: Type) -> Result<Value<'ctx>>
    {
        match [from_value.r#type, to] {
            [Type::MutRef(box from), Type::MutRef(box to)]
            | [Type::Ref(box from), Type::Ref(box to)]
            | [from, to]
            // TODO i believe this if guard should be removed here, for the ensure later
                if from == to =>
            {
                ensure!(from == to, "Cast asks for `{}`, got `{}`", from, to);

                Ok(Value { r#type: to.clone(), inner: from_value.inner })
            }
            [from, Type::MutRef(box to)] | [from, Type::Ref(box to)] => {
                ensure!(from == to, "Cast asks for `{}`, got `{}`", from, to);

                let ptr = self
                    .builder
                    .build_alloca(from.as_llvm_basic_type(self.ctx)?, "cast")?;

                self.builder.build_store(ptr, from_value.inner)?;

                Ok(Value { r#type: to.clone(), inner: ptr.as_basic_value_enum() })
            }
            [Type::MutRef(box from), to] | [Type::Ref(box from), to] => {
                ensure!(from == to, "Cast asks for `{}`, got `{}`", from, to);

                let load = self.builder.build_load(
                    to.as_llvm_basic_type(self.ctx)?,
                    from_value.inner.into_pointer_value(),
                    "cast",
                )?;

                Ok(Value { r#type: to.clone(), inner: load })
            }
            _ => unreachable!(),
        }
    }

    // TODO allow making a cast here directly
    #[inline]
    pub fn gen_non_void_expression(
        &self,
        parent_func: &Option<Rc<RefCell<Function<'ctx>>>>,
        expression: Expression,
    ) -> Result<Value<'ctx>>
    {
        self.gen_expression(parent_func, expression)
            .and_then(|e| e.ok_or(anyhow!("Using void as an expression")))
    }

    pub fn gen_expression(
        &self,
        parent_func: &Option<Rc<RefCell<Function<'ctx>>>>,
        expression: Expression,
    ) -> Result<Option<Value<'ctx>>>
    {
        Ok(match expression
        {
            Expression::Literal { value, r#type } => Some(match r#type
            {
                LiteralType::String =>
                {
                    let bytes = value[1..value.len() - 1].as_bytes();

                    Value {
                        inner: self.ctx.const_string(bytes, false).into(),
                        r#type: Type::Array {
                            scalar: Box::new(Type::Integer {
                                width: 8,
                                signed: true,
                            }),
                            size: bytes.len().try_into()?,
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
                    inner: self.ctx.i32_type().const_int(value.parse()?, false).into(),
                    r#type: Type::Integer {
                        width: 32,
                        signed: true,
                    },
                },
                LiteralType::Float => Value {
                    inner: self.ctx.f64_type().const_float(value.parse()?).into(),
                    r#type: Type::Float(64),
                },
            }),
            Expression::Path(path) =>
            {
                let name = path.last().unwrap();

                if let Some(global) = self.runtime.borrow().constants.get(path[0].as_str())
                {
                    Some(Value {
                        r#type: global.r#type.clone(),
                        inner: self
                            .module
                            .get_global(name)
                            .unwrap()
                            .get_initializer()
                            .unwrap(),
                    })
                }
                else
                {
                    let lookup = parent_func
                        .as_ref()
                        .unwrap()
                        .borrow()
                        .stack
                        .get(name)
                        .ok_or_else(|| anyhow!("Identifier `{}` not found", name))?
                        .clone();

                    Some(lookup)
                }
            }
            Expression::Binary(box n) => Some(self.gen_binary(parent_func, n)?),
            Expression::Unary(op, box e) =>
            {
                let value = self.gen_non_void_expression(parent_func, e)?;
                // TODO cast should be immediate
                let value = self.ref_cast(
                    value,
                    parent_func.as_ref().unwrap().borrow().return_type.clone(),
                )?;

                if op == Operator::Minus && value.inner.is_int_value()
                {
                    Some(Value {
                        r#type: value.r#type,
                        inner: self
                            .builder
                            .build_int_neg(value.inner.into_int_value(), "neg")?
                            .into(),
                    })
                }
                else if op == Operator::Minus && value.inner.is_float_value()
                {
                    Some(Value {
                        r#type: value.r#type,
                        inner: self
                            .builder
                            .build_float_neg(value.inner.into_float_value(), "neg")?
                            .into(),
                    })
                }
                else
                {
                    unreachable!()
                }
            }
            Expression::Call { path, args } =>
            {
                let name = path.last().unwrap();

                let runtime = self.runtime.borrow();

                let Some(function) = runtime.functions.get(name)
                else
                {
                    bail!("Function `{}` not found", name);
                };

                let ret = self.builder.build_call(
                    function.borrow().inner,
                    args.into_iter()
                        .enumerate()
                        .map(|(i, e)| {
                            let function = function.borrow();
                            let Some(decl_type) = function.arguments.get(i)
                            else
                            {
                                bail!(
                                    "Function `{}` expects {} arguments",
                                    name,
                                    function.arguments.len()
                                );
                            };

                            let value = self
                                .gen_non_void_expression(&parent_func.as_ref().map(Rc::clone), e)?;

                            // TODO we might need the cast everywhere else.. test
                            Ok(self.ref_cast(value, decl_type.1.clone())?.inner.into())
                        })
                        .collect::<Result<Vec<_>>>()?
                        .as_slice(),
                    "call",
                )?;

                ret.try_as_basic_value().left().map(|r| Value {
                    inner: r,
                    r#type: function.borrow().return_type.clone(),
                })
            }
            Expression::If {
                condition,
                block,
                else_block,
            } =>
            {
                let then = self
                    .ctx
                    .append_basic_block(parent_func.as_ref().unwrap().borrow().inner, "then");
                let r#else = self
                    .ctx
                    .append_basic_block(parent_func.as_ref().unwrap().borrow().inner, "else");

                let r#continue = if else_block.is_none()
                {
                    Some(self.ctx.append_basic_block(
                        parent_func.as_ref().unwrap().borrow().inner,
                        "continue",
                    ))
                }
                else
                {
                    None
                };

                self.builder.position_at_end(then);

                for statement in block
                {
                    self.gen_statement(&parent_func.as_ref().map(Rc::clone), statement)?;
                }

                if let Some(r#continue) = r#continue
                {
                    self.builder.build_unconditional_branch(r#continue)?;
                }

                self.builder.position_at_end(r#else);

                if let Some(r#continue) = r#continue
                {
                    self.builder.build_unconditional_branch(r#continue)?;
                }
                else
                {
                    else_block.unwrap().into_iter().try_for_each(|s| {
                        self.gen_statement(&parent_func.as_ref().map(Rc::clone), s)
                    })?;
                }

                self.builder.position_at_end(
                    parent_func
                        .as_ref()
                        .unwrap()
                        .borrow()
                        .inner
                        .get_first_basic_block()
                        .unwrap(),
                );

                let gen_condition = self.gen_non_void_expression(parent_func, *condition)?;

                if !gen_condition.inner.is_int_value()
                {
                    bail!("Expected condition, got {:?}", gen_condition.r#type);
                }

                self.builder.build_conditional_branch(
                    gen_condition.inner.into_int_value(),
                    then,
                    r#else,
                )?;

                if let Some(r#continue) = r#continue
                {
                    self.builder.position_at_end(r#continue);
                }

                None // TODO this is incorrect
            }
            Expression::Parenthesis(box e) =>
            {
                let value = self.gen_non_void_expression(parent_func, e)?;

                Some(value)
            }
            Expression::Tuple(_) =>
            {
                // let mapped = eco_vec
                //     .into_iter()
                //     .map(|e| Ok(self.gen_non_void_expression(parent_func, e)?.inner))
                //     .collect::<Result<Vec<_>, _>>()?;

                // let llvm_value = self.ctx.const_struct(mapped.as_slice(), false);

                // let value = Value {
                //     r#type: Type::Tuple(mapped.iter().map(|v| v.get_type()).collect()),
                //     inner: llvm_value.into(),
                // };

                // Some(value)
                todo!()
            }
            Expression::Array(_) =>
            {
                // let size = eco_vec.len();

                // self.builder.build_array_alloca();

                todo!()
            }
        })
    }
}
