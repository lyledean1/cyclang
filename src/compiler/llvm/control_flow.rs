use crate::c_str;
use crate::compiler::TypeBase;
use crate::parser::Expression;
extern crate llvm_sys;
use super::context::ASTContext;
use crate::compiler::int1_type;
use crate::compiler::NumberType;
use llvm_sys::core::*;
use llvm_sys::LLVMIntPredicate;

pub fn new_if_stmt(
    context: &mut ASTContext,
    condition: Box<Expression>,
    if_stmt: Box<Expression>,
    else_stmt: Box<Option<Expression>>,
) {
    unsafe {
        let function = context.current_function.function;
        let if_entry_block: *mut llvm_sys::LLVMBasicBlock = context.current_function.block;

        LLVMPositionBuilderAtEnd(context.builder, if_entry_block);

        let cond: Box<dyn TypeBase> = context.match_ast(*condition);
        // Build If Block
        let then_block = LLVMAppendBasicBlock(function, c_str!("then_block"));
        let merge_block = LLVMAppendBasicBlock(function, c_str!("merge_block"));

        context.set_current_block(then_block);

        context.match_ast(*if_stmt);

        // Each
        LLVMBuildBr(context.builder, merge_block); // Branch to merge_block

        // Build Else Block
        let else_block = LLVMAppendBasicBlock(function, c_str!("else_block"));
        context.set_current_block(else_block);

        match *else_stmt {
            Some(v_stmt) => {
                context.match_ast(v_stmt);
                LLVMBuildBr(context.builder, merge_block); // Branch to merge_block
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

        let cmp = LLVMBuildLoad2(context.builder, int1_type(), cond.get_ptr().unwrap(), c_str!("cmp"));
        LLVMBuildCondBr(context.builder, cmp, then_block, else_block);

        context.set_current_block(merge_block);
    }
}

pub fn new_while_stmt(
    context: &mut ASTContext,
    condition: Box<Expression>,
    while_block_stmt: Box<Expression>,
) -> Box<dyn TypeBase> {
    unsafe {
        let function = context.current_function.function;

        let loop_cond_block = LLVMAppendBasicBlock(function, c_str!("loop_cond"));
        let loop_body_block = LLVMAppendBasicBlock(function, c_str!("loop_body"));
        let loop_exit_block = LLVMAppendBasicBlock(function, c_str!("loop_exit"));

        // Set bool type in entry block
        let var_name = c_str!("while_value_bool_var");
        let bool_type_ptr = LLVMBuildAlloca(context.builder, int1_type(), var_name);
        let value_condition = context.match_ast(*condition);

        let cmp = LLVMBuildLoad2(
            context.builder,
            int1_type(),
            value_condition.get_ptr().unwrap(),
            c_str!("cmp"),
        );

        LLVMBuildStore(context.builder, cmp, bool_type_ptr);

        LLVMBuildBr(context.builder, loop_cond_block);

        context.set_current_block(loop_body_block);
        // Check if the global variable already exists

        context.match_ast(*while_block_stmt);

        LLVMBuildBr(context.builder, loop_cond_block); // Jump back to loop condition

        context.set_current_block(loop_cond_block);
        // Build loop condition block
        let value_cond_load = LLVMBuildLoad2(
            context.builder,
            int1_type(),
            value_condition.get_ptr().unwrap(),
            c_str!("while_value_bool_var"),
        );

        LLVMBuildCondBr(
            context.builder,
            value_cond_load,
            loop_body_block,
            loop_exit_block,
        );

        // Position builder at loop exit block
        context.set_current_block(loop_exit_block);
        value_condition
    }
}

pub fn new_for_loop(
    context: &mut ASTContext,
    var_name: String,
    init: i32,
    length: i32,
    increment: i32,
    for_block_expr: Box<Expression>,
) -> Box<dyn TypeBase> {
    unsafe {
        let for_block = context.current_function.block;

        context.set_current_block(for_block);
        let loop_cond_block =
            LLVMAppendBasicBlock(context.current_function.function, c_str!("loop_cond"));
        let loop_body_block =
            LLVMAppendBasicBlock(context.current_function.function, c_str!("loop_body"));
        // is this not needed?
        // let loop_incr_block =
        //     LLVMAppendBasicBlock(context.current_function.function, c_str!("loop_incr"));
        let loop_exit_block =
            LLVMAppendBasicBlock(context.current_function.function, c_str!("loop_exit"));

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
                c_str!(""),
            ),
            LLVMConstInt(
                LLVMInt32TypeInContext(context.context),
                op_rhs.try_into().unwrap(),
                0,
            ),
            c_str!(""),
        );
        LLVMBuildCondBr(
            context.builder,
            loop_condition,
            loop_body_block,
            loop_exit_block,
        );

        // Build loop body block
        context.set_current_block(loop_body_block);
        let for_block_cond = context.match_ast(*for_block_expr);

        let new_value = LLVMBuildAdd(
            context.builder,
            LLVMBuildLoad2(
                context.builder,
                LLVMInt32TypeInContext(context.context),
                ptr.unwrap(),
                c_str!(""),
            ),
            LLVMConstInt(LLVMInt32TypeInContext(context.context), increment as u64, 0),
            c_str!(""),
        );
        LLVMBuildStore(context.builder, new_value, ptr.unwrap());
        LLVMBuildBr(context.builder, loop_cond_block); // Jump back to loop condition

        // Position builder at loop exit block
        context.set_current_block(loop_exit_block);

        for_block_cond
    }
}
