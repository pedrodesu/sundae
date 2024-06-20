use std::{cell::RefCell, rc::Rc};

use anyhow::{bail, Result};
use inkwell::{
    types::{AnyTypeEnum, BasicType},
    values::BasicValue,
};

use crate::{Codegen, Function, Type};
use compiler_parser::Item;

impl<'ctx> Codegen<'ctx> {
    pub fn gen_item(&self, item: Item) -> Result<()> {
        match item {
            Item::Const { .. } => {
                /*
                let r#type = self.basic_type(&name.1).unwrap();
                let r#const = codegen
                    .module
                    .add_global(r#type.as_basic_type_enum(), None, &name.0);

                let value = r#type.as_basic_type_enum().const_zero();

                // TODO find best way to store types.
                r#const.set_initializer(&value);

                /*
                r#type
                    .as_any_type_enum()
                    .into_int_type()
                    .const_int(42, true);
                */

                // r#const.set_initializer(&codegen.ctx.const_string(b"sex", false));
                */
                todo!()
            }
            Item::Function { signature, body } => {
                // TODO should replace all "default" 32 with ptr type? make sure..
                let return_type = if signature.name.0 == "main" {
                    Type::Integer {
                        width: 32,
                        signed: true,
                    }
                } else {
                    signature
                        .name
                        .1
                        .clone()
                        .map(Type::try_from)
                        .transpose()?
                        .unwrap_or_default()
                };

                let arguments = signature
                    .arguments
                    .clone()
                    .into_iter()
                    .map(|a| Ok((a.0, Type::try_from(a.1)?)))
                    .collect::<Result<Vec<_>>>()?;

                let inner = {
                    // TODO should be better
                    let fn_type = if let Type::Void = return_type {
                        return_type
                            .as_llvm_any_type(self.ctx)
                            .into_void_type()
                            .fn_type(
                                arguments
                                    .iter()
                                    .map(|(_, t)| t.as_llvm_basic_type(self.ctx).map(|t| t.into()))
                                    .collect::<Result<Vec<_>>>()?
                                    .as_slice(),
                                false,
                            )
                    } else {
                        return_type.as_llvm_basic_type(self.ctx)?.fn_type(
                            arguments
                                .iter()
                                .map(|(_, t)| t.as_llvm_basic_type(self.ctx).map(|t| t.into()))
                                .collect::<Result<Vec<_>>>()?
                                .as_slice(),
                            false,
                        )
                    };

                    self.module
                        .add_function(signature.name.0.as_str(), fn_type, None)
                };

                let function = Rc::new(RefCell::new(Function {
                    arguments,
                    return_type: return_type.clone(),
                    stack: Default::default(),
                    inner,
                }));

                function.borrow_mut().init_block(self);

                function.borrow_mut().init_args_stack(self)?;

                for statement in body {
                    self.gen_statement(Rc::clone(&function), statement.clone())?;
                }

                if signature.name.0 == "main" {
                    let ret =
                        if let AnyTypeEnum::IntType(v) = return_type.as_llvm_any_type(self.ctx) {
                            Some(Box::new(v.const_zero()) as Box<dyn BasicValue>)
                        } else {
                            bail!("type {return_type:?} can't be converted to a integer type")
                        };

                    self.builder.build_return(ret.as_deref())?;
                } else if signature.name.1.is_none() {
                    self.builder.build_return(None)?;
                }

                self.runtime
                    .borrow_mut()
                    .functions
                    .insert(signature.name.0.clone(), function);

                Ok(())
            }
        }
    }
}
