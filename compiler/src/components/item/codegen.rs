use std::{collections::HashMap, ffi::CString};

use crate::{
    codegen::Codegen,
    components::codegen_types::{Function, Type, Value},
};

use anyhow::Result;
use llvm_sys::core::{
    LLVMAddFunction, LLVMAppendBasicBlockInContext, LLVMBuildAlloca, LLVMBuildRetVoid,
    LLVMBuildStore, LLVMFunctionType, LLVMGetParam, LLVMPositionBuilderAtEnd,
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
                let ret_type = signature
                    .name
                    .1
                    .clone()
                    .map(|v| Type::try_from(v))
                    .transpose()?
                    .unwrap_or_else(|| Type::Void);

                self.functions
                    .insert(signature.name.0.clone(), ret_type.clone());

                let mut params = signature
                    .arguments
                    .clone()
                    .into_iter()
                    .map(|a| Ok(Type::try_from(a.1)?.get_type(self.ctx)))
                    .collect::<Result<Vec<_>>>()?;

                let func = unsafe {
                    let func_type = LLVMFunctionType(
                        ret_type.get_type(self.ctx),
                        params.as_mut_ptr(),
                        params.len() as _,
                        false as _,
                    );

                    let name = CString::new(signature.name.0).unwrap();

                    LLVMAddFunction(self.module, name.as_ptr(), func_type)
                };

                let mut wrap = Function {
                    inner: func,
                    stack: HashMap::new(),
                };

                unsafe {
                    let block =
                        LLVMAppendBasicBlockInContext(self.ctx, func, "entry\0".as_ptr() as _);
                    LLVMPositionBuilderAtEnd(self.builder, block);
                }

                signature
                    .arguments
                    .into_iter()
                    .enumerate()
                    .map(|(i, a)| {
                        let arg = unsafe { LLVMGetParam(func, i as _) };

                        let r#type = Type::try_from(a.1)?;

                        unsafe {
                            let name = CString::new(a.0.clone()).unwrap();

                            let alloc = LLVMBuildAlloca(
                                self.builder,
                                r#type.get_type(self.ctx),
                                name.as_ptr(),
                            );
                            LLVMBuildStore(self.builder, arg, alloc);

                            wrap.stack.insert(
                                a.0,
                                Value {
                                    inner: alloc,
                                    r#type,
                                },
                            );
                        }

                        Ok(())
                    })
                    .collect::<Result<_>>()?;

                for statement in body {
                    self.gen_statement(&mut wrap, statement)?;
                }

                if signature.name.1.is_none() {
                    unsafe {
                        LLVMBuildRetVoid(self.builder);
                    }
                }

                Ok(())
            }
        }
    }
}
