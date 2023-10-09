use anyhow::{anyhow, Result};
use llvm_sys::{
    core::{
        LLVMBuildAdd, LLVMBuildAnd, LLVMBuildICmp, LLVMBuildLShr, LLVMBuildMul, LLVMBuildOr,
        LLVMBuildSDiv, LLVMBuildShl, LLVMBuildSub, LLVMBuildXor,
    },
    LLVMIntPredicate,
};

use crate::{
    codegen::Codegen,
    components::codegen_types::{Function, Type, Value},
};

use super::{Node, Operator};

impl Codegen {
    pub fn gen_binary(&self, func: &mut Function, node: Node) -> Result<Value> {
        let Node::Compound(l, op, r) = node else {
            unreachable!()
        };

        let mut eval_side = |side: Box<Node>| match *side {
            Node::Scalar(node) => self
                .gen_expression(func, *node)
                .map(|v| v.ok_or(anyhow!("Expected return value from expression"))),
            node @ Node::Compound(..) => self.gen_binary(func, node).map(Ok),
        };

        let (l, r) = (eval_side(l)??.inner, eval_side(r)??.inner);

        // TODO check for and generate according instructions for fp operands
        // same thing for signed and unsigned predicate types

        // TODO read abt difference on mul and div variants and what not
        // and on E/U abt NaN
        let value = unsafe {
            match op {
                Operator::Sum => LLVMBuildAdd(self.builder, l, r, "sum\0".as_ptr() as _),
                Operator::Sub => LLVMBuildSub(self.builder, l, r, "sub\0".as_ptr() as _),
                Operator::Star => LLVMBuildMul(self.builder, l, r, "mul\0".as_ptr() as _),
                Operator::Div => LLVMBuildSDiv(self.builder, l, r, "div\0".as_ptr() as _),
                // LLVM doesn't implement && or ||, rather they work like & and | to bools (i1)
                Operator::And | Operator::ShAnd => {
                    LLVMBuildAnd(self.builder, l, r, "and\0".as_ptr() as _)
                }

                Operator::Or | Operator::ShOr => {
                    LLVMBuildOr(self.builder, l, r, "or\0".as_ptr() as _)
                }

                Operator::Shl => LLVMBuildShl(self.builder, l, r, "shl\0".as_ptr() as _),
                // Logical (LShr) vs Arithmetic (AShr) Right Shift
                Operator::Shr => LLVMBuildLShr(self.builder, l, r, "shr\0".as_ptr() as _),
                Operator::Xor => LLVMBuildXor(self.builder, l, r, "xor\0".as_ptr() as _),
                Operator::Lt => LLVMBuildICmp(
                    self.builder,
                    LLVMIntPredicate::LLVMIntSLT,
                    l,
                    r,
                    "lt\0".as_ptr() as _,
                ),
                Operator::Gt => LLVMBuildICmp(
                    self.builder,
                    LLVMIntPredicate::LLVMIntSGT,
                    l,
                    r,
                    "gt\0".as_ptr() as _,
                ),
                Operator::Le => LLVMBuildICmp(
                    self.builder,
                    LLVMIntPredicate::LLVMIntSLE,
                    l,
                    r,
                    "le\0".as_ptr() as _,
                ),
                Operator::Ge => LLVMBuildICmp(
                    self.builder,
                    LLVMIntPredicate::LLVMIntSGE,
                    l,
                    r,
                    "ge\0".as_ptr() as _,
                ),
                Operator::EqEq => LLVMBuildICmp(
                    self.builder,
                    LLVMIntPredicate::LLVMIntEQ,
                    l,
                    r,
                    "eqeq\0".as_ptr() as _,
                ),
                Operator::Neq => LLVMBuildICmp(
                    self.builder,
                    LLVMIntPredicate::LLVMIntNE,
                    l,
                    r,
                    "neq\0".as_ptr() as _,
                ),
            }
        };

        // TODO use correct type
        Ok(Value {
            inner: value,
            r#type: Type::Integer {
                width: 32,
                signed: true,
            },
        })
    }
}
