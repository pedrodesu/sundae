use std::ffi::CString;

use anyhow::Result;
use llvm_sys::core::{LLVMBuildAlloca, LLVMBuildRet, LLVMBuildStore};

use crate::{
    codegen::{Codegen, Function, Value},
    components::codegen_types::Type,
};

use super::Statement;

impl Codegen {
    pub fn gen_statement(&self, func: &mut Function, statement: Statement) -> Result<()> {
        match statement {
            Statement::Return(e) => {
                if let Some(e) = e
                    .and_then(|e| self.gen_expression(func, e).transpose())
                    .transpose()?
                {
                    unsafe {
                        LLVMBuildRet(self.builder, e.inner);
                    }
                }
            }
            Statement::Expression(e) => {
                self.gen_expression(func, e)?;
            }
            Statement::Assign {
                destination,
                source,
            } => {
                let destination = self.gen_non_void_expression(func, destination)?;

                unsafe {
                    LLVMBuildStore(
                        self.builder,
                        self.gen_non_void_expression(func, source)?.inner,
                        destination.inner,
                    );
                }
            }
            Statement::Local {
                mutable,
                name,
                init,
            } => {
                // TODO impl mut
                let r#type = Type::try_from(name.1.unwrap())?;

                let alloc = unsafe {
                    let name = CString::new(name.0.clone()).unwrap();
                    LLVMBuildAlloca(self.builder, r#type.get_type(self.ctx), name.as_ptr() as _)
                };

                func.stack.insert(
                    name.0,
                    Value {
                        inner: alloc.into(),
                        r#type,
                    },
                );

                if let Some(init) = init {
                    unsafe {
                        LLVMBuildStore(
                            self.builder,
                            self.gen_non_void_expression(func, init)?.inner,
                            alloc,
                        );
                    }
                }
            }
        }

        Ok(())
    }
}
