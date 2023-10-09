use std::ffi::CString;

use anyhow::{anyhow, bail, Result};
use llvm_sys::{
    core::{
        LLVMAppendBasicBlockInContext, LLVMBuildAlloca, LLVMBuildBr, LLVMBuildCall2,
        LLVMBuildCondBr, LLVMBuildLoad2, LLVMBuildStore, LLVMConstInt, LLVMConstStringInContext,
        LLVMDoubleTypeInContext, LLVMGetFirstBasicBlock, LLVMGetNamedFunction, LLVMGetTypeKind,
        LLVMGlobalGetValueType, LLVMInt32TypeInContext, LLVMInt8TypeInContext,
        LLVMPositionBuilderAtEnd, LLVMTypeOf,
    },
    LLVMTypeKind,
};

use crate::{
    codegen::Codegen,
    components::codegen_types::{Function, Type, Value},
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
            Expression::Reference(expr) => {
                let expr = self.gen_non_void_expression(func, *expr)?;

                unsafe {
                    let alloc = LLVMBuildAlloca(
                        self.builder,
                        expr.r#type.get_type(self.ctx),
                        "ref\0".as_ptr() as _,
                    );
                    LLVMBuildStore(self.builder, expr.inner, alloc);

                    Some(Value {
                        inner: alloc,
                        r#type: Type::Pointer(Box::new(expr.r#type)),
                    })
                }
            }
            Expression::Dereference(expr) => {
                let expr = self.gen_non_void_expression(func, *expr)?;

                let pointee_type = if let Type::Pointer(inner) = expr.r#type {
                    *inner
                } else {
                    bail!("Trying to dereference non-reference")
                };

                let value = unsafe {
                    LLVMBuildLoad2(
                        self.builder,
                        pointee_type.get_type(self.ctx),
                        expr.inner,
                        "deref\0".as_ptr() as _,
                    )
                };

                Some(Value {
                    inner: value,
                    r#type: pointee_type,
                })
            }
            Expression::Path(path) => {
                let name = path.last().unwrap();

                let lookup = func
                    .stack
                    .get(name)
                    .ok_or_else(|| anyhow!("Identifier `{}` not found", name))?
                    .clone();

                Some(Value {
                    inner: unsafe {
                        let name = CString::new(name.as_str()).unwrap();
                        LLVMBuildLoad2(
                            self.builder,
                            lookup.r#type.get_type(self.ctx),
                            lookup.inner,
                            name.as_ptr(),
                        )
                    },
                    r#type: lookup.r#type,
                })
            }
            Expression::Binary(n) => Some(self.gen_binary(func, n)?),
            Expression::Call { path, args } => {
                let name = path.last().unwrap();

                let function = unsafe {
                    let name = CString::new(name.as_str()).unwrap();
                    LLVMGetNamedFunction(self.module, name.as_ptr())
                };

                if function.is_null() {
                    bail!("Function `{}` not found", name);
                }

                let ret = unsafe {
                    LLVMBuildCall2(
                        self.builder,
                        LLVMGlobalGetValueType(function),
                        function,
                        args.clone()
                            .into_iter()
                            .map(|e| {
                                self.gen_non_void_expression(func, e)
                                    .map(|v| v.inner.into())
                            })
                            .collect::<Result<Vec<_>>>()?
                            .as_mut_ptr(),
                        args.len() as _,
                        "call\0".as_ptr() as _,
                    )
                };

                // TODO make this shit better, fucking lifetimes lol
                if unsafe {
                    !matches!(
                        LLVMGetTypeKind(LLVMTypeOf(ret)),
                        LLVMTypeKind::LLVMVoidTypeKind
                    )
                } {
                    Some(Value {
                        inner: ret,
                        r#type: self.functions.get(name).unwrap().clone(),
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
