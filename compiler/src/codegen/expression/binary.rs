use inkwell::{values::BasicValueEnum, IntPredicate};

use crate::parser::expression::binary::{Node, Operator};

use super::{Codegen, Function};

use anyhow::Result;

impl<'ctx> Codegen<'ctx> {
    pub(super) fn gen_binary(
        &self,
        func: &mut Function<'ctx>,
        node: Node,
    ) -> Result<BasicValueEnum> {
        let Node::Compound(l, op, r) = node else {
            unreachable!()
        };

        let mut eval_side = |side: Box<Node>| match *side {
            Node::Scalar(node) => self.gen_non_void_expression(func, *node),
            node @ Node::Compound(..) => self.gen_binary(func, node),
        };

        let (l, r) = (
            Self::assure_int_expr(eval_side(l))?,
            Self::assure_int_expr(eval_side(r))?,
        );

        // TODO check for and generate according instructions for fp operands
        // same thing for signed and unsigned predicate types

        // TODO read abt difference on mul and div variants and what not
        // and on E/U abt NaN
        Ok(match op {
            Operator::Sum => self.builder.build_int_add(l, r, "sum").into(),
            Operator::Sub => self.builder.build_int_sub(l, r, "sub").into(),
            Operator::Star => self.builder.build_int_mul(l, r, "mul").into(),
            Operator::Div => self.builder.build_int_signed_div(l, r, "div").into(),
            // LLVM doesn't implement && or ||, rather they work like & and | to bools (i1)
            Operator::And | Operator::ShAnd => self.builder.build_and(l, r, "and").into(),
            Operator::Or | Operator::ShOr => self.builder.build_or(l, r, "or").into(),
            Operator::Shl => self.builder.build_left_shift(l, r, "shl").into(),
            // Logical (LShr) vs Arithmetic (AShr) Right Shift
            Operator::Shr => self.builder.build_right_shift(l, r, false, "shr").into(),
            Operator::Xor => self.builder.build_xor(l, r, "xor").into(),
            Operator::Lt => self
                .builder
                .build_int_compare(IntPredicate::SLT, l, r, "lt")
                .into(),
            Operator::Gt => self
                .builder
                .build_int_compare(IntPredicate::SGT, l, r, "gt")
                .into(),
            Operator::Le => self
                .builder
                .build_int_compare(IntPredicate::SLE, l, r, "le")
                .into(),
            Operator::Ge => self
                .builder
                .build_int_compare(IntPredicate::SGE, l, r, "ge")
                .into(),
            Operator::EqEq => self
                .builder
                .build_int_compare(IntPredicate::EQ, l, r, "eqeq")
                .into(),
            Operator::Neq => self
                .builder
                .build_int_compare(IntPredicate::NE, l, r, "neq")
                .into(),
        })
    }
}
