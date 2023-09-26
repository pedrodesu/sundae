use anyhow::{anyhow, bail, Result};
use inkwell::values::{BasicValueEnum, IntValue};

use crate::{
    lexer::definitions::LiteralType,
    parser::expression::{binary::Node, Expression},
};

use super::{Codegen, Function};

mod binary;

impl<'ctx> Codegen<'ctx> {
    #[inline]
    pub fn assure_int_expr(expr: Result<BasicValueEnum>) -> Result<IntValue> {
        if let BasicValueEnum::IntValue(v) = expr? {
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
    ) -> Result<BasicValueEnum> {
        self.gen_expression(func, expression)
            .and_then(|e| e.ok_or(anyhow!("Using a void value as expression")))
    }

    pub fn gen_expression(
        &self,
        func: &mut Function<'ctx>,
        expression: Expression,
    ) -> Result<Option<BasicValueEnum>> {
        Ok(match expression {
            Expression::Literal { value, r#type } => Some(match r#type {
                LiteralType::String => self
                    .ctx
                    .const_string(&value[1..value.len() - 1].as_bytes(), true)
                    .into(),

                LiteralType::Rune => self
                    .ctx
                    .i8_type()
                    .const_int(value.as_bytes()[1].into(), false)
                    .into(),

                LiteralType::Int => self
                    .ctx
                    .i32_type()
                    .const_int(value.parse().unwrap(), false)
                    .into(),

                LiteralType::Float => self
                    .ctx
                    .f64_type()
                    .const_float(value.parse().unwrap())
                    .into(),
            }),
            Expression::Reference(expr) => {
                // self.builder.build_alloca(self.type, name);

                // self.gen_expression(func, *expr).unwrap().as_basic_value_enum();

                // return Some(Box::new(func.stack.get(&path[0]).unwrap().1));

                todo!()
            }
            Expression::Dereference(expr) => {
                // let inner = self.gen_expression(func, expr).unwrap();

                todo!()
            }
            Expression::Path(path) => {
                let name = path.last().unwrap();

                let lookup = *func
                    .stack
                    .get(name)
                    .ok_or_else(|| anyhow!("Identifier `{}` not found", name))?;

                Some(self.builder.build_load(lookup.0, lookup.1, &path[0]))
            }
            Expression::Binary(n) => {
                let Node::Compound(..) = n else {
                    unreachable!()
                };

                Some(self.gen_binary(func, n)?)
            }
            Expression::Call { path, args } => {
                let name = path.last().unwrap();

                let function = self
                    .module
                    .get_function(name)
                    .ok_or_else(|| anyhow!("Function `{}` not found", name))?;

                let ret = self.builder.build_call(
                    function,
                    args.into_iter()
                        .map(|e| self.gen_non_void_expression(func, e).map(|v| v.into()))
                        .collect::<Result<Vec<_>>>()?
                        .as_slice(),
                    "call",
                );

                let ret = ret.try_as_basic_value();

                // TODO make this shit better, fucking lifetimes lol

                if ret.is_left() {
                    Some(ret.unwrap_left())
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
