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
        let then_block = context.append_basic_block(function, "then_block");
        let merge_block = context.append_basic_block(function, "merge_block");

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
        let else_block = context.append_basic_block(function, "else_block");
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

        let cmp = context.build_load(
            cond.get_ptr().unwrap(),
            int1_type(),
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

        let loop_cond_block = context.append_basic_block(function, "loop_cond");
        let loop_body_block = context.append_basic_block(function, "loop_body");
        let loop_exit_block = context.append_basic_block(function, "loop_exit");

        let bool_type_ptr = context.build_alloca(
            int1_type(),
            cstr_from_string("while_value_bool_var").as_ptr(),
        );
        let value_condition = context.match_ast(condition)?;

        let cmp = context.build_load(
            value_condition.get_ptr().unwrap(),
            int1_type(),
            cstr_from_string("cmp").as_ptr(),
        );

        context.build_store(cmp, bool_type_ptr);

        LLVMBuildBr(context.builder, loop_cond_block);

        context.set_current_block(loop_body_block);
        // Check if the global variable already exists

        context.match_ast(while_block_stmt)?;

        LLVMBuildBr(context.builder, loop_cond_block); // Jump back to loop condition

        context.set_current_block(loop_cond_block);
        // Build loop condition block
        let value_cond_load = context.build_load(
            value_condition.get_ptr().unwrap(),
            int1_type(),
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
        let function = context.current_function.function;
        context.set_current_block(for_block);

        let loop_cond_block = context.append_basic_block(function, "loop_cond");
        let loop_body_block = context.append_basic_block(function, "loop_body");
        let loop_exit_block = context.append_basic_block(function, "loop_exit");

        let i: Box<dyn TypeBase> = NumberType::new(Box::new(init), "i".to_string(), context);

        let value = i.get_value();
        let ptr = i.get_ptr();
        context.var_cache.set(&var_name, i, context.depth);

        context.build_store(value, ptr.unwrap());
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

        // Not sure why LLVMInt32TypeIntInContex
        let lhs_val = context.build_load(
            op_lhs.unwrap(),
            LLVMInt32TypeInContext(context.context),
            cstr_from_string("i").as_ptr(),
        );
        let loop_condition = LLVMBuildICmp(
            context.builder,
            op,
            lhs_val,
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
        let lhs_val = context.build_load(
            ptr.unwrap(),
            LLVMInt32TypeInContext(context.context),
            cstr_from_string("i").as_ptr(),
        );

        let new_value = LLVMBuildAdd(
            context.builder,
            lhs_val,
            LLVMConstInt(LLVMInt32TypeInContext(context.context), increment as u64, 0),
            cstr_from_string("i").as_ptr(),
        );
        context.build_store(new_value, ptr.unwrap());
        LLVMBuildBr(context.builder, loop_cond_block); // Jump back to loop condition

        // Position builder at loop exit block
        context.set_current_block(loop_exit_block);

        Ok(for_block_cond)
    }
}
