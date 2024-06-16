use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use compiler_parser::Statement;

use crate::{Codegen, Function, Type, Value};

impl<'ctx> Codegen<'ctx> {
    pub fn gen_statement(
        &self,
        func: Rc<RefCell<Function<'ctx>>>,
        statement: Statement,
    ) -> Result<()> {
        match statement {
            Statement::Return(e) => {
                let ret = e
                    .and_then(|e| self.gen_expression(func, e).transpose())
                    .transpose()?;

                self.builder
                    .build_return(ret.map(|v| Box::new(v.inner) as Box<_>).as_deref())?;

                Ok(())
            }
            Statement::Expression(e) => self.gen_expression(func, e).map(|_| ()),
            Statement::Assign {
                destination,
                source,
            } => {
                let destination = self.gen_non_void_expression(Rc::clone(&func), destination)?;

                self.builder.build_store(
                    destination.inner.into_pointer_value(),
                    self.gen_non_void_expression(func, source)?.inner,
                )?;

                Ok(())
            }
            Statement::Local {
                mutable: _,
                name,
                init,
            } => {
                // TODO impl mut
                let r#type = Type::try_from(name.1.unwrap())?;

                let alloc = self
                    .builder
                    .build_alloca(r#type.as_llvm_basic_type(&self.ctx)?, &name.0)?;

                func.borrow_mut().stack.insert(
                    name.0,
                    Value {
                        inner: alloc.into(),
                        r#type,
                    },
                );

                if let Some(init) = init {
                    self.builder
                        .build_store(alloc, self.gen_non_void_expression(func, init)?.inner)?;
                }

                Ok(())
            }
        }
    }
}
