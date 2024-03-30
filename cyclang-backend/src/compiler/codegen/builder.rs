use crate::compiler::codegen::context::{LLVMFunction, LLVMFunctionCache};
use crate::compiler::codegen::{cstr_from_string, int1_type, int64_type, int8_ptr_type};
use crate::compiler::context::ASTContext;
use crate::compiler::types::num::NumberType;
use crate::compiler::types::return_type::ReturnType;
use crate::compiler::types::void::VoidType;
use crate::compiler::types::{BaseTypes, TypeBase};
use crate::compiler::CompileOptions;
use anyhow::Result;
use cyclang_parser::{Expression, Type};
use libc::c_uint;
use llvm_sys::core::{
    LLVMAddFunction, LLVMAppendBasicBlock, LLVMAppendBasicBlockInContext, LLVMArrayType2,
    LLVMBuildAdd, LLVMBuildAlloca, LLVMBuildBr, LLVMBuildCall2, LLVMBuildCondBr, LLVMBuildGEP2,
    LLVMBuildGlobalStringPtr, LLVMBuildICmp, LLVMBuildLoad2, LLVMBuildRet, LLVMBuildRetVoid,
    LLVMBuildSExt, LLVMBuildStore, LLVMConstArray2, LLVMConstInt, LLVMContextCreate,
    LLVMContextDispose, LLVMCreateBuilderInContext, LLVMDisposeBuilder, LLVMDisposeMessage,
    LLVMDisposeModule, LLVMFunctionType, LLVMGetIntTypeWidth, LLVMGetParam, LLVMInt32TypeInContext,
    LLVMInt8TypeInContext, LLVMModuleCreateWithName, LLVMPointerType, LLVMPositionBuilderAtEnd,
    LLVMPrintModuleToFile, LLVMSetTarget, LLVMTypeOf, LLVMVoidTypeInContext,
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
use llvm_sys::LLVMIntPredicate;
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

            let llvm_func_cache = LLVMFunctionCache::new();

            let mut codegen_builder = LLVMCodegenBuilder {
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
            };
            codegen_builder.build_helper_funcs(main_block);
            Ok(codegen_builder)
        }
    }

    pub fn dispose_and_get_module_str(&self) -> Result<String> {
        unsafe {
            self.build_ret_void();

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
            _ => {
                unreachable!("get_printf_str not implemented for type {:?}", val)
            }
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

    pub fn new_if_stmt(
        context: &mut ASTContext,
        condition: Expression,
        if_stmt: Expression,
        else_stmt: Option<Expression>,
    ) -> Result<Box<dyn TypeBase>> {
        let mut return_type: Box<dyn TypeBase> = Box::new(VoidType {});
        let function = context.codegen.current_function.function;
        let if_entry_block: *mut llvm_sys::LLVMBasicBlock = context.codegen.current_function.block;

        context.codegen.position_builder_at_end(if_entry_block);

        let cond: Box<dyn TypeBase> = context.match_ast(condition)?;
        // Build If Block
        let then_block = context.codegen.append_basic_block(function, "then_block");
        let merge_block = context.codegen.append_basic_block(function, "merge_block");

        context.codegen.set_current_block(then_block);

        let stmt = context.match_ast(if_stmt)?;

        match stmt.get_type() {
            BaseTypes::Return => {
                // if its a return type we will skip branching in the LLVM IR
                return_type = Box::new(ReturnType {});
            }
            _ => {
                context.codegen.build_br(merge_block); // Branch to merge_block
            }
        }
        // Each

        // Build Else Block
        let else_block = context.codegen.append_basic_block(function, "else_block");
        context.codegen.set_current_block(else_block);

        match else_stmt {
            Some(v_stmt) => {
                let stmt = context.match_ast(v_stmt)?;
                match stmt.get_type() {
                    BaseTypes::Return => {
                        // if its a return type we will skip branching in the LLVM IR
                        return_type = Box::new(ReturnType {});
                    }
                    _ => {
                        context.codegen.build_br(merge_block);
                    }
                }
            }
            _ => {
                context.codegen.position_builder_at_end(else_block);
                context.codegen.build_br(merge_block);
            }
        }

        context.codegen.position_builder_at_end(merge_block);
        context.codegen.set_current_block(merge_block);

        context.codegen.set_current_block(if_entry_block);

        let cmp = context
            .codegen
            .build_load(cond.get_ptr().unwrap(), int1_type(), "cmp");
        context.codegen.build_cond_br(cmp, then_block, else_block);

        context.codegen.set_current_block(merge_block);
        Ok(return_type)
    }

    pub fn new_while_stmt(
        context: &mut ASTContext,
        condition: Expression,
        while_block_stmt: Expression,
    ) -> Result<Box<dyn TypeBase>> {
        let function = context.codegen.current_function.function;

        let loop_cond_block = context.codegen.append_basic_block(function, "loop_cond");
        let loop_body_block = context.codegen.append_basic_block(function, "loop_body");
        let loop_exit_block = context.codegen.append_basic_block(function, "loop_exit");

        let bool_type_ptr = context
            .codegen
            .build_alloca(int1_type(), "while_value_bool_var");
        let value_condition = context.match_ast(condition)?;

        let cmp =
            context
                .codegen
                .build_load(value_condition.get_ptr().unwrap(), int1_type(), "cmp");

        context.codegen.build_store(cmp, bool_type_ptr);

        context.codegen.build_br(loop_cond_block);

        context.codegen.set_current_block(loop_body_block);
        // Check if the global variable already exists

        context.match_ast(while_block_stmt)?;

        context.codegen.build_br(loop_cond_block); // Jump back to loop condition

        context.codegen.set_current_block(loop_cond_block);
        // Build loop condition block
        let value_cond_load = context.codegen.build_load(
            value_condition.get_ptr().unwrap(),
            int1_type(),
            "while_value_bool_var",
        );

        context
            .codegen
            .build_cond_br(value_cond_load, loop_body_block, loop_exit_block);

        // Position builder at loop exit block
        context.codegen.set_current_block(loop_exit_block);
        Ok(value_condition)
    }

    pub fn new_for_loop(
        context: &mut ASTContext,
        var_name: String,
        init: i32,
        length: i32,
        increment: i32,
        for_block_expr: Expression,
    ) -> Result<Box<dyn TypeBase>> {
        unsafe {
            let for_block = context.codegen.current_function.block;
            let function = context.codegen.current_function.function;
            context.codegen.set_current_block(for_block);

            let loop_cond_block = context.codegen.append_basic_block(function, "loop_cond");
            let loop_body_block = context.codegen.append_basic_block(function, "loop_body");
            let loop_exit_block = context.codegen.append_basic_block(function, "loop_exit");

            let i: Box<dyn TypeBase> = NumberType::new(Box::new(init), "i".to_string(), context);

            let value = i.get_value();
            let ptr = i.get_ptr();
            context.var_cache.set(&var_name, i, context.depth);

            context.codegen.build_store(value, ptr.unwrap());
            // Branch to loop condition block
            context.codegen.build_br(loop_cond_block);

            // Build loop condition block
            context.codegen.set_current_block(loop_cond_block);

            // TODO: improve this logic for identifying for and reverse fors
            let mut op = LLVMIntPredicate::LLVMIntSLT;
            if increment < 0 {
                op = LLVMIntPredicate::LLVMIntSGT;
            }

            let op_lhs = ptr;
            let op_rhs = length;

            // Not sure why LLVMInt32TypeIntInContex
            let lhs_val = context.codegen.build_load(
                op_lhs.unwrap(),
                LLVMInt32TypeInContext(context.codegen.context),
                "i",
            );

            let icmp_val = context.codegen.const_int(
                LLVMInt32TypeInContext(context.codegen.context),
                op_rhs.try_into().unwrap(),
                0,
            );
            let loop_condition = LLVMBuildICmp(
                context.codegen.builder,
                op,
                lhs_val,
                icmp_val,
                cstr_from_string("").as_ptr(),
            );

            context
                .codegen
                .build_cond_br(loop_condition, loop_body_block, loop_exit_block);

            // Build loop body block
            context.codegen.set_current_block(loop_body_block);
            let for_block_cond = context.match_ast(for_block_expr)?;
            let lhs_val = context.codegen.build_load(
                ptr.unwrap(),
                LLVMInt32TypeInContext(context.codegen.context),
                "i",
            );

            let incr_val = context.codegen.const_int(
                LLVMInt32TypeInContext(context.codegen.context),
                increment as u64,
                0,
            );

            let new_value = LLVMBuildAdd(
                context.codegen.builder,
                lhs_val,
                incr_val,
                cstr_from_string("i").as_ptr(),
            );
            context.codegen.build_store(new_value, ptr.unwrap());
            context.codegen.build_br(loop_cond_block); // Jump back to loop condition

            // Position builder at loop exit block
            context.codegen.set_current_block(loop_exit_block);

            Ok(for_block_cond)
        }
    }

    pub fn build_helper_funcs(&mut self, main_block: LLVMBasicBlockRef) {
        unsafe {
            let bool_to_str_func = self.build_bool_to_str_func();

            self.llvm_func_cache.set("bool_to_str", bool_to_str_func);
            let void_type: *mut llvm_sys::LLVMType = LLVMVoidTypeInContext(self.context);

            //printf
            let print_func_type = LLVMFunctionType(void_type, [int8_ptr_type()].as_mut_ptr(), 1, 1);
            let print_func = LLVMAddFunction(
                self.module,
                cstr_from_string("printf").as_ptr(),
                print_func_type,
            );
            self.llvm_func_cache.set(
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
                LLVMPointerType(LLVMInt8TypeInContext(self.context), 0),
                LLVMPointerType(LLVMInt8TypeInContext(self.context), 0),
                LLVMPointerType(LLVMInt8TypeInContext(self.context), 0),
                LLVMPointerType(LLVMInt8TypeInContext(self.context), 0),
            ];
            let ret_type = LLVMPointerType(LLVMInt8TypeInContext(self.context), 0);
            let sprintf_type =
                LLVMFunctionType(ret_type, arg_types.as_mut_ptr(), arg_types.len() as u32, 1);
            let sprintf = LLVMAddFunction(
                self.module,
                cstr_from_string("sprintf").as_ptr(),
                sprintf_type,
            );
            self.llvm_func_cache.set(
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
        }
    }

    pub unsafe fn build_bool_to_str_func(&self) -> LLVMFunction {
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
        let true_global = LLVMBuildGlobalStringPtr(
            builder,
            cstr_from_string("true\n").as_ptr(),
            cstr_from_string("true_str").as_ptr(),
        );

        LLVMPositionBuilderAtEnd(builder, then_block);
        LLVMBuildRet(builder, true_global);

        // Build the 'else' block (return "false")
        let false_global = LLVMBuildGlobalStringPtr(
            builder,
            cstr_from_string("false\n").as_ptr(),
            cstr_from_string("false_str").as_ptr(),
        );
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
}
