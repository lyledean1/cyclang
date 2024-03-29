use crate::compiler::codegen::context::{LLVMFunction, LLVMFunctionCache};
use crate::compiler::codegen::functions::build_helper_funcs;
use crate::compiler::codegen::{cstr_from_string, int64_type};
use crate::compiler::CompileOptions;
use crate::parser::Type;
use anyhow::Result;
use libc::c_uint;
use llvm_sys::core::{
    LLVMAddFunction, LLVMAppendBasicBlock, LLVMAppendBasicBlockInContext, LLVMArrayType2,
    LLVMBuildAlloca, LLVMBuildBr, LLVMBuildCall2, LLVMBuildCondBr, LLVMBuildGEP2,
    LLVMBuildGlobalStringPtr, LLVMBuildLoad2, LLVMBuildRet, LLVMBuildRetVoid, LLVMBuildSExt,
    LLVMBuildStore, LLVMConstArray2, LLVMConstInt, LLVMContextCreate, LLVMContextDispose,
    LLVMCreateBuilderInContext, LLVMDisposeBuilder, LLVMDisposeMessage, LLVMDisposeModule,
    LLVMFunctionType, LLVMGetIntTypeWidth, LLVMModuleCreateWithName, LLVMPositionBuilderAtEnd,
    LLVMPrintModuleToFile, LLVMSetTarget, LLVMTypeOf,
    LLVMVoidTypeInContext,
};
use llvm_sys::execution_engine::{
    LLVMCreateExecutionEngineForModule, LLVMDisposeExecutionEngine, LLVMGetFunctionAddress,
    LLVMLinkInMCJIT,
};
use llvm_sys::prelude::{
    LLVMBasicBlockRef, LLVMBool, LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMTypeRef,
    LLVMValueRef,
};
use llvm_sys::target::{LLVM_InitializeNativeAsmPrinter, LLVM_InitializeNativeTarget};
use std::collections::HashMap;
use std::process::Command;
use std::ptr;

pub struct LLVMCodegenBuilder {
    pub builder: LLVMBuilderRef,
    pub module: LLVMModuleRef,
    pub context: LLVMContextRef,
    pub llvm_func_cache: LLVMFunctionCache,
    pub current_function: LLVMFunction,
    pub printf_str_value: LLVMValueRef,
    pub printf_str_num_value: LLVMValueRef,
    pub printf_str_num64_value: LLVMValueRef,
    is_execution_engine: bool,
    is_default_target: bool,
}

impl LLVMCodegenBuilder {
    // Initialise execution engine and LLVM IR constructs
    pub fn init(compile_options: Option<CompileOptions>) -> Result<LLVMCodegenBuilder> {
        unsafe {
            let mut is_execution_engine = false;
            let mut is_default_target: bool = true;

            if let Some(compile_options) = compile_options {
                is_execution_engine = compile_options.is_execution_engine;
                is_default_target = compile_options.target.is_none();
            }

            if is_execution_engine {
                LLVMLinkInMCJIT();
            }

            if is_default_target {
                LLVM_InitializeNativeTarget();
                LLVM_InitializeNativeAsmPrinter();
            }
            if !is_default_target {
                compile_options.unwrap().target.unwrap().initialize();
            }

            let context = LLVMContextCreate();
            let module = LLVMModuleCreateWithName(cstr_from_string("main").as_ptr());
            let builder = LLVMCreateBuilderInContext(context);
            if !is_default_target {
                LLVMSetTarget(
                    module,
                    cstr_from_string("wasm32-unknown-unknown-wasm").as_ptr(),
                );
            }
            // common void type
            let void_type: *mut llvm_sys::LLVMType = LLVMVoidTypeInContext(context);

            // our "main" function which will be the entry point when we run the executable
            let main_func_type = LLVMFunctionType(void_type, ptr::null_mut(), 0, 0);
            let main_func =
                LLVMAddFunction(module, cstr_from_string("main").as_ptr(), main_func_type);
            let main_block = LLVMAppendBasicBlockInContext(
                context,
                main_func,
                cstr_from_string("main").as_ptr(),
            );
            LLVMPositionBuilderAtEnd(builder, main_block);

            // Define common functions

            let llvm_func_cache = build_helper_funcs(module, context, main_block);

            let format_str = "%d\n";
            let printf_str_num_value = LLVMBuildGlobalStringPtr(
                builder,
                cstr_from_string(format_str).as_ptr(),
                cstr_from_string("number_printf_val").as_ptr(),
            );
            let printf_str_num64_value = LLVMBuildGlobalStringPtr(
                builder,
                cstr_from_string("%llu\n").as_ptr(),
                cstr_from_string("number64_printf_val").as_ptr(),
            );
            let printf_str_value = LLVMBuildGlobalStringPtr(
                builder,
                cstr_from_string("%s\n").as_ptr(),
                cstr_from_string("str_printf_val").as_ptr(),
            );

            Ok(LLVMCodegenBuilder {
                builder,
                module,
                context,
                llvm_func_cache,
                current_function: LLVMFunction {
                    function: main_func,
                    func_type: main_func_type,
                    block: main_block,
                    entry_block: main_block,
                    symbol_table: HashMap::new(),
                    args: vec![],
                    return_type: Type::None,
                },
                printf_str_value,
                printf_str_num_value,
                printf_str_num64_value,
                is_execution_engine,
                is_default_target,
            })
        }
    }

    pub fn dispose_and_get_module_str(&self) -> Result<String> {
        unsafe {
            LLVMBuildRetVoid(self.builder);

            // Run execution engine
            let mut engine = ptr::null_mut();
            let mut error = ptr::null_mut();

            // Call the main function. It should execute its code.
            if self.is_execution_engine {
                if LLVMCreateExecutionEngineForModule(&mut engine, self.module, &mut error) != 0 {
                    LLVMDisposeMessage(error);
                    panic!("Failed to create execution engine");
                }
                let main_func: extern "C" fn() = std::mem::transmute(LLVMGetFunctionAddress(
                    engine,
                    b"main\0".as_ptr() as *const _,
                ));
                main_func();
            }

            if !self.is_execution_engine {
                LLVMPrintModuleToFile(
                    self.module,
                    cstr_from_string("bin/main.ll").as_ptr(),
                    ptr::null_mut(),
                );
            }
            // clean up
            LLVMDisposeBuilder(self.builder);
            if self.is_execution_engine {
                LLVMDisposeExecutionEngine(engine);
            }
            if !self.is_execution_engine {
                LLVMDisposeModule(self.module);
            }
            LLVMContextDispose(self.context);
            self.emit_binary()
        }
    }

    pub fn emit_binary(&self) -> Result<String> {
        if !self.is_execution_engine {
            Command::new("clang")
                .arg("bin/main.ll")
                .arg("-o")
                .arg("bin/main")
                .output()?;
            let output = Command::new("bin/main").output()?;
            return Ok(String::from_utf8_lossy(&output.stdout).to_string());
        }
        Ok("".to_string())
    }

    /// build_load
    ///
    /// This reads a value from one memory location via the LLVMBuildLoad instruction
    ///
    /// # Arguments
    ///
    /// * `ptr` - The LLVM Value you are loading from memory
    /// * `ptr_type` - The LLVM Type you will be storing in memory
    /// * `name` - The LLVM name of the alloca
    ///
    pub fn build_load(&self, ptr: LLVMValueRef, ptr_type: LLVMTypeRef, name: &str) -> LLVMValueRef {
        unsafe { LLVMBuildLoad2(self.builder, ptr_type, ptr, cstr_from_string(name).as_ptr()) }
    }

    /// build_store
    ///
    /// This stores a value into memory on the stack via the LLVMBuildStore instruction
    ///
    /// # Arguments
    ///
    /// * `val` - The LLVM Value you are storing into memory
    /// * `ptr` - The LLVM pointer you will be storing the value in memory
    ///
    pub fn build_store(&self, val: LLVMValueRef, ptr: LLVMValueRef) {
        unsafe {
            LLVMBuildStore(self.builder, val, ptr);
        }
    }

    /// build_alloca
    ///
    /// This builds memory on the stack via the LLVMBuildAlloca instruction
    ///
    /// # Arguments
    ///
    /// * `ptr_type` - The LLVM Type you will be storing in memory
    /// * `name` - The LLVM name of the alloca
    ///
    pub fn build_alloca(&self, ptr_type: LLVMTypeRef, name: &str) -> LLVMValueRef {
        unsafe { LLVMBuildAlloca(self.builder, ptr_type, cstr_from_string(name).as_ptr()) }
    }

    /// build_alloca_store
    ///
    /// This calls LLVM to allocate memory on the stack via the LLVMBuildAlloca function and then
    /// stores the provided value into that new allocated stack memory. It then returns a pointer to that value.
    ///
    /// # Arguments
    ///
    /// * `val` - The LLVM Value you will be storing in memory
    /// * `ptr_type` - The LLVM Type you will be storing in memory
    /// * `name` - The LLVM name of the alloca
    ///
    pub fn build_alloca_store(
        &self,
        val: LLVMValueRef,
        ptr_type: LLVMTypeRef,
        name: &str,
    ) -> LLVMValueRef {
        let ptr = self.build_alloca(ptr_type, name);
        self.build_store(val, ptr);
        ptr
    }

    /// build_load_store
    ///
    /// This reads a value from one memory location via the LLVMBuildLoad instruction
    /// and writes it to another via the LLVMBuildStore location.
    ///
    /// # Arguments
    ///
    /// * `load_ptr` - The LLVM Value you are loading from memory
    /// * `store_ptr` - The LLVM Type you will be storing in memory
    /// * `ptr_type` - The LLVM Type you will be storing in memory
    /// * `name` - The LLVM name of the alloca
    ///
    pub fn build_load_store(
        &self,
        load_ptr: LLVMValueRef,
        store_ptr: LLVMValueRef,
        ptr_type: LLVMTypeRef,
        name: &str,
    ) {
        let rhs_val = self.build_load(load_ptr, ptr_type, name);
        self.build_store(rhs_val, store_ptr);
    }

    pub fn append_basic_block(&self, function: LLVMValueRef, name: &str) -> LLVMBasicBlockRef {
        unsafe { LLVMAppendBasicBlock(function, cstr_from_string(name).as_ptr()) }
    }

    pub fn build_call(
        &self,
        func: LLVMFunction,
        args: Vec<LLVMValueRef>,
        num_args: c_uint,
        name: &str,
    ) -> LLVMValueRef {
        unsafe {
            LLVMBuildCall2(
                self.builder,
                func.func_type,
                func.function,
                args.clone().as_mut_ptr(),
                num_args,
                cstr_from_string(name).as_ptr(),
            )
        }
    }

    pub fn cast_i32_to_i64(
        &self,
        mut lhs_value: LLVMValueRef,
        rhs_value: LLVMValueRef,
    ) -> LLVMValueRef {
        unsafe {
            let lhs_value_type = LLVMTypeOf(lhs_value);
            let lhs_value_width = LLVMGetIntTypeWidth(lhs_value_type);
            let rhs_value_type = LLVMTypeOf(rhs_value);
            let rhs_value_width = LLVMGetIntTypeWidth(rhs_value_type);

            if let (32, 64) = (lhs_value_width, rhs_value_width) {
                lhs_value = LLVMBuildSExt(
                    self.builder,
                    lhs_value,
                    int64_type(),
                    cstr_from_string("cast_to_i64").as_ptr(),
                );
            }
            lhs_value
        }
    }

    pub fn build_br(&self, block: LLVMBasicBlockRef) -> LLVMValueRef {
        unsafe { LLVMBuildBr(self.builder, block) }
    }

    pub fn build_cond_br(
        &self,
        cond: LLVMValueRef,
        then_block: LLVMBasicBlockRef,
        else_block: LLVMBasicBlockRef,
    ) -> LLVMValueRef {
        unsafe { LLVMBuildCondBr(self.builder, cond, then_block, else_block) }
    }

    pub fn position_builder_at_end(&self, block: LLVMBasicBlockRef) {
        unsafe {
            LLVMPositionBuilderAtEnd(self.builder, block);
        }
    }

    pub fn build_ret_void(&self) {
        unsafe {
            LLVMBuildRetVoid(self.builder);
        }
    }

    pub fn build_ret(&self, value: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildRet(self.builder, value) }
    }

    pub fn const_int(
        &self,
        int_type: LLVMTypeRef,
        val: ::libc::c_ulonglong,
        sign_extend: LLVMBool,
    ) -> LLVMValueRef {
        unsafe { LLVMConstInt(int_type, val, sign_extend) }
    }

    pub fn const_array(
        &self,
        element_type: LLVMTypeRef,
        const_values: *mut LLVMValueRef,
        length: u64,
    ) -> LLVMValueRef {
        unsafe { LLVMConstArray2(element_type, const_values, length) }
    }

    pub fn array_type(&self, element_type: LLVMTypeRef, element_count: u64) -> LLVMTypeRef {
        unsafe { LLVMArrayType2(element_type, element_count) }
    }

    pub fn build_gep(
        &self,
        llvm_type: LLVMTypeRef,
        ptr: LLVMValueRef,
        indices: *mut LLVMValueRef,
        num_indices: ::libc::c_uint,
        name: *const ::libc::c_char,
    ) -> LLVMValueRef {
        unsafe { LLVMBuildGEP2(self.builder, llvm_type, ptr, indices, num_indices, name) }
    }
}
