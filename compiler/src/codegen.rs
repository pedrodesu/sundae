use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    process::Command,
};

use llvm_sys::{
    core::{
        LLVMAddFunction, LLVMContextCreate, LLVMContextDispose, LLVMCreateBuilderInContext,
        LLVMDisposeBuilder, LLVMDisposeMessage, LLVMDisposeModule, LLVMFunctionType,
        LLVMInt32TypeInContext, LLVMModuleCreateWithName, LLVMPrintModuleToFile,
        LLVMVoidTypeInContext,
    },
    prelude::{LLVMBuilderRef, LLVMContextRef, LLVMModuleRef},
    target::{LLVM_InitializeAllAsmPrinters, LLVM_InitializeNativeTarget},
    target_machine::{
        LLVMCodeGenFileType, LLVMCodeGenOptLevel, LLVMCodeModel, LLVMCreateTargetMachine,
        LLVMDisposeTargetMachine, LLVMGetDefaultTargetTriple, LLVMGetTargetFromTriple,
        LLVMRelocMode, LLVMTargetMachineEmitToFile,
    },
};

use crate::{components::codegen_types::Type, parser::AST};

use anyhow::{anyhow, bail, Result};

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
        let path = CString::new(format!("output/{module}.ll")).unwrap();
        let mut err = std::ptr::null_mut();
        if LLVMPrintModuleToFile(codegen.module, path.as_ptr(), &mut err) != 0 {
            let err_obj = anyhow!(
                "Couldn't output LLVM IR: {}",
                CStr::from_ptr(err).to_str().unwrap()
            );
            LLVMDisposeMessage(err);
            return Err(err_obj);
        };

        LLVM_InitializeAllAsmPrinters();

        if LLVM_InitializeNativeTarget() != 0 {
            bail!("Couldn't initialize LLVM native target");
        }

        let triple = LLVMGetDefaultTargetTriple();
        let mut target_ptr = std::ptr::null_mut();
        let mut err = std::ptr::null_mut();
        if LLVMGetTargetFromTriple(triple, &mut target_ptr, &mut err) != 0 {
            let err_obj = anyhow!(
                "Couldn't get target triple: {}",
                CStr::from_ptr(err).to_str().unwrap()
            );
            LLVMDisposeMessage(err);
            return Err(err_obj);
        }

        let target_machine = LLVMCreateTargetMachine(
            target_ptr,
            triple,
            "generic\0".as_ptr() as _,
            "\0".as_ptr() as _,
            LLVMCodeGenOptLevel::LLVMCodeGenLevelNone,
            LLVMRelocMode::LLVMRelocDefault,
            LLVMCodeModel::LLVMCodeModelDefault,
        );
        LLVMDisposeMessage(triple);

        let path = CString::new(format!("output/{module}.o")).unwrap();
        let mut err = std::ptr::null_mut();
        if LLVMTargetMachineEmitToFile(
            target_machine,
            codegen.module,
            path.as_ptr().cast_mut(),
            LLVMCodeGenFileType::LLVMObjectFile,
            &mut err,
        ) != 0
        {
            let err_obj = anyhow!(
                "Couldn't write to object file: {}",
                CStr::from_ptr(err).to_str().unwrap()
            );
            LLVMDisposeMessage(err);
            return Err(err_obj);
        }

        let exec = Command::new("clang-17")
            .args([
                "target/debug/libsundae_library.so",
                &format!("output/{module}.o"),
                "-o",
                &format!("output/{module}"),
            ])
            .output();

        match exec {
            Err(e) => bail!("Couldn't link object file: {e}"),
            Ok(o) => {
                if !o.stderr.is_empty() {
                    bail!(
                        "Couldn't link object file: {}",
                        std::str::from_utf8(&o.stderr).unwrap()
                    );
                }
            }
        }

        // TODO no deferring, use drop-like behaviour with structs correctly, this leaks
        LLVMDisposeTargetMachine(target_machine);
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
