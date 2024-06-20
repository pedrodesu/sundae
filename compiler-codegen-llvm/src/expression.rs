use std::{cell::RefCell, rc::Rc};

use anyhow::{anyhow, bail, ensure, Result};
use compiler_lexer::definitions::LiteralType;
use compiler_parser::Expression;

use crate::{Codegen, Function, Type, Value};

mod binary;

impl<'ctx> Codegen<'ctx> {
    #[inline]
    pub fn gen_non_void_expression(
        &self,
        func: Rc<RefCell<Function<'ctx>>>,
        expression: Expression,
    ) -> Result<Value> {
        self.gen_expression(func, expression)
            .and_then(|e| e.ok_or(anyhow!("Using a void value as expression")))
    }

    pub fn gen_expression(
        &self,
        func: Rc<RefCell<Function<'ctx>>>,
        expression: Expression,
    ) -> Result<Option<Value>> {
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
            Expression::Path(path) => {
                let name = path.last().unwrap();

                let lookup = func
                    .borrow()
                    .stack
                    .get(name)
                    .ok_or_else(|| anyhow!("Identifier `{}` not found", name))?
                    .clone();

                Some(lookup)
            }
            Expression::Binary(n) => Some(self.gen_binary(func, n)?),
            Expression::Call { path, args } => {
                let name = path.last().unwrap();

                let rt = self.runtime.borrow();

                let Some(function) = rt.functions.get(name) else {
                    bail!("Function `{}` not found", name);
                };

                let ret = self.builder.build_call(
                    function.borrow().inner,
                    args.into_iter()
                        .enumerate()
                        .map(|(i, e)| {
                            let value = self.gen_non_void_expression(Rc::clone(&func), e)?;

                            let function = function.borrow();
                            let Some(decl_type) = function.arguments.get(i) else {
                                bail!(
                                    "Function `{}` expects {} arguments",
                                    name,
                                    function.arguments.len()
                                );
                            };

                            // TODO turn this into a function and integrate into locals and maybe assignments, normalize ref type coersion
                            // TODO can prolly be bettered
                            match [&value.r#type, &decl_type.1] {
                                [Type::MutRef(box arg_base_type), Type::MutRef(box decl_base_type)]
                                | [Type::Ref(box arg_base_type), Type::Ref(box decl_base_type)]
                                | [arg_base_type, decl_base_type]
                                    if arg_base_type == decl_base_type =>
                                {
                                    ensure!(
                                        *arg_base_type == *decl_base_type,
                                        "Function `{}` argument `{}` asks for a {}, got {}",
                                        name,
                                        decl_type.0,
                                        arg_base_type,
                                        decl_base_type
                                    );

                                    Ok(value.inner.into())
                                }
                                [arg_base_type, Type::MutRef(box decl_base_type)] => {
                                    ensure!(
                                        *arg_base_type == *decl_base_type,
                                        "Function `{}` argument `{}` asks for a {}, got {}",
                                        name,
                                        decl_type.0,
                                        arg_base_type,
                                        decl_base_type
                                    );

                                    let ptr = self.builder.build_alloca(
                                        arg_base_type.as_llvm_basic_type(&self.ctx)?,
                                        "cast",
                                    )?;

                                    self.builder.build_store(ptr, value.inner)?;

                                    Ok(ptr.into())
                                }
                                [Type::MutRef(box arg_base_type), decl_base_type] => {
                                    ensure!(
                                        *arg_base_type == *decl_base_type,
                                        "Function `{}` argument `{}` asks for a {}, got {}",
                                        name,
                                        decl_type.0,
                                        arg_base_type,
                                        decl_base_type
                                    );

                                    let cast = self.builder.build_load(
                                        decl_base_type.as_llvm_basic_type(&self.ctx)?,
                                        value.inner.into_pointer_value(),
                                        "cast",
                                    )?;

                                    Ok(cast.into())
                                }
                                _ => todo!(),
                            }
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
            } => {
                let then = self.ctx.append_basic_block(func.borrow().inner, "then");
                let r#else = self.ctx.append_basic_block(func.borrow().inner, "else");

                // TODO optimise else conds
                let r#continue = if else_block.is_none() {
                    Some(self.ctx.append_basic_block(func.borrow().inner, "continue"))
                } else {
                    None
                };

                self.builder.position_at_end(then);

                for statement in block {
                    self.gen_statement(Rc::clone(&func), statement)?;
                }

                if let Some(r#continue) = r#continue {
                    self.builder.build_unconditional_branch(r#continue)?;
                }

                self.builder.position_at_end(r#else);

                if let Some(r#continue) = r#continue {
                    self.builder.build_unconditional_branch(r#continue)?;
                } else {
                    else_block
                        .unwrap()
                        .into_iter()
                        .try_for_each(|s| self.gen_statement(Rc::clone(&func), s))?;
                }

                self.builder
                    .position_at_end(func.borrow().inner.get_first_basic_block().unwrap());

                let gen_condition = self.gen_non_void_expression(func, *condition)?;

                // TODO how to handle bools with our logic?
                if !gen_condition.inner.is_int_value() {
                    bail!("Expected condition, got {:?}", gen_condition.r#type);
                }

                self.builder.build_conditional_branch(
                    gen_condition.inner.into_int_value(),
                    then,
                    r#else,
                )?;

                if let Some(r#continue) = r#continue {
                    self.builder.position_at_end(r#continue);
                }

                None
            }
        })
    }
}
