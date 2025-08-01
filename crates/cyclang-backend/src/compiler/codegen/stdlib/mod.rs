pub mod list;
pub mod string;

use crate::compiler::codegen::context::LLVMFunctionCache;
use anyhow::{anyhow, Result};
use llvm_sys::bit_reader::LLVMParseBitcodeInContext2;
use llvm_sys::core::LLVMCreateMemoryBufferWithContentsOfFile;
use llvm_sys::linker::LLVMLinkModules2;
use llvm_sys::prelude::{LLVMContextRef, LLVMMemoryBufferRef, LLVMModuleRef};
use std::ffi::CString;
use std::ptr;

use std::io::Write;
use tempfile::NamedTempFile;

const TYPES_BC: &[u8] = include_bytes!("./types.bc");

fn get_types_bc_path() -> Result<NamedTempFile, std::io::Error> {
    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(TYPES_BC)?;
    Ok(temp_file)
}

/// # Safety
///
/// Loads the bitcode file generated from string.c
pub unsafe fn load_bitcode_and_set_stdlib_funcs(
    context: LLVMContextRef,
    module: LLVMModuleRef,
    func_cache: LLVMFunctionCache,
) -> Result<LLVMFunctionCache> {
    let mut module_std: LLVMModuleRef = ptr::null_mut();
    let mut buffer: LLVMMemoryBufferRef = ptr::null_mut();
    let mut error: *mut i8 = ptr::null_mut();

    let temp_file = get_types_bc_path()?;
    let path = CString::new(temp_file.path().to_str().unwrap())?;
    // let path =
    //     CString::new("./crates/cyclang-backend/src/compiler/codegen/stdlib/types.bc").unwrap();
    let fail = LLVMCreateMemoryBufferWithContentsOfFile(path.as_ptr(), &mut buffer, &mut error);
    if fail != 0 {
        return Err(anyhow!("error loading bitcode and set stdlib funcs, check file exists for types.bc"));
    }

    // Parse the bitcode file
    let fail = LLVMParseBitcodeInContext2(context, buffer, &mut module_std);
    if fail != 0 {
        return Err(anyhow!("error loading bitcode"));
    }

    let result = LLVMLinkModules2(module, module_std);
    if result != 0 {
        return Err(anyhow!("error loading bitcode"));
    }
    Ok(func_cache)
}
