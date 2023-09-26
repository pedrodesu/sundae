use std::collections::HashMap;

use crate::parser::item::Item;

use super::{types::Type, Codegen, Function};

use anyhow::Result;
use inkwell::types::BasicType;

impl<'ctx> Codegen<'ctx> {
    pub fn gen_item(&self, item: Item) -> Result<()> {
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
            }
            Item::Function { signature, body } => {
                let ret_type = signature
                    .name
                    .1
                    .clone()
                    .and_then(|v| Type::try_from(v).ok())
                    .unwrap_or_else(|| Type::Void);

                let params = signature
                    .arguments
                    .clone()
                    .into_iter()
                    .map(|a| {
                        Type::try_from(a.1)
                            .unwrap()
                            .into_basic_type(self.ctx)
                            .unwrap()
                            .into()
                    })
                    .collect::<Vec<_>>();

                let func_type = match ret_type {
                    Type::Void => ret_type
                        .into_void_type(self.ctx)
                        .unwrap()
                        .fn_type(params.as_slice(), false),
                    _ => ret_type
                        .into_basic_type(self.ctx)
                        .unwrap()
                        .fn_type(params.as_slice(), false),
                };

                let func = self
                    .module
                    .add_function(signature.name.0.as_str(), func_type, None);

                let mut wrap = Function {
                    inner: func,
                    stack: HashMap::new(),
                };

                let block = self.ctx.append_basic_block(func, "entry");
                self.builder.position_at_end(block);

                signature
                    .arguments
                    .into_iter()
                    .enumerate()
                    .for_each(|(i, a)| {
                        let arg = func.get_nth_param(i.try_into().unwrap()).unwrap();

                        let r#type = Type::try_from(a.1)
                            .unwrap()
                            .into_basic_type(self.ctx)
                            .unwrap();

                        let alloc = self.builder.build_alloca(r#type, a.0.as_str());

                        self.builder.build_store(alloc, arg);

                        wrap.stack.insert(a.0, (r#type, alloc));
                    });

                for statement in body {
                    self.gen_statement(&mut wrap, statement)?;
                }

                if signature.name.1.is_none() {
                    self.builder.build_return(None);
                }
            }
        }

        Ok(())
    }
}
