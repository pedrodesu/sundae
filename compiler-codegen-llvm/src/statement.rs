use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use compiler_parser::Statement;
use inkwell::values::BasicValue;

use crate::{Codegen, Function, Type, Value};

impl<'ctx> Codegen<'ctx>
{
    pub fn gen_statement(
        &self,
        parent_func: &Option<Rc<RefCell<Function<'ctx>>>>,
        statement: Statement,
    ) -> Result<()>
    {
        match statement
        {
            Statement::Return(e) =>
            {
                let ret = e
                    .and_then(|e| self.gen_expression(parent_func, e).transpose())
                    .transpose()?;

                self.builder.build_return(
                    ret.map(|v| {
                        Ok::<Box<dyn BasicValue<'_>>, anyhow::Error>(Box::new(
                            // this generic should be implicit?
                            self.ref_cast(
                                v,
                                parent_func.as_ref().unwrap().borrow().return_type.clone(),
                            )?
                            .inner,
                        )
                            as Box<dyn BasicValue>)
                    })
                    .transpose()?
                    .as_deref(),
                )?;

                Ok(())
            }
            Statement::Expression(e) => self.gen_expression(parent_func, e).map(|_| ()),
            Statement::Assign {
                destination,
                source,
            } =>
            {
                let destination = self
                    .gen_non_void_expression(&parent_func.as_ref().map(Rc::clone), destination)?;

                let source = self.gen_non_void_expression(parent_func, source)?;

                self.builder.build_store(
                    destination.inner.into_pointer_value(),
                    if let Type::Ref(box inner_type) | Type::MutRef(box inner_type) = source.r#type
                    {
                        let inner_ptr = source.inner.into_pointer_value();

                        self.builder.build_load(
                            inner_type.as_llvm_basic_type(self.ctx)?,
                            inner_ptr,
                            "load",
                        )?
                    }
                    else
                    {
                        source.inner
                    },
                )?;

                Ok(())
            }
            Statement::Local {
                mutable: _,
                name,
                init,
            } =>
            {
                // TODO impl mut
                let r#type = Type::try_from(name.1.unwrap())?;

                let alloc = self
                    .builder
                    .build_alloca(r#type.as_llvm_basic_type(self.ctx)?, &name.0)?;

                parent_func.as_ref().unwrap().borrow_mut().stack.insert(
                    name.0,
                    Value {
                        inner: alloc.into(),
                        r#type: Type::MutRef(Box::new(r#type.clone())),
                    },
                );

                if let Some(init) = init
                {
                    self.builder.build_store(
                        alloc,
                        self.ref_cast(self.gen_non_void_expression(parent_func, init)?, r#type)?
                            .inner,
                    )?;
                }

                Ok(())
            }
        }
    }
}
