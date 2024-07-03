use std::{cell::RefCell, rc::Rc};

use anyhow::{bail, Result};
use inkwell::{
    types::{AnyTypeEnum, BasicType},
    values::BasicValue,
};

use crate::{Codegen, Function, Type, Value};
use compiler_parser::Item;

impl<'ctx> Codegen<'ctx> {
    pub fn gen_item(&self, item: Item) -> Result<()> {
        match item {
            Item::Const { name, value } => {
                let r#type = Type::try_from(name.1.unwrap())?;
                let global = self.module.add_global(
                    r#type.as_llvm_basic_type(self.ctx)?,
                    None,
                    name.0.as_str(),
                );

                global.set_constant(true);

                self.runtime.borrow_mut().constants.insert(
                    name.0,
                    Value {
                        r#type,
                        inner: global.as_basic_value_enum(),
                    },
                );

                global.set_initializer(
                    (Box::new(self.gen_non_void_expression(None, value)?.inner)
                        as Box<dyn BasicValue>)
                        .as_ref(),
                );

                Ok(())
            }
            Item::Function { signature, body } => {
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
                    let arguments = arguments
                        .iter()
                        .map(|(_, t)| t.as_llvm_basic_type(self.ctx).map(|t| t.into()))
                        .collect::<Result<Vec<_>>>()?;

                    let fn_type = if let Type::Void = return_type {
                        return_type
                            .as_llvm_any_type(self.ctx)
                            .into_void_type()
                            .fn_type(arguments.as_slice(), false)
                    } else {
                        return_type
                            .as_llvm_basic_type(self.ctx)?
                            .fn_type(arguments.as_slice(), false)
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
                    self.gen_statement(Some(&Rc::clone(&function)), statement.clone())?;
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
