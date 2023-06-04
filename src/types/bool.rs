use crate::context::ASTContext;
use crate::types::llvm::*;
use std::any::Any;
extern crate llvm_sys;
use crate::types::{Base, BaseTypes, Comparison, Debug, TypeBase};
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::LLVMIntPredicate;

use super::Arithmetic;

macro_rules! c_str {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const i8
    };
}

#[derive(Debug, Clone)]
pub struct BoolType {
    pub builder: LLVMBuilderRef,
    value: bool,
    llmv_value: LLVMValueRef,
    llmv_value_pointer: LLVMValueRef,
    name: String,
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
                    int1_type(),
                );
            },
            _ => {
                unreachable!(
                    "Can't add type {:?} and type {:?}",
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
                    int1_type(),
                );
            },
            _ => {
                unreachable!(
                    "Can't add type {:?} and type {:?}",
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
    number_type: LLVMTypeRef,
) -> Box<dyn TypeBase> {
    // TODO: figure out how to print bool value as string?
    let cmp = LLVMBuildICmp(_context.builder, comparison, lhs, rhs, c_str!("result"));
    // let result_str = LLVMBuildIntToPtr(builder, result, int8_ptr_type(), c_str!(""));
    let bool_cmp = LLVMBuildZExt(_context.builder, cmp, number_type, c_str!("bool_cmp"));
    let bool_value = LLVMConstIntGetZExtValue(bool_cmp) != 0;
    return BoolType::new(Box::new(bool_value), _name, _context);
}

impl Arithmetic for BoolType {}

impl Debug for BoolType {
    fn print(&self, ast_context: &mut ASTContext) {
        unsafe {
            let value = LLVMBuildLoad2(
                self.builder,
                int1_ptr_type(),
                self.get_ptr(),
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
                value: value_as_bool,
                llmv_value: bool_value,
                llmv_value_pointer: alloca,
            })
        }
    }
    fn assign(&self, _ast_context: &mut ASTContext, _rhs: Box<dyn TypeBase>) {
        match _rhs.get_type() {
            BaseTypes::Bool => unsafe {
                // let alloca = _rhs.get_ptr();
                // let name = LLVMGetValueName(_rhs.get_value());
                // let new_value = LLVMBuildLoad2(self.builder, int1_ptr_type(), _rhs.get_value(), name);
                LLVMBuildStore(self.builder, _rhs.get_value(), self.get_ptr());
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
    fn get_ptr(&self) -> LLVMValueRef {
        self.llmv_value_pointer
    }
}
