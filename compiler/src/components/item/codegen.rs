use std::ffi::CString;

use crate::{
    codegen::{Codegen, Function},
    components::codegen_types::Type,
};

use anyhow::Result;
use llvm_sys::core::{
    LLVMAddFunction, LLVMBuildRet, LLVMBuildRetVoid, LLVMConstInt, LLVMFunctionType,
};

use super::Item;

impl Codegen {
    pub fn gen_item(&mut self, item: Item) -> Result<()> {
        match item {
            Item::Const { name, value } => {
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

                let inner = unsafe {
                    let func_type = LLVMFunctionType(
                        return_type.get_type(self.ctx),
                        arguments
                            .iter()
                            .map(|(_, t)| t.get_type(self.ctx))
                            .collect::<Vec<_>>()
                            .as_mut_ptr(),
                        arguments.len() as _,
                        false as _,
                    );

                    let name = CString::new(signature.name.0.clone()).unwrap();

                    LLVMAddFunction(self.module, name.as_ptr(), func_type)
                };

                let mut function = Function {
                    arguments,
                    return_type,
                    stack: Default::default(),
                    inner,
                };

                self.runtime
                    .functions
                    .insert(signature.name.0.clone(), function.clone());

                function.init_block(self);

                function.init_args_stack(self)?;

                for statement in body {
                    self.gen_statement(&mut function, statement)?;
                }

                if signature.name.0 == "main" {
                    unsafe {
                        LLVMBuildRet(
                            self.builder,
                            LLVMConstInt(function.return_type.get_type(self.ctx), 0, true as _),
                        );
                    }
                } else if signature.name.1.is_none() {
                    unsafe {
                        LLVMBuildRetVoid(self.builder);
                    }
                }

                Ok(())
            }
        }
    }
}
