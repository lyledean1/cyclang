extern crate llvm_sys;

use llvm_sys::core::*;
use llvm_sys::prelude::*;
use std::ffi::CString;

pub mod context;
pub mod control_flow;
pub mod functions;
pub mod types;

pub fn cstr_from_string(name: &str) -> CString {
    // can't return pointer type since it will get dropped due to lifetimes
    CString::new(name).unwrap()
}

pub fn int1_type() -> LLVMTypeRef {
    unsafe { LLVMInt1Type() }
}

pub fn int1_ptr_type() -> LLVMTypeRef {
    unsafe { LLVMPointerType(LLVMInt1Type(), 0) }
}

pub fn int8_type() -> LLVMTypeRef {
    unsafe { LLVMInt8Type() }
}

pub fn int32_type() -> LLVMTypeRef {
    unsafe { LLVMInt32Type() }
}

pub fn int32_ptr_type() -> LLVMTypeRef {
    unsafe { LLVMPointerType(LLVMInt32Type(), 0) }
}

pub fn int8_ptr_type() -> LLVMTypeRef {
    unsafe { LLVMPointerType(LLVMInt8Type(), 0) }
}

pub fn var_type_str(name: String, type_name: String) -> String {
    name + "_" + &type_name
}
