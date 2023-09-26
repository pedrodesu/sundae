use crate::parser::statement::Statement;

use super::{types::Type, Codegen, Function};

use anyhow::Result;

impl<'ctx> Codegen<'ctx> {
    pub fn gen_statement(&self, func: &mut Function<'ctx>, statement: Statement) -> Result<()> {
        match statement {
            Statement::Return(e) => {
                if let Some(ret) = e
                    .map(|e| self.gen_non_void_expression(func, e))
                    .transpose()?
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
                // TODO var assumes is a ptr value. is that correct?
                let var = self
                    .gen_non_void_expression(func, destination)?
                    .into_pointer_value();

                self.builder
                    .build_store(var, self.gen_non_void_expression(func, source)?);
            }
            Statement::Local {
                mutable,
                name,
                init,
            } => {
                // TODO impl mut
                let r#type = Type::try_from(name.1.unwrap())
                    .unwrap()
                    .into_basic_type(self.ctx)
                    .unwrap();

                let alloc = self.builder.build_alloca(r#type, name.0.as_str());

                func.stack.insert(name.0, (r#type, alloc));

                if let Some(init) = init {
                    self.builder
                        .build_store(alloc, self.gen_non_void_expression(func, init)?);
                }
            }
        }

        Ok(())
    }
}
