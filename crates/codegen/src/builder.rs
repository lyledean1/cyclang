use crate::context::{LLVMCallFn, LLVMFunction, LLVMFunctionCache};
use crate::stdlib::list::load_list_helper_funcs;
use crate::stdlib::load_bitcode_and_set_stdlib_funcs;
use crate::stdlib::string::load_string_helper_funcs;
use crate::{
    cstr_from_string, int1_type, int32_ptr_type, int32_type, int64_type, int8_ptr_type,
};
use crate::code_generator::GeneratedValue;
use crate::typed_ast::ResolvedType;
use crate::types::bool::BoolType;
use crate::types::{BaseTypes, TypeBase};
use crate::CompileOptions;
use anyhow::{anyhow, Result};
use libc::c_uint;
use llvm_sys::analysis::{LLVMVerifierFailureAction, LLVMVerifyModule};
use llvm_sys::core::{
    LLVMAddFunction, LLVMAppendBasicBlock, LLVMAppendBasicBlockInContext, LLVMArrayType2,
    LLVMBuildAdd, LLVMBuildAlloca, LLVMBuildBr, LLVMBuildCall2, LLVMBuildCondBr, LLVMBuildGEP2,
    LLVMBuildGlobalString, LLVMBuildICmp, LLVMBuildLoad2, LLVMBuildMul, LLVMBuildRet,
    LLVMBuildRetVoid, LLVMBuildSDiv, LLVMBuildSExt, LLVMBuildStore, LLVMBuildSub, LLVMConstArray2,
    LLVMConstInt, LLVMCreateBuilderInContext,
    LLVMDeleteFunction, LLVMDisposeBuilder, LLVMDisposeModule, LLVMGetBasicBlockTerminator,
    LLVMGetGlobalContext,
    LLVMFunctionType, LLVMGetIntTypeWidth, LLVMGetNamedFunction, LLVMGetParam, LLVMGetTypeByName2,
    LLVMInt8TypeInContext, LLVMModuleCreateWithName, LLVMPointerType, LLVMPositionBuilderAtEnd,
    LLVMPrintModuleToFile, LLVMPrintModuleToString, LLVMPrintValueToString,
    LLVMSetDataLayout, LLVMSetTarget, LLVMTypeOf, LLVMVoidTypeInContext, LLVMDisposeMessage,
};
use llvm_sys::error::{LLVMDisposeErrorMessage, LLVMGetErrorMessage, LLVMErrorRef};
use llvm_sys::orc2::lljit::{
    LLVMOrcCreateLLJIT, LLVMOrcCreateLLJITBuilder, LLVMOrcDisposeLLJIT,
    LLVMOrcLLJITAddLLVMIRModule, LLVMOrcLLJITBuilderSetJITTargetMachineBuilder,
    LLVMOrcLLJITGetDataLayoutStr, LLVMOrcLLJITGetGlobalPrefix,
    LLVMOrcLLJITGetMainJITDylib, LLVMOrcLLJITGetTripleString, LLVMOrcLLJITLookup,
};
use llvm_sys::orc2::{
    LLVMOrcCreateDynamicLibrarySearchGeneratorForProcess, LLVMOrcCreateNewThreadSafeContext,
    LLVMOrcCreateNewThreadSafeModule, LLVMOrcDefinitionGeneratorRef, LLVMOrcJITDylibAddGenerator,
    LLVMOrcJITTargetMachineBuilderDetectHost, LLVMOrcJITTargetMachineBuilderRef,
    LLVMOrcDisposeThreadSafeContext, LLVMOrcThreadSafeModuleRef,
};
use llvm_sys::prelude::{
    LLVMBasicBlockRef, LLVMBool, LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMTypeRef,
    LLVMValueRef,
};
use llvm_sys::target::{LLVM_InitializeNativeAsmPrinter, LLVM_InitializeNativeTarget};
use llvm_sys::LLVMIntPredicate;
use llvm_sys::LLVMIntPredicate::{
    LLVMIntEQ, LLVMIntNE, LLVMIntSGE, LLVMIntSGT, LLVMIntSLE, LLVMIntSLT,
};
use std::ffi::CString;
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
    emit_llvm_ir: bool,
    emit_llvm_ir_main_only: bool,
    emit_llvm_ir_with_called: bool,
}

macro_rules! llvm_build_fn {
    ($fn_name:ident, $builder:expr, $lhs:expr, $rhs:expr, $name:expr) => {{
        $fn_name($builder, $lhs, $rhs, $name)
    }};
}

impl LLVMCodegenBuilder {
    // Initialise execution engine and LLVM IR constructs
    pub fn init(compile_options: Option<CompileOptions>) -> Result<LLVMCodegenBuilder> {
        unsafe {
            let mut is_execution_engine = false;
            let mut is_default_target: bool = true;

            let mut emit_llvm_ir = false;
            let mut emit_llvm_ir_main_only = true;
            let mut emit_llvm_ir_with_called = false;
            if let Some(compile_options) = compile_options {
                is_execution_engine = compile_options.is_execution_engine;
                is_default_target = compile_options.target.is_none();
                emit_llvm_ir = compile_options.emit_llvm_ir;
                emit_llvm_ir_main_only = compile_options.emit_llvm_ir_main_only;
                emit_llvm_ir_with_called = compile_options.emit_llvm_ir_with_called;
            }

            if is_default_target {
                LLVM_InitializeNativeTarget();
                LLVM_InitializeNativeAsmPrinter();
            }
            if !is_default_target {
                compile_options.unwrap().target.unwrap().initialize();
            }

            let context = LLVMGetGlobalContext();
            let module = LLVMModuleCreateWithName(cstr_from_string("main").as_ptr());
            let builder = LLVMCreateBuilderInContext(context);
            if !is_default_target {
                LLVMSetTarget(
                    module,
                    cstr_from_string("wasm32-unknown-unknown-wasm").as_ptr(),
                );
            }

            let dummy_func = Self::build_dummy_function(context, builder, module);

            // Define common functions

            let printf_str_num_value = LLVMBuildGlobalString(
                builder,
                cstr_from_string("%d\n").as_ptr(),
                cstr_from_string("number_printf_val").as_ptr(),
            );
            let printf_str_num64_value = LLVMBuildGlobalString(
                builder,
                cstr_from_string("%llu\n").as_ptr(),
                cstr_from_string("number64_printf_val").as_ptr(),
            );
            let printf_str_value = LLVMBuildGlobalString(
                builder,
                cstr_from_string("%s\n").as_ptr(),
                cstr_from_string("str_printf_val").as_ptr(),
            );

            let llvm_func_cache = LLVMFunctionCache::new();

            let llvm_func_cache =
                load_bitcode_and_set_stdlib_funcs(context, module, llvm_func_cache)?;

            let mut codegen_builder = LLVMCodegenBuilder {
                builder,
                module,
                context,
                llvm_func_cache,
                current_function: dummy_func.clone(),
                printf_str_value,
                printf_str_num_value,
                printf_str_num64_value,
                is_execution_engine,
                emit_llvm_ir,
                emit_llvm_ir_main_only,
                emit_llvm_ir_with_called,
            };
            LLVMDeleteFunction(dummy_func.function);
            codegen_builder.build_helper_funcs();
            Ok(codegen_builder)
        }
    }

    pub fn dispose_and_get_module_str(&self) -> Result<String> {
        unsafe {
            if self.is_execution_engine {
                // Verify module before JIT to surface invalid IR instead of crashing.
                let mut error: *mut i8 = ptr::null_mut();
                let has_error = LLVMVerifyModule(
                    self.module,
                    LLVMVerifierFailureAction::LLVMReturnStatusAction,
                    &mut error,
                );
                if has_error != 0 {
                    let msg = if error.is_null() {
                        "Unknown verification error".to_string()
                    } else {
                        let cstr = std::ffi::CStr::from_ptr(error);
                        let msg = cstr.to_string_lossy().to_string();
                        LLVMDisposeMessage(error);
                        msg
                    };
                    let ir = LLVMPrintModuleToString(self.module);
                    let ir_str = if ir.is_null() {
                        "<unable to print module>".to_string()
                    } else {
                        let cstr = std::ffi::CStr::from_ptr(ir);
                        let s = cstr.to_string_lossy().to_string();
                        LLVMDisposeMessage(ir);
                        s
                    };
                    return Err(anyhow!("LLVM module verification failed:\n{msg}\n{ir_str}"));
                }
                self.run_orc_jit_main()?;
            }

            if self.emit_llvm_ir {
                let module_ir = {
                    let ir = LLVMPrintModuleToString(self.module);
                    if ir.is_null() {
                        "<unable to print module>".to_string()
                    } else {
                        let cstr = std::ffi::CStr::from_ptr(ir);
                        let s = cstr.to_string_lossy().to_string();
                        LLVMDisposeMessage(ir);
                        s
                    }
                };

                let ir_str = if self.emit_llvm_ir_main_only {
                    if self.emit_llvm_ir_with_called {
                        extract_main_with_called_from_ir(&module_ir).unwrap_or(module_ir)
                    } else {
                        extract_main_only_from_ir(&module_ir).unwrap_or(module_ir)
                    }
                } else {
                    module_ir
                };
                LLVMDisposeBuilder(self.builder);
                if !self.is_execution_engine {
                    LLVMDisposeModule(self.module);
                }
                // Global context is managed by LLVM; don't dispose it.
                return Ok(ir_str);
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
            if !self.is_execution_engine {
                LLVMDisposeModule(self.module);
            }
            // Global context is managed by LLVM; don't dispose it.
            self.emit_binary()
        }
    }

    fn orc_error_to_anyhow(err: LLVMErrorRef, context: &str) -> anyhow::Error {
        unsafe {
            if err.is_null() {
                return anyhow::anyhow!("unknown ORC error");
            }
            let msg_ptr = LLVMGetErrorMessage(err);
            let msg = if msg_ptr.is_null() {
                "unknown ORC error".to_string()
            } else {
                let cstr = std::ffi::CStr::from_ptr(msg_ptr);
                let msg = cstr.to_string_lossy().to_string();
                LLVMDisposeErrorMessage(msg_ptr);
                msg
            };
            anyhow::anyhow!("{}: {}", context, msg)
        }
    }

    fn run_orc_jit_main(&self) -> Result<()> {
        unsafe {
            let tsc = LLVMOrcCreateNewThreadSafeContext();
            // Create LLJIT with host target
            let mut jit = ptr::null_mut();
            let builder = LLVMOrcCreateLLJITBuilder();
            let mut jtmb: LLVMOrcJITTargetMachineBuilderRef = ptr::null_mut();
            let err = LLVMOrcJITTargetMachineBuilderDetectHost(&mut jtmb);
            if !err.is_null() {
                return Err(Self::orc_error_to_anyhow(
                    err,
                    "ORC: failed to detect host target",
                ));
            }
            LLVMOrcLLJITBuilderSetJITTargetMachineBuilder(builder, jtmb);

            let err = LLVMOrcCreateLLJIT(&mut jit, builder);
            if !err.is_null() {
                return Err(Self::orc_error_to_anyhow(err, "ORC: failed to create LLJIT"));
            }

            // Set module target + data layout to match LLJIT
            let triple = LLVMOrcLLJITGetTripleString(jit);
            LLVMSetTarget(self.module, triple);
            let data_layout = LLVMOrcLLJITGetDataLayoutStr(jit);
            LLVMSetDataLayout(self.module, data_layout);

            // Allow resolving symbols like printf from the current process
            let mut gen: LLVMOrcDefinitionGeneratorRef = ptr::null_mut();
            let global_prefix = LLVMOrcLLJITGetGlobalPrefix(jit);
            let err = LLVMOrcCreateDynamicLibrarySearchGeneratorForProcess(
                &mut gen,
                global_prefix,
                None,
                ptr::null_mut(),
            );
            if !err.is_null() {
                let dispose_err = LLVMOrcDisposeLLJIT(jit);
                if !dispose_err.is_null() {
                    return Err(Self::orc_error_to_anyhow(
                        dispose_err,
                        "ORC: failed to dispose LLJIT after error",
                    ));
                }
                return Err(Self::orc_error_to_anyhow(
                    err,
                    "ORC: failed to create process symbol generator",
                ));
            }

            let jd = LLVMOrcLLJITGetMainJITDylib(jit);
            LLVMOrcJITDylibAddGenerator(jd, gen);

            // Create ThreadSafeModule and add to LLJIT
            let tsm: LLVMOrcThreadSafeModuleRef =
                LLVMOrcCreateNewThreadSafeModule(self.module, tsc);
            let err = LLVMOrcLLJITAddLLVMIRModule(jit, jd, tsm);
            if !err.is_null() {
                LLVMOrcDisposeThreadSafeContext(tsc);
                let dispose_err = LLVMOrcDisposeLLJIT(jit);
                if !dispose_err.is_null() {
                    return Err(Self::orc_error_to_anyhow(
                        dispose_err,
                        "ORC: failed to dispose LLJIT after error",
                    ));
                }
                return Err(Self::orc_error_to_anyhow(err, "ORC: failed to add module"));
            }

            // Lookup and call main
            let mut addr = 0u64;
            let err = LLVMOrcLLJITLookup(jit, &mut addr, c"main".as_ptr());
            if !err.is_null() {
                let dispose_err = LLVMOrcDisposeLLJIT(jit);
                if !dispose_err.is_null() {
                    return Err(Self::orc_error_to_anyhow(
                        dispose_err,
                        "ORC: failed to dispose LLJIT after error",
                    ));
                }
                LLVMOrcDisposeThreadSafeContext(tsc);
                return Err(Self::orc_error_to_anyhow(
                    err,
                    "ORC: failed to lookup symbol 'main'",
                ));
            }
            let main_func: extern "C" fn() = std::mem::transmute(addr);
            main_func();

            let err = LLVMOrcDisposeLLJIT(jit);
            LLVMOrcDisposeThreadSafeContext(tsc);
            if !err.is_null() {
                return Err(Self::orc_error_to_anyhow(err, "ORC: failed to dispose LLJIT"));
            }

            Ok(())
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

    // create dummy function to add global variables that is deleted after init
    pub fn build_dummy_function(
        context: LLVMContextRef,
        builder: LLVMBuilderRef,
        module: LLVMModuleRef,
    ) -> LLVMFunction {
        unsafe {
            let void_type: *mut llvm_sys::LLVMType = LLVMVoidTypeInContext(context);
            let dummy_func_type = LLVMFunctionType(void_type, ptr::null_mut(), 0, 0);
            let dummy_func =
                LLVMAddFunction(module, cstr_from_string("dummy").as_ptr(), dummy_func_type);
            let dummy_func_block = LLVMAppendBasicBlockInContext(
                context,
                dummy_func,
                cstr_from_string("entry").as_ptr(),
            );
            LLVMPositionBuilderAtEnd(builder, dummy_func_block);
            LLVMFunction {
                function: dummy_func,
                func_type: dummy_func_type,
                block: dummy_func_block,
                entry_block: dummy_func_block,
            }
        }
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
        func: LLVMCallFn,
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

    pub fn set_current_block(&mut self, block: LLVMBasicBlockRef) {
        self.position_builder_at_end(block);
        self.current_function.block = block;
    }

    pub fn set_entry_block(&mut self, block: LLVMBasicBlockRef) {
        self.current_function.entry_block = block;
    }

    pub fn get_printf_str(&mut self, val: BaseTypes) -> LLVMValueRef {
        match val {
            BaseTypes::Number => self.printf_str_num_value,
            BaseTypes::Number64 => self.printf_str_num64_value,
            BaseTypes::Bool => self.printf_str_value,
            BaseTypes::String => self.printf_str_value,
            BaseTypes::List(_) => self.printf_str_value, // placeholder - no-op
            _ => {
                unreachable!("get_printf_str not implemented for type {:?}", val)
            }
        }
    }

    pub fn build_br(&self, block: LLVMBasicBlockRef) -> LLVMValueRef {
        unsafe { LLVMBuildBr(self.builder, block) }
    }

    pub fn block_has_terminator(&self, block: LLVMBasicBlockRef) -> bool {
        unsafe { !LLVMGetBasicBlockTerminator(block).is_null() }
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

    pub fn build_helper_funcs(&mut self) {
        unsafe {
            let bool_to_str_func = self.build_bool_to_str_func();

            self.llvm_func_cache.set("bool_to_str", bool_to_str_func);
            let void_type: *mut llvm_sys::LLVMType = LLVMVoidTypeInContext(self.context);

            let printf_original_function_name =
                CString::new("printf").expect("CString::new failed");
            let printf_original_function =
                LLVMGetNamedFunction(self.module, printf_original_function_name.as_ptr());
            let print_func_type = LLVMFunctionType(void_type, [int8_ptr_type()].as_mut_ptr(), 1, 1);

            self.llvm_func_cache.set(
                "printf",
                LLVMCallFn {
                    function: printf_original_function,
                    func_type: print_func_type,
                },
            );
            load_string_helper_funcs(self.context, self.module, &mut self.llvm_func_cache);
            load_list_helper_funcs(self.context, self.module, &mut self.llvm_func_cache);
        }
    }

    pub fn build_bool_to_str_func(&self) -> LLVMCallFn {
        unsafe {
            // Create the function
            let char_ptr_type = LLVMPointerType(LLVMInt8TypeInContext(self.context), 0);
            let func_type = LLVMFunctionType(char_ptr_type, &mut int1_type(), 1, 0);
            let function = LLVMAddFunction(
                self.module,
                cstr_from_string("bool_to_str").as_ptr(),
                func_type,
            );

        // Create the basic blocks
        let entry_block = LLVMAppendBasicBlockInContext(
            self.context,
            function,
            cstr_from_string("entry").as_ptr(),
        );
        let then_block = LLVMAppendBasicBlockInContext(
            self.context,
            function,
            cstr_from_string("then").as_ptr(),
        );
        let else_block = LLVMAppendBasicBlockInContext(
            self.context,
            function,
            cstr_from_string("else").as_ptr(),
        );

        // Build the entry block
        let builder = LLVMCreateBuilderInContext(self.context);
        LLVMPositionBuilderAtEnd(builder, entry_block);
        let condition = LLVMGetParam(function, 0);

        LLVMBuildCondBr(builder, condition, then_block, else_block);

        // Build the 'then' block (return "true")
        let true_global = LLVMBuildGlobalString(
            builder,
            cstr_from_string("true\n").as_ptr(),
            cstr_from_string("true_str").as_ptr(),
        );

        LLVMPositionBuilderAtEnd(builder, then_block);
        LLVMBuildRet(builder, true_global);

        // Build the 'else' block (return "false")
        let false_global = LLVMBuildGlobalString(
            builder,
            cstr_from_string("false\n").as_ptr(),
            cstr_from_string("false_str").as_ptr(),
        );
        LLVMPositionBuilderAtEnd(builder, else_block);
        LLVMBuildRet(builder, false_global);

            LLVMCallFn {
                function,
                func_type,
            }
        }
    }

    pub fn icmp(
        &self,
        lhs: Box<dyn TypeBase>,
        rhs: Box<dyn TypeBase>,
        op: LLVMIntPredicate,
    ) -> Result<Box<dyn TypeBase>> {
        unsafe {
            match (lhs.get_ptr(), lhs.get_type()) {
                (Some(lhs_ptr), BaseTypes::Number) => {
                    let mut lhs_val =
                        self.build_load(lhs_ptr, lhs.get_llvm_type(), lhs.get_name_as_str());
                    let mut rhs_val = self.build_load(
                        rhs.get_ptr().unwrap(),
                        rhs.get_llvm_type(),
                        rhs.get_name_as_str(),
                    );
                    lhs_val = self.cast_i32_to_i64(lhs_val, rhs_val);
                    rhs_val = self.cast_i32_to_i64(rhs_val, lhs_val);
                    let cmp = LLVMBuildICmp(
                        self.builder,
                        op,
                        lhs_val,
                        rhs_val,
                        cstr_from_string("result").as_ptr(),
                    );
                    let alloca = self.build_alloca_store(cmp, int1_type(), "bool_cmp");
                    Ok(Box::new(BoolType {
                        name: lhs.get_name_as_str().to_string(),
                        builder: self.builder,
                        llvm_value: cmp,
                        llvm_value_pointer: alloca,
                    }))
                }
                _ => {
                    let mut lhs_val = lhs.get_value();
                    let mut rhs_val = rhs.get_value();
                    lhs_val = self.cast_i32_to_i64(lhs_val, rhs_val);
                    rhs_val = self.cast_i32_to_i64(rhs_val, lhs_val);
                    let cmp = LLVMBuildICmp(
                        self.builder,
                        op,
                        lhs_val,
                        rhs_val,
                        cstr_from_string("result").as_ptr(),
                    );
                    let alloca = self.build_alloca_store(cmp, int1_type(), "bool_cmp");
                    Ok(Box::new(BoolType {
                        name: lhs.get_name_as_str().to_string(),
                        builder: self.builder,
                        llvm_value: cmp,
                        llvm_value_pointer: alloca,
                    }))
                }
            }
        }
    }

    pub fn llvm_build_fn(&self, lhs: LLVMValueRef, rhs: LLVMValueRef, op: String) -> LLVMValueRef {
        unsafe {
            match op.as_str() {
                "+" => {
                    llvm_build_fn!(
                        LLVMBuildAdd,
                        self.builder,
                        lhs,
                        rhs,
                        cstr_from_string("addNumberType").as_ptr()
                    )
                }
                "-" => {
                    llvm_build_fn!(
                        LLVMBuildSub,
                        self.builder,
                        lhs,
                        rhs,
                        cstr_from_string("subNumberType").as_ptr()
                    )
                }
                "*" => {
                    llvm_build_fn!(
                        LLVMBuildMul,
                        self.builder,
                        lhs,
                        rhs,
                        cstr_from_string("mulNumberType").as_ptr()
                    )
                }
                "/" => {
                    llvm_build_fn!(
                        LLVMBuildSDiv,
                        self.builder,
                        lhs,
                        rhs,
                        cstr_from_string("mulNumberType").as_ptr()
                    )
                }
                _ => {
                    unreachable!()
                }
            }
        }
    }

    pub fn arithmetic_v2(
        &self,
        lhs: &GeneratedValue,
        rhs: &GeneratedValue,
        op: &String,
    ) -> Result<GeneratedValue> {
        match (&lhs.ty, &rhs.ty) {
            // String concatenation
            (ResolvedType::String, ResolvedType::String) => {
                if op != "+" {
                    return Err(anyhow!(
                        "Only + operator is supported for strings, got {}",
                        op
                    ));
                }
                let add_string_func = self
                    .llvm_func_cache
                    .get("stringAdd")
                    .ok_or_else(|| anyhow!("stringAdd function not found"))?;
                self.build_call(add_string_func, vec![lhs.value, rhs.value], 2, "");
                Ok(GeneratedValue {
                    value: lhs.value,
                    pointer: lhs.pointer,
                    ty: ResolvedType::String,
                })
            }
            // List concatenation
            (ResolvedType::List(lhs_inner), ResolvedType::List(rhs_inner))
                if lhs_inner == rhs_inner =>
            {
                if op != "+" {
                    return Err(anyhow!(
                        "Only + operator is supported for lists, got {}",
                        op
                    ));
                }
                let concat_func_name = match **lhs_inner {
                    ResolvedType::I32 => "concatInt32List",
                    ResolvedType::String => "concatStringList",
                    _ => {
                        return Err(anyhow!(
                            "List concatenation not supported for type {:?}",
                            lhs_inner
                        ))
                    }
                };
                let concat_func = self
                    .llvm_func_cache
                    .get(concat_func_name)
                    .ok_or_else(|| anyhow!("{} function not found", concat_func_name))?;
                let result =
                    self.build_call(concat_func, vec![lhs.value, rhs.value], 2, "list_concat");
                Ok(GeneratedValue {
                    value: result,
                    pointer: Some(result),
                    ty: ResolvedType::List(lhs_inner.clone()),
                })
            }
            // I32 arithmetic
            (ResolvedType::I32, ResolvedType::I32) => match (lhs.pointer, rhs.pointer) {
                (Some(ptr), Some(rhs_ptr)) => {
                    let mut lhs_val = self.build_load(ptr, int32_type(), "lhs");
                    let mut rhs_val = self.build_load(rhs_ptr, int32_type(), "rhs");
                    lhs_val = self.cast_i32_to_i64(lhs_val, rhs_val);
                    rhs_val = self.cast_i32_to_i64(rhs_val, lhs_val);
                    let result = self.llvm_build_fn(lhs_val, rhs_val, op.to_string());
                    let alloca = self.build_alloca_store(
                        result,
                        int32_ptr_type(), //todo fix
                        "lhs",
                    );
                    Ok(GeneratedValue {
                        value: result,
                        pointer: Some(alloca),
                        ty: ResolvedType::I32,
                    })
                }
                _ => {
                    let mut lhs_val = lhs.value;
                    let mut rhs_val = rhs.value;
                    lhs_val = self.cast_i32_to_i64(lhs_val, rhs_val);
                    rhs_val = self.cast_i32_to_i64(rhs_val, lhs_val);
                    let result = self.llvm_build_fn(lhs_val, rhs_val, op.to_string());
                    let alloca = self.build_alloca_store(
                        result,
                        int32_ptr_type(), //todo fix
                        "rhs",
                    );
                    Ok(GeneratedValue {
                        value: result,
                        pointer: Some(alloca),
                        ty: ResolvedType::I32,
                    })
                }
            },
            _ => Err(anyhow!(
                "Unsupported arithmetic operation {} for types {:?} and {:?}",
                op,
                lhs.ty,
                rhs.ty
            )),
        }
    }

    pub fn cmp(
        &self,
        lhs: Box<dyn TypeBase>,
        rhs: Box<dyn TypeBase>,
        op: String,
    ) -> Result<Box<dyn TypeBase>> {
        match rhs.get_type() {
            BaseTypes::String => {
                let is_string_equal_func = self
                    .llvm_func_cache
                    .get("isStringEqual")
                    .ok_or(anyhow!("unable to get function isStringEqual"))?;
                let is_string_equal_args = vec![lhs.get_ptr().unwrap(), rhs.get_ptr().unwrap()];

                let bool_value = self.build_call(is_string_equal_func, is_string_equal_args, 2, "");
                let alloca = self.build_alloca_store(bool_value, int1_type(), "");
                return Ok(Box::new(BoolType {
                    name: "bool_type".to_string(),
                    builder: self.builder,
                    llvm_value: bool_value,
                    llvm_value_pointer: alloca,
                }));
            }
            BaseTypes::Number | BaseTypes::Bool => {}
            _ => {
                unreachable!(
                    "Can't do operation type {:?} and type {:?}",
                    lhs.get_type(),
                    rhs.get_type()
                )
            }
        }
        match op.as_str() {
            "==" => self.icmp(lhs, rhs, LLVMIntEQ),
            "!=" => self.icmp(lhs, rhs, LLVMIntNE),
            "<" => self.icmp(lhs, rhs, LLVMIntSLT),
            "<=" => self.icmp(lhs, rhs, LLVMIntSLE),
            ">" => self.icmp(lhs, rhs, LLVMIntSGT),
            ">=" => self.icmp(lhs, rhs, LLVMIntSGE),
            _ => {
                unimplemented!()
            }
        }
    }

    pub fn get_string_type(&self) -> LLVMTypeRef {
        let string_struct_name = CString::new("struct.StringType").expect("CString::new failed");
        unsafe { LLVMGetTypeByName2(self.context, string_struct_name.as_ptr()) }
    }

    pub fn get_string_ptr_type(&self) -> LLVMTypeRef {
        unsafe { LLVMPointerType(self.get_string_type(), 0) }
    }

    pub fn get_list_int32_ptr_type(&self) -> LLVMTypeRef {
        int32_ptr_type()
    }

    pub fn get_list_string_ptr_type(&self) -> LLVMTypeRef {
        unsafe { LLVMPointerType(self.get_string_ptr_type(), 0) }
    }
}

fn extract_main_only_from_ir(module_ir: &str) -> Option<String> {
    let mut lines = module_ir.lines();
    let mut buf = String::new();
    let mut in_main = false;
    let mut brace_depth = 0i32;

    while let Some(line) = lines.next() {
        if !in_main {
            let trimmed = line.trim_start();
            if trimmed.starts_with("define ") && trimmed.contains("@main") {
                in_main = true;
                buf.push_str(line);
                buf.push('\n');
                brace_depth += line.matches('{').count() as i32;
                brace_depth -= line.matches('}').count() as i32;
                if brace_depth <= 0 && line.contains('}') {
                    return Some(buf);
                }
                continue;
            }
        } else {
            buf.push_str(line);
            buf.push('\n');
            brace_depth += line.matches('{').count() as i32;
            brace_depth -= line.matches('}').count() as i32;
            if brace_depth <= 0 {
                return Some(buf);
            }
        }
    }
    None
}

fn extract_main_with_called_from_ir(module_ir: &str) -> Option<String> {
    let main_ir = extract_main_only_from_ir(module_ir)?;
    let mut out = String::new();
    out.push_str(&main_ir);
    let calls = collect_called_functions(&main_ir);
    for name in calls {
        if let Some(def) = extract_function_def(module_ir, &name) {
            out.push('\n');
            out.push_str(&def);
        }
    }
    Some(out)
}

fn collect_called_functions(ir: &str) -> std::collections::HashSet<String> {
    let mut names = std::collections::HashSet::new();
    for line in ir.lines() {
        if !line.contains("call") {
            continue;
        }
        let mut i = 0;
        let bytes = line.as_bytes();
        while i < bytes.len() {
            if bytes[i] as char == '@' {
                let start = i + 1;
                let mut end = start;
                while end < bytes.len() {
                    let ch = bytes[end] as char;
                    if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' {
                        end += 1;
                    } else {
                        break;
                    }
                }
                if end > start {
                    let name = &line[start..end];
                    if !name.starts_with("llvm.") && name != "printf" {
                        names.insert(name.to_string());
                    }
                }
                i = end;
            } else {
                i += 1;
            }
        }
    }
    names
}

fn extract_function_def(module_ir: &str, name: &str) -> Option<String> {
    let needle = format!("@{name}");
    let mut lines = module_ir.lines();
    let mut buf = String::new();
    let mut in_fn = false;
    let mut brace_depth = 0i32;

    while let Some(line) = lines.next() {
        if !in_fn {
            let trimmed = line.trim_start();
            if trimmed.starts_with("define ") && trimmed.contains(&needle) {
                in_fn = true;
                buf.push_str(line);
                buf.push('\n');
                brace_depth += line.matches('{').count() as i32;
                brace_depth -= line.matches('}').count() as i32;
                if brace_depth <= 0 && line.contains('}') {
                    return Some(buf);
                }
                continue;
            }
        } else {
            buf.push_str(line);
            buf.push('\n');
            brace_depth += line.matches('{').count() as i32;
            brace_depth -= line.matches('}').count() as i32;
            if brace_depth <= 0 {
                return Some(buf);
            }
        }
    }
    None
}
