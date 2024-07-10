use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use compiler_parser::{Node, Operator};
use inkwell::{values::BasicValue, IntPredicate};

use crate::{Codegen, Function, Type, Value};

impl<'ctx> Codegen<'ctx> {
    pub fn gen_binary(
        &self,
        parent_func: Option<&Rc<RefCell<Function<'ctx>>>>,
        node: Node,
    ) -> Result<Value<'ctx>> {
        let Node::Compound(box l, op, box r) = node else {
            unreachable!()
        };

        let eval_side = |parent_func, node| {
            anyhow::Ok(
                match node {
                    Node::Scalar(box node) => self.gen_non_void_expression(parent_func, node)?,
                    node @ Node::Compound(..) => self.gen_binary(parent_func, node)?,
                }
                .inner
                .into_int_value(),
            )
            // TODO ^ gotta do checking before, as on other places
        };

        let binding = Rc::clone(parent_func.unwrap());
        let l = eval_side(Some(&binding), l)?;
        let r = eval_side(parent_func, r)?;

        // TODO check for and generate according instructions for fp operands
        // same thing for signed and unsigned predicate types

        // TODO read abt difference on E/U abt NaN

        let value = match op {
            Operator::Plus => self.builder.build_int_add(l, r, "sum"),
            Operator::Minus => self.builder.build_int_sub(l, r, "sub"),
            Operator::Star => self.builder.build_int_mul(l, r, "mul"),
            Operator::Div => self.builder.build_int_signed_div(l, r, "div"),
            // LLVM doesn't implement && or ||, rather they work like & and | to bools (i1)
            Operator::And | Operator::ShAnd => self.builder.build_and(l, r, "and"),
            Operator::Or | Operator::ShOr => self.builder.build_or(l, r, "or"),
            Operator::Shl => self.builder.build_left_shift(l, r, "shl"),
            Operator::Shr => self.builder.build_right_shift(l, r, true, "shr"),
            Operator::Xor => self.builder.build_xor(l, r, "xor"),
            Operator::Lt => self
                .builder
                .build_int_compare(IntPredicate::SLT, l, r, "lt"),
            Operator::Gt => self
                .builder
                .build_int_compare(IntPredicate::SGT, l, r, "gt"),
            Operator::Le => self
                .builder
                .build_int_compare(IntPredicate::SLE, l, r, "le"),
            Operator::Ge => self
                .builder
                .build_int_compare(IntPredicate::SGE, l, r, "ge"),
            Operator::EqEq => self
                .builder
                .build_int_compare(IntPredicate::EQ, l, r, "eqeq"),
            Operator::Neq => self
                .builder
                .build_int_compare(IntPredicate::NE, l, r, "neq"),
        }?;

        // TODO use correct type
        Ok(Value {
            inner: value.as_basic_value_enum(),
            r#type: Type::Integer {
                width: 32,
                signed: true,
            },
        })
    }
}
