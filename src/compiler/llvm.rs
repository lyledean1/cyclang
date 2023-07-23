use crate::context::LLVMFunction;
use std::collections::HashMap;

extern crate llvm_sys;
use crate::parser::Type;
use llvm_sys::core::*;
use llvm_sys::prelude::*;

#[macro_export]
macro_rules! c_str {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const i8
    };
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

pub fn c_str(format_str: &str) -> *const i8 {
    format_str.as_ptr() as *const i8
}


pub fn var_type_str(name: String, type_name: String) -> String {
    name + "_" + &type_name
}

pub unsafe fn build_bool_to_str_func(
    module: LLVMModuleRef,
    context: LLVMContextRef,
) -> LLVMFunction {
    // Create the function
    let char_ptr_type = unsafe { LLVMPointerType(LLVMInt8TypeInContext(context), 0) };
    let func_type = unsafe { LLVMFunctionType(char_ptr_type, &mut int1_ptr_type(), 1, 0) };
    let function = unsafe { LLVMAddFunction(module, c_str!("bool_to_str"), func_type) };

    // Create the basic blocks
    let entry_block = unsafe { LLVMAppendBasicBlockInContext(context, function, c_str!("entry")) };
    let then_block = unsafe { LLVMAppendBasicBlockInContext(context, function, c_str!("then")) };
    let else_block = unsafe { LLVMAppendBasicBlockInContext(context, function, c_str!("else")) };

    // Build the entry block
    let builder = unsafe { LLVMCreateBuilderInContext(context) };
    LLVMPositionBuilderAtEnd(builder, entry_block);
    let condition = LLVMGetParam(function, 0);
    let value = LLVMBuildLoad2(builder, int1_type(), condition, c_str!("load_bool"));
    LLVMBuildCondBr(builder, value, then_block, else_block);

    // Build the 'then' block (return "true")
    let true_global = LLVMBuildGlobalStringPtr(builder, c_str!("true\n"), c_str!("true_str"));

    LLVMPositionBuilderAtEnd(builder, then_block);
    LLVMBuildRet(builder, true_global);

    // Build the 'else' block (return "false")
    let false_global = LLVMBuildGlobalStringPtr(builder, c_str!("false\n"), c_str!("false_str"));
    LLVMPositionBuilderAtEnd(builder, else_block);
    LLVMBuildRet(builder, false_global);

    LLVMFunction {
        function,
        func_type,
        entry_block,
        block: entry_block,
        symbol_table: HashMap::new(),
        args: vec![],
        return_type: Type::Bool, // ignore
    }
}
