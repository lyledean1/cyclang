use crate::compiler::codegen::context::{LLVMCallFn, LLVMFunctionCache};
use crate::compiler::codegen::{int32_ptr_type, int32_type};
use llvm_sys::core::{
    LLVMFunctionType, LLVMGetNamedFunction, LLVMGetTypeByName2, LLVMPointerType,
    LLVMVoidTypeInContext,
};
use llvm_sys::prelude::{LLVMContextRef, LLVMModuleRef, LLVMTypeRef};
use std::ffi::CString;

/// # Safety
///
/// Load List Helper funcs
pub unsafe fn load_list_helper_funcs(
    context: LLVMContextRef,
    module: LLVMModuleRef,
    llvm_func_cache: &mut LLVMFunctionCache,
) {
    let void_type = LLVMVoidTypeInContext(context);

    // createInt32List
    let mut list_create_int32_args = vec![int32_type()];
    create_and_set_llvm_function(
        module,
        llvm_func_cache,
        "create_int32_tList",
        &mut list_create_int32_args,
        int32_ptr_type(),
    );
    // setInt32Value
    let mut list_set_int32_args = vec![int32_ptr_type(), int32_type(), int32_type()];
    create_and_set_llvm_function(
        module,
        llvm_func_cache,
        "set_int32_tValue",
        &mut list_set_int32_args,
        void_type,
    );
    // getInt32Value
    let mut list_get_int32_args = vec![int32_ptr_type(), int32_type()];
    create_and_set_llvm_function(
        module,
        llvm_func_cache,
        "get_int32_tValue",
        &mut list_get_int32_args,
        int32_ptr_type(),
    );
    // printInt32List
    let mut print_list_int32_args = vec![int32_ptr_type()];
    create_and_set_llvm_function(
        module,
        llvm_func_cache,
        "printInt32List",
        &mut print_list_int32_args,
        void_type,
    );

    let mut len_list_int32_args = vec![int32_ptr_type()];
    create_and_set_llvm_function(
        module,
        llvm_func_cache,
        "lenInt32List",
        &mut len_list_int32_args,
        int32_type(),
    );

    let mut concat_int_32_args = vec![int32_ptr_type(), int32_ptr_type()];
    create_and_set_llvm_function(
        module,
        llvm_func_cache,
        "concatInt32List",
        &mut concat_int_32_args,
        int32_ptr_type(),
    );

    // * String * //
    let string_struct_name = CString::new("struct.StringType").expect("CString::new failed");
    let string_type = LLVMGetTypeByName2(context, string_struct_name.as_ptr());
    let string_ptr_type = LLVMPointerType(string_type, 0);
    let string_ptr_ptr_type = LLVMPointerType(string_ptr_type, 0);

    let mut list_create_string_list_args = vec![int32_type()];
    create_and_set_llvm_function(
        module,
        llvm_func_cache,
        "createStringList",
        &mut list_create_string_list_args,
        string_ptr_ptr_type,
    );
    // setInt32Value
    let mut list_set_string_args = vec![string_ptr_ptr_type, string_ptr_type, int32_type()];
    create_and_set_llvm_function(
        module,
        llvm_func_cache,
        "setStringValue",
        &mut list_set_string_args,
        void_type,
    );
    // getInt32Value
    let mut list_get_string_args = vec![string_ptr_ptr_type, int32_type()];
    create_and_set_llvm_function(
        module,
        llvm_func_cache,
        "getStringValue",
        &mut list_get_string_args,
        string_ptr_type,
    );
    // printStringList
    let mut print_list_string_args = vec![string_ptr_ptr_type];
    create_and_set_llvm_function(
        module,
        llvm_func_cache,
        "printStringList",
        &mut print_list_string_args,
        void_type,
    );

    let mut len_list_string_args = vec![string_ptr_ptr_type];
    create_and_set_llvm_function(
        module,
        llvm_func_cache,
        "lenStringList",
        &mut len_list_string_args,
        int32_type(),
    );
    // concatStringList
    let mut concat_string_list_args = vec![string_ptr_ptr_type, string_ptr_ptr_type];
    create_and_set_llvm_function(
        module,
        llvm_func_cache,
        "concatStringList",
        &mut concat_string_list_args,
        string_ptr_ptr_type,
    );
}

unsafe fn create_and_set_llvm_function(
    module: LLVMModuleRef,
    llvm_func_cache: &mut LLVMFunctionCache,
    func_name: &str,
    func_args: &mut Vec<LLVMTypeRef>,
    return_type: LLVMTypeRef,
) {
    let llvm_function_name = CString::new(func_name).expect("CString::new failed");
    let llvm_function = LLVMGetNamedFunction(module, llvm_function_name.as_ptr());
    let llvm_function_type = LLVMFunctionType(
        return_type,
        func_args.as_mut_ptr(),
        func_args.len() as u32,
        0,
    );
    llvm_func_cache.set(
        func_name,
        LLVMCallFn {
            function: llvm_function,
            func_type: llvm_function_type,
        },
    );
}
