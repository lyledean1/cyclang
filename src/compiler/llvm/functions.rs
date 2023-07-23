// These are inbuilt LLVM functions used as helpers inside the code
use crate::compiler::llvm::context::LLVMFunction;
use crate::compiler::llvm::context::LLVMFunctionCache;
use crate::compiler::llvm::*;
use crate::parser::Type;
use std::collections::HashMap;

//TODO: delete duplicate
macro_rules! c_str {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const i8
    };
}

pub unsafe fn build_helper_funcs(
    module: LLVMModuleRef,
    context: LLVMContextRef,
    main_block: LLVMBasicBlockRef,
) -> LLVMFunctionCache {
    let mut llvm_func_cache = LLVMFunctionCache::new();
    let bool_to_str_func = build_bool_to_str_func(module, context);
    llvm_func_cache.set("bool_to_str", bool_to_str_func);
    let void_type: *mut llvm_sys::LLVMType = LLVMVoidTypeInContext(context);

    //printf
    let print_func_type = LLVMFunctionType(void_type, [int8_ptr_type()].as_mut_ptr(), 1, 1);
    let print_func = LLVMAddFunction(module, c_str!("printf"), print_func_type);
    llvm_func_cache.set(
        "printf",
        LLVMFunction {
            function: print_func,
            func_type: print_func_type,
            block: main_block,
            entry_block: main_block,
            symbol_table: HashMap::new(),
            args: vec![],
            return_type: Type::None,
        },
    );
    //sprintf
    let mut arg_types = [
        LLVMPointerType(LLVMInt8TypeInContext(context), 0),
        LLVMPointerType(LLVMInt8TypeInContext(context), 0),
        LLVMPointerType(LLVMInt8TypeInContext(context), 0),
        LLVMPointerType(LLVMInt8TypeInContext(context), 0),
    ];
    let ret_type = LLVMPointerType(LLVMInt8TypeInContext(context), 0);
    let sprintf_type =
        LLVMFunctionType(ret_type, arg_types.as_mut_ptr(), arg_types.len() as u32, 1);
    let sprintf = LLVMAddFunction(module, "sprintf\0".as_ptr() as *const i8, sprintf_type);
    llvm_func_cache.set(
        "sprintf",
        LLVMFunction {
            function: sprintf,
            func_type: sprintf_type,
            block: main_block,
            entry_block: main_block,
            symbol_table: HashMap::new(),
            args: vec![],
            return_type: Type::None,
        },
    );
    llvm_func_cache
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
