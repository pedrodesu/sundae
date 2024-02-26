use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    process::Command,
};

use llvm_sys::{
    core::{
        LLVMAddFunction, LLVMAppendBasicBlockInContext, LLVMBuildAlloca, LLVMBuildStore,
        LLVMContextCreate, LLVMContextDispose, LLVMCreateBuilderInContext, LLVMDisposeBuilder,
        LLVMDisposeMessage, LLVMDisposeModule, LLVMFunctionType, LLVMGetParam,
        LLVMInt32TypeInContext, LLVMModuleCreateWithName, LLVMPositionBuilderAtEnd,
        LLVMPrintModuleToFile, LLVMVoidTypeInContext,
    },
    prelude::{LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMValueRef},
    support::LLVMLoadLibraryPermanently,
    target::{LLVM_InitializeAllAsmPrinters, LLVM_InitializeNativeTarget},
    target_machine::{
        LLVMCodeGenFileType, LLVMCodeGenOptLevel, LLVMCodeModel, LLVMCreateTargetMachine,
        LLVMDisposeTargetMachine, LLVMGetDefaultTargetTriple, LLVMGetTargetFromTriple,
        LLVMRelocMode, LLVMTargetMachineEmitToFile,
    },
};

use crate::{components::codegen_types::Type, parser::AST};

use anyhow::{anyhow, bail, Result};

#[derive(Clone, Debug)]
pub struct Value {
    pub r#type: Type,
    pub inner: LLVMValueRef,
}

pub struct Local {
    mutable: bool,
    value: Value,
}

#[derive(Clone, Debug)]
pub struct Function {
    pub arguments: Vec<(String, Type)>,
    pub return_type: Type,
    pub stack: HashMap<String, Value>,
    pub inner: LLVMValueRef,
}

impl Function {
    #[inline]
    pub fn init_block(&mut self, codegen: &Codegen) {
        unsafe {
            let block =
                LLVMAppendBasicBlockInContext(codegen.ctx, self.inner, "entry\0".as_ptr() as _);

            LLVMPositionBuilderAtEnd(codegen.builder, block);
        }
    }

    #[inline]
    pub fn init_args_stack(&mut self, codegen: &Codegen) -> Result<()> {
        self.arguments
            .clone()
            .into_iter()
            .enumerate()
            .map(|(i, (name, r#type))| {
                let arg = unsafe { LLVMGetParam(self.inner, i as _) };

                let c_name = CString::new(name.as_str()).unwrap();

                unsafe {
                    let inner = LLVMBuildAlloca(
                        codegen.builder,
                        r#type.get_type(codegen.ctx),
                        c_name.as_ptr(),
                    );
                    LLVMBuildStore(codegen.builder, arg, inner);

                    self.stack.insert(name, Value { r#type, inner });
                }

                Ok(())
            })
            .collect()
    }
}

pub struct Runtime {
    pub functions: HashMap<String, Function>,
}

pub struct Codegen {
    pub ctx: LLVMContextRef,
    pub module: LLVMModuleRef,
    pub builder: LLVMBuilderRef,
    pub runtime: Runtime,
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
            runtime: Runtime {
                functions: Default::default(),
            },
        }
    };

    // TODO how to extern whole lib
    // you don't, just extern everything necessary in lib source code and then call it on my exe?
    let target_machine = unsafe {
        LLVM_InitializeAllAsmPrinters();

        if LLVM_InitializeNativeTarget() != 0 {
            bail!("Couldn't initialize LLVM native target");
        }

        let triple = LLVMGetDefaultTargetTriple();
        let mut target_ptr: *mut llvm_sys::target_machine::LLVMTarget = std::ptr::null_mut();
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

        // TODO figure out how to use this shit!!! how to call functions from external dll
        if LLVMLoadLibraryPermanently("target/debug/libsundae_library.so\0".as_ptr() as _) != 0 {
            bail!("Couldn't load standard library form LLVM");
        }

        codegen.runtime.functions.insert(
            "putd".to_string(),
            Function {
                arguments: vec![(
                    "n".to_string(),
                    Type::Integer {
                        width: 32,
                        signed: true,
                    },
                )],
                return_type: Type::Void,
                stack: Default::default(),
                inner: LLVMAddFunction(
                    codegen.module,
                    "putd\0".as_ptr() as _,
                    LLVMFunctionType(
                        LLVMVoidTypeInContext(codegen.ctx),
                        [LLVMInt32TypeInContext(codegen.ctx)].as_mut_ptr(),
                        1 as _,
                        false as _,
                    ),
                ),
            },
        );

        target_machine
    };

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

        // See https://github.com/rust-lang/rust/blob/master/compiler/rustc_codegen_ssa/src/back/link.rs#L1280
        // order of priority on *nix cc -> lld -> ld
        // on msvc is lld -> link.exe
        // fuck other platforms for now
        // TODO consider mold?
        let exec = Command::new("cc")
            .args([
                &format!("output/{module}.o"),
                "target/debug/libsundae_library.so",
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
