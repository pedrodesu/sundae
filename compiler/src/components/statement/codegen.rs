use anyhow::{bail, Result};
use inkwell::values::BasicValueEnum;

use crate::{
    codegen::Codegen,
    components::codegen_types::{Function, Type, Value},
};

use super::Statement;

impl<'ctx> Codegen<'ctx> {
    pub fn gen_statement(&self, func: &mut Function<'ctx>, statement: Statement) -> Result<()> {
        match statement {
            Statement::Return(e) => {
                if let Some(ret) = e
                    .map(|e| self.gen_non_void_expression(func, e))
                    .transpose()?
                    .map(|v| v.inner)
                {
                    self.builder.build_return(Some(&ret));
                }
            }
            Statement::Expression(e) => {
                self.gen_expression(func, e)?;
            }
            Statement::Assign {
                destination,
                source,
            } => {
                let var = if let BasicValueEnum::PointerValue(v) =
                    self.gen_non_void_expression(func, destination)?.inner
                {
                    v
                } else {
                    bail!("Trying to assign to a non-pointer value")
                };

                self.builder
                    .build_store(var, self.gen_non_void_expression(func, source)?.inner);
            }
            Statement::Local {
                mutable,
                name,
                init,
            } => {
                // TODO impl mut
                let r#type = Type::try_from(name.1.unwrap())?;

                let alloc = self
                    .builder
                    .build_alloca(r#type.get_basic_type(self.ctx)?, name.0.as_str());

                func.stack.insert(
                    name.0,
                    Value {
                        inner: alloc.into(),
                        r#type,
                    },
                );

                if let Some(init) = init {
                    self.builder
                        .build_store(alloc, self.gen_non_void_expression(func, init)?.inner);
                }
            }
        }

        Ok(())
    }
}
