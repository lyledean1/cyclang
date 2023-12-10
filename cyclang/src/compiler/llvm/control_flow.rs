use crate::compiler::types::return_type::ReturnType;
use crate::compiler::types::void::VoidType;
use crate::compiler::types::BaseTypes;
use crate::compiler::TypeBase;
use crate::parser::Expression;
extern crate llvm_sys;
use super::context::ASTContext;
use crate::compiler::int1_type;
use crate::compiler::llvm::cstr_from_string;
use crate::compiler::NumberType;
use crate::cyclo_error::CycloError;
use llvm_sys::core::*;
use llvm_sys::LLVMIntPredicate;

pub fn new_if_stmt(
    context: &mut ASTContext,
    condition: Expression,
    if_stmt: Expression,
    else_stmt: Option<Expression>,
) -> Result<Box<dyn TypeBase>, CycloError> {
    unsafe {
        let mut return_type: Box<dyn TypeBase> = Box::new(VoidType {});
        let function = context.current_function.function;
        let if_entry_block: *mut llvm_sys::LLVMBasicBlock = context.current_function.block;

        LLVMPositionBuilderAtEnd(context.builder, if_entry_block);

        let cond: Box<dyn TypeBase> = context.match_ast(condition)?;
        // Build If Block
        let then_block = LLVMAppendBasicBlock(function, cstr_from_string("then_block").as_ptr());
        let merge_block = LLVMAppendBasicBlock(function, cstr_from_string("merge_block").as_ptr());

        context.set_current_block(then_block);

        let stmt = context.match_ast(if_stmt)?;

        match stmt.get_type() {
            BaseTypes::Return => {
                // if its a return type we will skip branching in the LLVM IR
                return_type = Box::new(ReturnType {});
            }
            _ => {
                LLVMBuildBr(context.builder, merge_block); // Branch to merge_block
            }
        }
        // Each

        // Build Else Block
        let else_block = LLVMAppendBasicBlock(function, cstr_from_string("else_block").as_ptr());
        context.set_current_block(else_block);

        match else_stmt {
            Some(v_stmt) => {
                let stmt = context.match_ast(v_stmt)?;
                match stmt.get_type() {
                    BaseTypes::Return => {
                        // if its a return type we will skip branching in the LLVM IR
                        return_type = Box::new(ReturnType {});
                    }
                    _ => {
                        LLVMBuildBr(context.builder, merge_block); // Branch to merge_block
                    }
                }
            }
            _ => {
                LLVMPositionBuilderAtEnd(context.builder, else_block);
                LLVMBuildBr(context.builder, merge_block); // Branch to merge_block
            }
        }

        // E
        LLVMPositionBuilderAtEnd(context.builder, merge_block);
        context.set_current_block(merge_block);

        context.set_current_block(if_entry_block);

        let cmp = LLVMBuildLoad2(
            context.builder,
            int1_type(),
            cond.get_ptr().unwrap(),
            cstr_from_string("cmp").as_ptr(),
        );
        LLVMBuildCondBr(context.builder, cmp, then_block, else_block);

        context.set_current_block(merge_block);
        Ok(return_type)
    }
}

pub fn new_while_stmt(
    context: &mut ASTContext,
    condition: Expression,
    while_block_stmt: Expression,
) -> Result<Box<dyn TypeBase>, CycloError> {
    unsafe {
        let function = context.current_function.function;

        let loop_cond_block =
            LLVMAppendBasicBlock(function, cstr_from_string("loop_cond").as_ptr());
        let loop_body_block =
            LLVMAppendBasicBlock(function, cstr_from_string("loop_body").as_ptr());
        let loop_exit_block =
            LLVMAppendBasicBlock(function, cstr_from_string("loop_exit").as_ptr());

        // Set bool type in entry block
        let bool_type_ptr = LLVMBuildAlloca(
            context.builder,
            int1_type(),
            cstr_from_string("while_value_bool_var").as_ptr(),
        );
        let value_condition = context.match_ast(condition)?;

        let cmp = LLVMBuildLoad2(
            context.builder,
            int1_type(),
            value_condition.get_ptr().unwrap(),
            cstr_from_string("cmp").as_ptr(),
        );

        LLVMBuildStore(context.builder, cmp, bool_type_ptr);

        LLVMBuildBr(context.builder, loop_cond_block);

        context.set_current_block(loop_body_block);
        // Check if the global variable already exists

        context.match_ast(while_block_stmt)?;

        LLVMBuildBr(context.builder, loop_cond_block); // Jump back to loop condition

        context.set_current_block(loop_cond_block);
        // Build loop condition block
        let value_cond_load = LLVMBuildLoad2(
            context.builder,
            int1_type(),
            value_condition.get_ptr().unwrap(),
            cstr_from_string("while_value_bool_var").as_ptr(),
        );

        LLVMBuildCondBr(
            context.builder,
            value_cond_load,
            loop_body_block,
            loop_exit_block,
        );

        // Position builder at loop exit block
        context.set_current_block(loop_exit_block);
        Ok(value_condition)
    }
}

pub fn new_for_loop(
    context: &mut ASTContext,
    var_name: String,
    init: i32,
    length: i32,
    increment: i32,
    for_block_expr: Expression,
) -> Result<Box<dyn TypeBase>, CycloError> {
    unsafe {
        let for_block = context.current_function.block;

        context.set_current_block(for_block);
        let loop_cond_block = LLVMAppendBasicBlock(
            context.current_function.function,
            cstr_from_string("loop_cond").as_ptr(),
        );
        let loop_body_block = LLVMAppendBasicBlock(
            context.current_function.function,
            cstr_from_string("loop_body").as_ptr(),
        );
        let loop_exit_block = LLVMAppendBasicBlock(
            context.current_function.function,
            cstr_from_string("loop_exit").as_ptr(),
        );

        let i: Box<dyn TypeBase> = NumberType::new(Box::new(init), "i".to_string(), context);

        let value = i.get_value();
        let ptr = i.get_ptr();
        context.var_cache.set(&var_name, i, context.depth);

        LLVMBuildStore(context.builder, value, ptr.unwrap());
        // Branch to loop condition block
        LLVMBuildBr(context.builder, loop_cond_block);

        // Build loop condition block
        context.set_current_block(loop_cond_block);

        // TODO: improve this logic for identifying for and reverse fors
        let mut op = LLVMIntPredicate::LLVMIntSLT;
        if increment < 0 {
            op = LLVMIntPredicate::LLVMIntSGT;
        }

        let op_lhs = ptr;
        let op_rhs = length;
        let loop_condition = LLVMBuildICmp(
            context.builder,
            op,
            LLVMBuildLoad2(
                context.builder,
                LLVMInt32TypeInContext(context.context),
                op_lhs.unwrap(),
                cstr_from_string("i").as_ptr(),
            ),
            LLVMConstInt(
                LLVMInt32TypeInContext(context.context),
                op_rhs.try_into().unwrap(),
                0,
            ),
            cstr_from_string("").as_ptr(),
        );
        LLVMBuildCondBr(
            context.builder,
            loop_condition,
            loop_body_block,
            loop_exit_block,
        );

        // Build loop body block
        context.set_current_block(loop_body_block);
        let for_block_cond = context.match_ast(for_block_expr)?;

        let new_value = LLVMBuildAdd(
            context.builder,
            LLVMBuildLoad2(
                context.builder,
                LLVMInt32TypeInContext(context.context),
                ptr.unwrap(),
                cstr_from_string("i").as_ptr(),
            ),
            LLVMConstInt(LLVMInt32TypeInContext(context.context), increment as u64, 0),
            cstr_from_string("i").as_ptr(),
        );
        LLVMBuildStore(context.builder, new_value, ptr.unwrap());
        LLVMBuildBr(context.builder, loop_cond_block); // Jump back to loop condition

        // Position builder at loop exit block
        context.set_current_block(loop_exit_block);

        Ok(for_block_cond)
    }
}
