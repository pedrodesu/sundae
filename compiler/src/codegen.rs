use std::{collections::HashMap, ffi::CString};

use llvm_sys::{
    bit_writer::LLVMWriteBitcodeToFile,
    core::{
        LLVMAddFunction, LLVMContextCreate, LLVMContextDispose, LLVMCreateBuilderInContext,
        LLVMDisposeBuilder, LLVMDisposeModule, LLVMFunctionType, LLVMInt32TypeInContext,
        LLVMModuleCreateWithName, LLVMPrintModuleToFile, LLVMVoidTypeInContext,
    },
    prelude::{LLVMBuilderRef, LLVMContextRef, LLVMModuleRef},
};

use crate::{components::codegen_types::Type, parser::AST};

use anyhow::Result;

pub struct Codegen {
    pub ctx: LLVMContextRef,
    pub module: LLVMModuleRef,
    pub builder: LLVMBuilderRef,
    pub functions: HashMap<String, Type>,
}

impl Drop for Codegen {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.builder);
            LLVMDisposeModule(self.module);
            LLVMContextDispose(self.ctx);
        }
    }
}

pub fn gen(module: &str, ast: AST) -> Result<()> {
    let mut codegen = unsafe {
        let ctx = LLVMContextCreate();

        let module = CString::new(module).unwrap();

        Codegen {
            ctx,
            module: LLVMModuleCreateWithName(module.as_ptr()),
            builder: LLVMCreateBuilderInContext(ctx),
            functions: HashMap::new(),
        }
    };

    // TODO how to extern whole lib
    // you don't, just extern everything necessary in lib source code and then call it on my exe?
    unsafe {
        LLVMAddFunction(
            codegen.module,
            b"putd\0".as_ptr() as _,
            LLVMFunctionType(
                LLVMVoidTypeInContext(codegen.ctx),
                [LLVMInt32TypeInContext(codegen.ctx)].as_mut_ptr(),
                1 as _,
                false as _,
            ),
        );
    }

    for item in ast.0 {
        codegen.gen_item(item)?;
    }

    unsafe {
        let path = CString::new(format!("{module}.ll")).unwrap();
        LLVMPrintModuleToFile(codegen.module, path.as_ptr(), std::ptr::null_mut());

        let path = CString::new(format!("{module}.bc")).unwrap();
        LLVMWriteBitcodeToFile(codegen.module, path.as_ptr());
    }

    Ok(())
}

// take some concepts of iconic:
// propagate errors so as to stop/yield execution
// binary operations succeed/fail and return rhs

/*

if -2 < -1 < 0 {

}

this executes, if condition will be sequently evaluated to a 'true' (if block)

//

let i = 10
i = i < find(pat, str)

this executes, i will equal to find output (assign)

*/
