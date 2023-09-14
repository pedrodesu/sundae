use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::{BasicType, BasicTypeEnum, FloatType, IntType, VoidType},
    values::{AnyValue, AsValueRef, BasicValue, BasicValueEnum, FunctionValue},
    IntPredicate,
};

use crate::{
    lexer::definitions::LiteralType,
    parser::{
        expression::{
            binary::{Node, Operator},
            Expression,
        },
        item::Item,
        statement::Statement,
        types::{BaseType, Type},
        AST,
    },
};

enum Returnable<'ctx> {
    BasicType(BasicTypeEnum<'ctx>),
    VoidType(VoidType<'ctx>),
}

struct Codegen<'ctx> {
    ctx: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
}

impl<'ctx> Codegen<'ctx> {
    // TODO: implement complex types, such as arrays and custom structs

    #[inline]
    fn int_type(&self, value: &str) -> Option<IntType<'ctx>> {
        match value {
            "bool" => Some(self.ctx.bool_type()),
            "u8" | "i8" => Some(self.ctx.i8_type()),
            "u16" | "i16" => Some(self.ctx.i16_type()),
            "u32" | "i32" => Some(self.ctx.i32_type()),
            "u64" | "i64" => Some(self.ctx.i64_type()),
            "u128" | "i128" => Some(self.ctx.i128_type()),
            _ => {
                let (signedness, bits) = value.split_at(1);
                if matches!(signedness, "u" | "i") {
                    Some(self.ctx.custom_width_int_type(bits.parse::<u32>().ok()?))
                } else {
                    None
                }
            }
        }
    }

    #[inline]
    fn float_type(&self, value: &str) -> Option<FloatType<'ctx>> {
        match value {
            "f16" => Some(self.ctx.f16_type()),
            "f32" => Some(self.ctx.f32_type()),
            "f64" => Some(self.ctx.f64_type()),
            "f128" => Some(self.ctx.f128_type()),
            _ => None,
        }
    }

    #[inline]
    fn void_type(&self, value: &str) -> Option<VoidType<'ctx>> {
        if value == "void" {
            Some(self.ctx.void_type())
        } else {
            None
        }
    }

    #[inline]
    pub fn scalar_basic_type(&self, value: &str) -> Option<BasicTypeEnum<'ctx>> {
        if let Some(t) = self.int_type(value) {
            Some(t.as_basic_type_enum())
        } else if let Some(t) = self.float_type(value) {
            Some(t.as_basic_type_enum())
        } else {
            None
        }
    }

    #[inline]
    pub fn basic_type(&self, value: Type) -> Option<BasicTypeEnum<'ctx>> {
        // implement stuff with consts and what not
        match value.base {
            BaseType::Array { .. } => todo!(),
            BaseType::Scalar { r#type } => self.scalar_basic_type(r#type.as_str()),
        }
    }

    #[inline]
    pub fn returnable_type(&self, value: Type) -> Option<Returnable<'ctx>> {
        match value.base {
            BaseType::Array { .. } => todo!(),
            BaseType::Scalar { r#type } if r#type == "void" => {
                Some(Returnable::VoidType(self.ctx.void_type()))
            }
            BaseType::Scalar { r#type } => self
                .scalar_basic_type(r#type.as_str())
                .map(|t| Returnable::BasicType(t)),
        }
    }

    fn gen_binary(
        &self,
        func: FunctionValue<'ctx>,
        node: Node,
    ) -> Box<dyn BasicValue<'ctx> + 'ctx> {
        match node {
            Node::Scalar(s) => self.gen_expression(func, *s).unwrap(),
            Node::Compound(l, op, r) => {
                let eval_side = |side: Box<Node>| match *side {
                    Node::Scalar(node) => self.gen_expression(func, *node).unwrap(),
                    node @ Node::Compound(..) => self.gen_binary(func, node),
                };

                let (l, r) = (
                    eval_side(l).as_basic_value_enum().into_int_value(),
                    eval_side(r).as_basic_value_enum().into_int_value(),
                );

                // TODO check for and generate according instructions for fp operands
                // same thing for signed and unsigned predicate types

                // TODO read abt difference on mul and div variants and what not
                // and on E/U abt NaN
                match op {
                    Operator::Sum => Box::new(self.builder.build_int_add(l, r, "sum")),
                    Operator::Sub => Box::new(self.builder.build_int_sub(l, r, "sub")),
                    Operator::Star => Box::new(self.builder.build_int_mul(l, r, "mul")),
                    Operator::Div => Box::new(self.builder.build_int_signed_div(l, r, "div")),
                    // LLVM doesn't implement && or ||, rather they work like & and | to bools (i1)
                    Operator::And | Operator::ShAnd => {
                        Box::new(self.builder.build_and(l, r, "and"))
                    }
                    Operator::Or | Operator::ShOr => Box::new(self.builder.build_or(l, r, "or")),
                    Operator::Shl => Box::new(self.builder.build_left_shift(l, r, "shl")),
                    // Logical (LShr) vs Arithmetic (AShr) Right Shift
                    Operator::Shr => Box::new(self.builder.build_right_shift(l, r, false, "shr")),
                    Operator::Xor => Box::new(self.builder.build_xor(l, r, "xor")),
                    Operator::Lt => {
                        Box::new(
                            self.builder
                                .build_int_compare(IntPredicate::SLT, l, r, "lt"),
                        )
                    }
                    Operator::Gt => {
                        Box::new(
                            self.builder
                                .build_int_compare(IntPredicate::SGT, l, r, "gt"),
                        )
                    }
                    Operator::Le => {
                        Box::new(
                            self.builder
                                .build_int_compare(IntPredicate::SLE, l, r, "le"),
                        )
                    }
                    Operator::Ge => {
                        Box::new(
                            self.builder
                                .build_int_compare(IntPredicate::SGE, l, r, "ge"),
                        )
                    }
                    Operator::EqEq => {
                        Box::new(
                            self.builder
                                .build_int_compare(IntPredicate::EQ, l, r, "eqeq"),
                        )
                    }
                    Operator::Neq => {
                        Box::new(
                            self.builder
                                .build_int_compare(IntPredicate::NE, l, r, "neq"),
                        )
                    }
                }
            }
        }
    }

    fn gen_expression(
        &self,
        func: FunctionValue<'ctx>,
        expression: Expression,
    ) -> Option<Box<dyn BasicValue<'ctx> + 'ctx>> {
        match expression {
            Expression::Literal { value, r#type } => match r#type {
                LiteralType::String => Some(Box::new(
                    self.ctx
                        .const_string(&value[1..value.len() - 1].as_bytes(), false),
                )),
                LiteralType::Rune => Some(Box::new(
                    self.ctx
                        .i8_type()
                        .const_int(value.parse::<u64>().unwrap(), false),
                )),
                LiteralType::Int => Some(Box::new(
                    self.ctx
                        .i32_type()
                        .const_int(value.parse::<u64>().unwrap(), false),
                )),
                LiteralType::Float => Some(Box::new(
                    self.ctx
                        .f64_type()
                        .const_float(value.parse::<f64>().unwrap()),
                )),
                LiteralType::Bool => Some(Box::new(
                    self.ctx
                        .bool_type()
                        .const_int(if value == "true" { 1 } else { 0 }, false),
                )),
            },
            Expression::Path(_) => Some(Box::new(func.get_nth_param(0).unwrap())),
            Expression::Reference { r#mut, value } => todo!(),
            Expression::Dereference { value } => todo!(),
            Expression::Binary(n) => match n {
                Node::Scalar(s) => self.gen_expression(func, *s),
                n @ Node::Compound(..) => Some(self.gen_binary(func, n)),
            },
            Expression::Call { path, args } => {
                let ret = self.builder.build_call(
                    self.module.get_function(path[0].as_str()).unwrap(),
                    args.into_iter()
                        .map(|e| {
                            self.gen_expression(func, e)
                                .unwrap()
                                .as_basic_value_enum()
                                .into()
                        })
                        .collect::<Vec<_>>()
                        .as_slice(),
                    "call",
                );

                Some(Box::new(ret.try_as_basic_value().unwrap_left()))
            }
            Expression::If {
                condition,
                block,
                else_block,
            } => {
                let then = self.ctx.append_basic_block(func, "then");
                let r#else = self.ctx.append_basic_block(func, "else");

                // TODO optimise else conds
                let r#continue = if else_block.is_none() {
                    Some(self.ctx.append_basic_block(func, "continue"))
                } else {
                    None
                };

                self.builder.position_at_end(then);

                block.into_iter().for_each(|s| self.gen_statement(func, s));
                if let Some(r#continue) = r#continue {
                    self.builder.build_unconditional_branch(r#continue);
                }

                self.builder.position_at_end(r#else);

                if let Some(r#continue) = r#continue {
                    self.builder.build_unconditional_branch(r#continue);
                } else {
                    else_block
                        .unwrap()
                        .into_iter()
                        .for_each(|s| self.gen_statement(func, s));
                }

                self.builder
                    .position_at_end(func.get_first_basic_block().unwrap());

                self.builder.build_conditional_branch(
                    self.gen_expression(func, *condition)
                        .unwrap()
                        .as_basic_value_enum()
                        .into_int_value(),
                    then,
                    r#else,
                );

                if let Some(r#continue) = r#continue {
                    self.builder.position_at_end(r#continue);
                }

                None
            }
        }
    }

    fn gen_statement(&self, func: FunctionValue<'ctx>, statement: Statement) {
        match statement {
            Statement::Return(e) => {
                self.builder
                    .build_return(e.and_then(|e| self.gen_expression(func, e)).as_deref());
            }
            Statement::Expression(e) => {
                self.gen_expression(func, e);
            }
            Statement::Assign {
                destination,
                source,
            } => todo!(),
            Statement::Local {
                mutable,
                name,
                init,
            } => todo!(),
        }
    }

    pub fn gen_item(&self, item: Item) {
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
                    .and_then(|v| self.returnable_type(v))
                    .unwrap_or_else(|| Returnable::VoidType(self.ctx.void_type()));

                let params = signature
                    .arguments
                    .into_iter()
                    .map(|a| self.basic_type(a.1).unwrap().into())
                    .collect::<Vec<_>>();

                let func_type = match ret_type {
                    Returnable::BasicType(t) => t.fn_type(params.as_slice(), false),
                    Returnable::VoidType(t) => t.fn_type(params.as_slice(), false),
                };

                let func = self
                    .module
                    .add_function(signature.name.0.as_str(), func_type, None);

                let block = self.ctx.append_basic_block(func, "entry");
                self.builder.position_at_end(block);

                body.into_iter().for_each(|s| self.gen_statement(func, s));
            }
        }
    }
}

pub fn gen(module: &str, ast: AST) {
    let ctx = Context::create();

    let codegen = Codegen {
        ctx: &ctx,
        module: ctx.create_module(module),
        builder: ctx.create_builder(),
    };

    // add library?
    /*
    unsafe {
        println!(
            "{}",
            LLVMLoadLibraryPermanently("target/debug/libsundae_library.so\0".as_ptr() as _)
        );
        let func_type = unsafe {
            LLVMFunctionType(
                LLVMVoidType(),
                [LLVMPointerTypeInContext(*CTX, 4)].as_mut_ptr(),
                1,
                false as _,
            )
        };
        LLVMAddFunction(r#mod(), "println\0".as_ptr() as _, func_type);
    }
    */

    ast.0.into_iter().for_each(|i| codegen.gen_item(i));

    codegen
        .module
        .print_to_file(format!("{module}.ll"))
        .unwrap();
}

// TODO remain work on codegen and also modularise it
// also really focus on error handling, this shit cannot pass like this
// and we're creating code which will break without warning with an invalid syntax
