use crate::c_str;
use crate::compiler::llvm::context::ASTContext;
use crate::compiler::llvm::*;

use std::any::Any;
extern crate llvm_sys;
use crate::compiler::types::{Base, BaseTypes, Comparison, Debug, Func, TypeBase};
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::LLVMIntPredicate;

use super::Arithmetic;

#[derive(Debug, Clone)]
pub struct BoolType {
    pub builder: LLVMBuilderRef,
    pub llmv_value: LLVMValueRef,
    pub llmv_value_pointer: LLVMValueRef,
    pub name: String,
}

impl Base for BoolType {
    fn get_type(&self) -> BaseTypes {
        BaseTypes::Bool
    }
}

impl Comparison for BoolType {
    fn eqeq(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::Bool => unsafe {
                return get_comparison_bool_type(
                    self.name.clone(),
                    context,
                    _rhs.get_value(),
                    self.get_value(),
                    LLVMIntPredicate::LLVMIntEQ,
                );
            },
            _ => {
                unreachable!(
                    "Can't run '==' on type {:?} and type {:?}",
                    self.get_type(),
                    _rhs.get_type()
                )
            }
        }
    }

    fn ne(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::Bool => unsafe {
                return get_comparison_bool_type(
                    self.name.clone(),
                    context,
                    _rhs.get_value(),
                    self.get_value(),
                    LLVMIntPredicate::LLVMIntNE,
                );
            },
            _ => {
                unreachable!(
                    "Can't run '!=' type {:?} and type {:?}",
                    self.get_type(),
                    _rhs.get_type()
                )
            }
        }
    }
}

unsafe fn get_comparison_bool_type(
    _name: String,
    _context: &mut ASTContext,
    rhs: LLVMValueRef,
    lhs: LLVMValueRef,
    comparison: LLVMIntPredicate,
) -> Box<dyn TypeBase> {
    let cmp = LLVMBuildICmp(
        _context.builder,
        comparison,
        rhs,
        lhs,
        c_str!("result"),
    );
    // let result_str = LLVMBuildIntToPtr(builder, result, int8_ptr_type(), c_str!(""));
    // let bool_cmp = LLVMBuildZExt(_context.builder, cmp, number_type, c_str!("bool_cmp"));
    // let bool_value = LLVMConstIntGetZExtValue(bool_cmp) != 0;
    let alloca = LLVMBuildAlloca(_context.builder, int1_type(), c_str!("bool_cmp"));
    LLVMBuildStore(_context.builder, cmp, alloca);
    Box::new(BoolType {
        name: _name,
        builder: _context.builder,
        llmv_value: cmp,
        llmv_value_pointer: alloca,
    })
}

impl Arithmetic for BoolType {}

impl Debug for BoolType {
    fn print(&self, ast_context: &mut ASTContext) {
        unsafe {
            let value = LLVMBuildLoad2(
                self.builder,
                int1_type(),
                self.get_ptr().unwrap(),
                self.get_name(),
            );

            let bool_func_args: *mut *mut llvm_sys::LLVMValue = [value].as_mut_ptr();

            match ast_context.llvm_func_cache.get("bool_to_str") {
                Some(bool_to_string) => {
                    let str_value = LLVMBuildCall2(
                        ast_context.builder,
                        bool_to_string.func_type,
                        bool_to_string.function,
                        bool_func_args,
                        1,
                        c_str!(""),
                    );

                    let print_args: *mut *mut llvm_sys::LLVMValue = [str_value].as_mut_ptr();
                    match ast_context.llvm_func_cache.get("printf") {
                        Some(print_func) => {
                            LLVMBuildCall2(
                                ast_context.builder,
                                print_func.func_type,
                                print_func.function,
                                print_args,
                                1,
                                c_str!(""),
                            );
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                }
                _ => {
                    unreachable!()
                }
            }
        }
    }
}

impl TypeBase for BoolType {
    fn new(_value: Box<dyn Any>, _name: String, _context: &mut ASTContext) -> Box<dyn TypeBase>
    where
        Self: Sized,
    {
        let value_as_bool = match _value.downcast_ref::<bool>() {
            Some(val) => *val,
            None => panic!("The input value must be a bool"),
        };
        unsafe {
            let mut num = 0;
            if let true = value_as_bool {
                num = 1
            }
            let bool_value = LLVMConstInt(int1_type(), num, 0);
            let var_name = c_str(_name.as_str());
            // Check if the global variable already exists
            let alloca = LLVMBuildAlloca(_context.builder, int1_type(), var_name);
            LLVMBuildStore(_context.builder, bool_value, alloca);
            Box::new(BoolType {
                name: _name,
                builder: _context.builder,
                llmv_value: bool_value,
                llmv_value_pointer: alloca,
            })
        }
    }
    fn assign(&mut self, _ast_context: &mut ASTContext, _rhs: Box<dyn TypeBase>) {
        match _rhs.get_type() {
            BaseTypes::Bool => unsafe {
                let rhs_val = LLVMBuildLoad2(
                    _ast_context.builder,
                    int1_type(),
                    _rhs.get_ptr().unwrap(),
                    c_str!("load_bool"),
                );
                LLVMBuildStore(self.builder, rhs_val, self.get_ptr().unwrap());
            },
            _ => {
                unreachable!(
                    "Can't reassign variable {:?} that has type {:?} to type {:?}",
                    self.name,
                    self.get_type(),
                    _rhs.get_type()
                )
            }
        }
    }
    fn get_value(&self) -> LLVMValueRef {
        self.llmv_value
    }
    fn get_ptr(&self) -> Option<LLVMValueRef> {
        Some(self.llmv_value_pointer)
    }
}

impl Func for BoolType {}
