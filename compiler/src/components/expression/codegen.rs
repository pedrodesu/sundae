use std::ffi::CString;

use anyhow::{anyhow, bail, Result};
use llvm_sys::{
    core::{
        LLVMAppendBasicBlockInContext, LLVMBuildBr, LLVMBuildCall2, LLVMBuildCondBr, LLVMConstInt,
        LLVMConstStringInContext, LLVMDoubleTypeInContext, LLVMGetFirstBasicBlock, LLVMGetTypeKind,
        LLVMInt32TypeInContext, LLVMInt8TypeInContext, LLVMPositionBuilderAtEnd, LLVMTypeOf,
    },
    LLVMTypeKind,
};

use crate::{
    codegen::{Codegen, Function, Value},
    components::codegen_types::Type,
    lexer::definitions::LiteralType,
};

use super::Expression;

impl Codegen {
    #[inline]
    pub fn gen_non_void_expression(
        &self,
        func: &mut Function,
        expression: Expression,
    ) -> Result<Value> {
        self.gen_expression(func, expression)
            .and_then(|e| e.ok_or(anyhow!("Using a void value as expression")))
    }

    pub fn gen_expression(
        &self,
        func: &mut Function,
        expression: Expression,
    ) -> Result<Option<Value>> {
        Ok(match expression {
            Expression::Literal { value, r#type } => Some(match r#type {
                LiteralType::String => {
                    let bytes = &value[1..value.len() - 1];

                    Value {
                        inner: unsafe {
                            let content = CString::new(bytes).unwrap();
                            LLVMConstStringInContext(
                                self.ctx,
                                content.as_ptr(),
                                bytes.len() as _,
                                true as _,
                            )
                        },
                        r#type: Type::Array {
                            scalar: Box::new(Type::Integer {
                                width: 8,
                                signed: true,
                            }),
                            size: bytes.len(),
                        },
                    }
                }
                LiteralType::Rune => Value {
                    inner: unsafe {
                        LLVMConstInt(
                            LLVMInt8TypeInContext(self.ctx),
                            value.as_bytes()[1] as _,
                            false as _,
                        )
                    },
                    r#type: Type::Integer {
                        width: 8,
                        signed: true,
                    },
                },
                LiteralType::Int => Value {
                    inner: unsafe {
                        LLVMConstInt(
                            LLVMInt32TypeInContext(self.ctx),
                            value.parse::<i32>().unwrap() as _,
                            false as _,
                        )
                    },
                    r#type: Type::Integer {
                        width: 32,
                        signed: true,
                    },
                },
                LiteralType::Float => Value {
                    inner: unsafe {
                        LLVMConstInt(
                            LLVMDoubleTypeInContext(self.ctx),
                            value.parse::<f64>().unwrap() as _,
                            false as _,
                        )
                    },
                    r#type: Type::Float(64),
                },
            }),
            Expression::Path(path) => {
                let name = path.last().unwrap();

                let lookup = func
                    .stack
                    .get(name)
                    .ok_or_else(|| anyhow!("Identifier `{}` not found", name))?
                    .clone();

                Some(lookup)
            }
            Expression::Binary(n) => Some(self.gen_binary(func, n)?),
            Expression::Call { path, args } => {
                let name = path.last().unwrap();

                let Some(function) = self.runtime.functions.get(name) else {
                    bail!("Function `{}` not found", name);
                };

                let ret = unsafe {
                    LLVMBuildCall2(
                        self.builder,
                        function.return_type.get_type(self.ctx),
                        function.inner,
                        args.clone()
                            .into_iter()
                            .enumerate()
                            .map(|(i, e)| {
                                let value = self.gen_non_void_expression(func, e)?;

                                let Some(decl_type) = function.arguments.get(i) else {
                                    bail!(
                                        "Function `{}` expects {} arguments",
                                        name,
                                        function.arguments.len()
                                    );
                                };

                                if value.r#type != decl_type.1 {
                                    bail!(
                                        "Function `{}` argument `{}` asks for a {:?}, got {:?}",
                                        name,
                                        decl_type.0,
                                        decl_type.1,
                                        value.r#type
                                    );
                                }

                                Ok(value.inner)
                            })
                            .collect::<Result<Vec<_>>>()?
                            .as_mut_ptr(),
                        args.len() as _,
                        "call\0".as_ptr() as _,
                    )
                };
                println!("2");
                // segfault on buildcall2?????????

                if unsafe {
                    !matches!(
                        LLVMGetTypeKind(LLVMTypeOf(ret)),
                        LLVMTypeKind::LLVMVoidTypeKind
                    )
                } {
                    Some(Value {
                        inner: ret,
                        r#type: self
                            .runtime
                            .functions
                            .get(name)
                            .unwrap()
                            .return_type
                            .clone(),
                    })
                } else {
                    None
                }
            }
            Expression::If {
                condition,
                block,
                else_block,
            } => {
                let then = unsafe {
                    LLVMAppendBasicBlockInContext(self.ctx, func.inner, "then\0".as_ptr() as _)
                };
                let r#else = unsafe {
                    LLVMAppendBasicBlockInContext(self.ctx, func.inner, "else\0".as_ptr() as _)
                };

                // TODO optimise else conds
                let r#continue = if else_block.is_none() {
                    Some(unsafe {
                        LLVMAppendBasicBlockInContext(
                            self.ctx,
                            func.inner,
                            "continue\0".as_ptr() as _,
                        )
                    })
                } else {
                    None
                };

                unsafe {
                    LLVMPositionBuilderAtEnd(self.builder, then);
                }

                for statement in block {
                    self.gen_statement(func, statement)?;
                }

                if let Some(r#continue) = r#continue {
                    unsafe {
                        LLVMBuildBr(self.builder, r#continue);
                    }
                }

                unsafe {
                    LLVMPositionBuilderAtEnd(self.builder, r#else);
                }

                if let Some(r#continue) = r#continue {
                    unsafe {
                        LLVMBuildBr(self.builder, r#continue);
                    }
                } else {
                    else_block
                        .unwrap()
                        .into_iter()
                        .map(|s| self.gen_statement(func, s))
                        .collect::<Result<_>>()?;
                }

                unsafe {
                    LLVMPositionBuilderAtEnd(self.builder, LLVMGetFirstBasicBlock(func.inner));

                    LLVMBuildCondBr(
                        self.builder,
                        self.gen_non_void_expression(func, *condition)?.inner,
                        then,
                        r#else,
                    );
                }

                if let Some(r#continue) = r#continue {
                    unsafe { LLVMPositionBuilderAtEnd(self.builder, r#continue) };
                }

                None
            }
        })
    }
}
