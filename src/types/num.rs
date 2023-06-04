use crate::types::llvm::*;
use std::any::Any;

use crate::context::ASTContext;
use crate::types::{BaseTypes, BoolType, TypeBase};

extern crate llvm_sys;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::LLVMIntPredicate;

macro_rules! c_str {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const i8
    };
}

#[derive(Debug, Clone)]
pub struct NumberType {
    //TODO: remove pub use of these
    pub llmv_value: LLVMValueRef,
    pub llmv_value_pointer: LLVMValueRef,
    name: String,
}

impl TypeBase for NumberType {
    fn new(_value: Box<dyn Any>, _name: String, _context: &mut ASTContext) -> Box<dyn TypeBase> {
        let value_as_i32 = match _value.downcast_ref::<i32>() {
            Some(val) => *val,
            None => panic!("The input value must be an i32"),
        };
        unsafe {
            let value = LLVMConstInt(int32_type(), value_as_i32.try_into().unwrap(), 0);
            let ptr = LLVMBuildAlloca(_context.builder, int32_ptr_type(), c_str(_name.as_str()));
            LLVMBuildStore(_context.builder, value, ptr);
            Box::new(NumberType {
                name: _name,
                llmv_value: value,
                llmv_value_pointer: ptr,
            })
        }
    }
    fn get_value(&self) -> LLVMValueRef {
        self.llmv_value
    }
    fn set_value(&mut self, _value: LLVMValueRef) {
        self.llmv_value = _value
    }
    fn get_type(&self) -> BaseTypes {
        BaseTypes::Number
    }
    fn get_ptr(&self) -> LLVMValueRef {
        self.llmv_value_pointer
    }
    fn set_ptr(&mut self, _value: LLVMValueRef) {
        self.llmv_value_pointer = _value;
    }
    fn add(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::Number => unsafe {
                let result = LLVMBuildAdd(
                    context.builder,
                    self.get_value(),
                    _rhs.get_value(),
                    c_str!("result"),
                );
                return NumberType::new(
                    Box::new(get_i32_value(result)),
                    self.name.clone(),
                    context,
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

    fn sub(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::Number => unsafe {
                let result = LLVMBuildSub(
                    context.builder,
                    self.get_value(),
                    _rhs.get_value(),
                    c_str!("result"),
                );

                return NumberType::new(
                    Box::new(get_i32_value(result)),
                    self.name.clone(),
                    context,
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

    fn mul(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::Number => unsafe {
                let result = LLVMBuildMul(
                    context.builder,
                    self.get_value(),
                    _rhs.get_value(),
                    c_str!("result"),
                );
                return NumberType::new(
                    Box::new(get_i32_value(result)),
                    self.name.clone(),
                    context,
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

    fn div(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::Number => unsafe {
                let result = LLVMBuildSDiv(
                    context.builder,
                    self.get_value(),
                    _rhs.get_value(),
                    c_str!("result"),
                );
                return NumberType::new(
                    Box::new(get_i32_value(result)),
                    self.name.clone(),
                    context,
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

    fn eqeq(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::Number => unsafe {
                return get_comparison_number_type(
                    self.name.clone(),
                    context,
                    _rhs.get_value(),
                    self.get_value(),
                    LLVMIntPredicate::LLVMIntEQ,
                    int8_type(),
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
            BaseTypes::Number => unsafe {
                return get_comparison_number_type(
                    self.name.clone(),
                    context,
                    _rhs.get_value(),
                    self.get_value(),
                    LLVMIntPredicate::LLVMIntNE,
                    int8_type(),
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

    fn gt(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::Number => unsafe {
                return get_comparison_number_type(
                    self.name.clone(),
                    context,
                    _rhs.get_value(),
                    self.get_value(),
                    LLVMIntPredicate::LLVMIntSGT,
                    int8_type(),
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

    fn gte(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::Number => unsafe {
                return get_comparison_number_type(
                    self.name.clone(),
                    context,
                    _rhs.get_value(),
                    self.get_value(),
                    LLVMIntPredicate::LLVMIntSGE,
                    int8_type(),
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

    fn lt(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::Number => unsafe {
                return get_comparison_number_type(
                    self.name.clone(),
                    context,
                    _rhs.get_value(),
                    self.get_value(),
                    LLVMIntPredicate::LLVMIntSLT,
                    int8_type(),
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

    fn lte(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::Number => unsafe {
                return get_comparison_number_type(
                    self.name.clone(),
                    context,
                    _rhs.get_value(),
                    self.get_value(),
                    LLVMIntPredicate::LLVMIntSLE,
                    int8_type(),
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

    fn print(&self, ast_context: &mut ASTContext) {
        unsafe {
            let format_str = "%d\n\0";
            let value_is_str = LLVMBuildGlobalStringPtr(
                ast_context.builder,
                format_str.as_ptr() as *const i8,
                c_str!(""),
            );
            // Load Value from Value Index Ptr
            let val = LLVMBuildLoad2(
                ast_context.builder,
                int32_ptr_type(),
                self.get_ptr(),
                c_str!("load_number_ptr"),
            );

            let print_args = [value_is_str, val].as_mut_ptr();
            match ast_context.llvm_func_cache.get("printf") {
                Some(print_func) => {
                    LLVMBuildCall2(
                        ast_context.builder,
                        print_func.func_type,
                        print_func.function,
                        print_args,
                        2,
                        c_str!(""),
                    );
                }
                _ => {
                    unreachable!()
                }
            }
        }
    }
}

unsafe fn get_comparison_number_type(
    _name: String,
    _context: &mut ASTContext,
    rhs: LLVMValueRef,
    lhs: LLVMValueRef,
    comparison: LLVMIntPredicate,
    number_type: LLVMTypeRef,
) -> Box<dyn TypeBase> {
    let cmp = LLVMBuildICmp(_context.builder, comparison, lhs, rhs, c_str!("result"));
    // let result_str = LLVMBuildIntToPtr(builder, result, int8_ptr_type(), c_str!(""));
    let bool_cmp = LLVMBuildZExt(_context.builder, cmp, number_type, c_str!("bool_cmp"));
    let bool_value = LLVMConstIntGetZExtValue(bool_cmp) != 0;

    return BoolType::new(Box::new(bool_value), _name, _context);
}
