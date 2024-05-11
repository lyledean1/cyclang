use std::collections::HashMap;
use std::ffi::CString;
use llvm_sys::core::{LLVMFunctionType, LLVMGetNamedFunction, LLVMGetTypeByName2, LLVMPointerType, LLVMVoidTypeInContext};
use llvm_sys::prelude::{LLVMBasicBlockRef, LLVMContextRef, LLVMModuleRef};
use cyclang_parser::Type;
use crate::compiler::codegen::context::{LLVMFunction, LLVMFunctionCache};
use crate::compiler::codegen::{int32_type, int8_ptr_type};

pub unsafe fn load_list_helper_funcs(
    context: LLVMContextRef,
    module: LLVMModuleRef,
    llvm_func_cache: &mut LLVMFunctionCache,
    block: LLVMBasicBlockRef,
) {
    let void_type: *mut llvm_sys::LLVMType = LLVMVoidTypeInContext(context);
    let list_struct_name = CString::new("struct.StringType").expect("CString::new failed");
    let list_type = LLVMGetTypeByName2(context, list_struct_name.as_ptr());
    let list_ptr_type = LLVMPointerType(list_type, 0);

    let list_push_int32_function_name = CString::new("pushInt32").expect("CString::new failed");
    let list_push_int32_function = LLVMGetNamedFunction(module, list_push_int32_function_name.as_ptr());

    // todo: load array correctly
    let mut list_push_int32_args = [list_ptr_type, int32_type()];
    let list_push_int32_func_type = LLVMFunctionType(
        void_type,
        list_push_int32_args.as_mut_ptr(),
        list_push_int32_args.len() as u32,
        0,
    );
    llvm_func_cache.set(
        "pushInt32",
        LLVMFunction {
            function: list_push_int32_function,
            func_type: list_push_int32_func_type,
            block,
            entry_block: block,
            symbol_table: HashMap::new(),
            args: vec![int8_ptr_type()],
            return_type: Type::None,
        },
    );
}