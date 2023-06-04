use crate::types::llvm::*;
use std::any::Any;
use crate::context::ASTContext;
extern crate llvm_sys;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::LLVMIntPredicate;
use crate::types::{BaseTypes, TypeBase};

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
    fn get_value(&self) -> LLVMValueRef {
        self.llmv_value
    }
    fn set_value(&mut self, _value: LLVMValueRef) {
        self.llmv_value = _value;
    }
    fn get_type(&self) -> BaseTypes {
        BaseTypes::Bool
    }
    fn get_ptr(&self) -> LLVMValueRef {
        self.llmv_value_pointer
    }
    fn set_ptr(&mut self, _value: LLVMValueRef) {
        self.llmv_value_pointer = _value;
    }
    fn print(&self, ast_context: &mut ASTContext) {
        unsafe {
            let mut llvm_value_str =
                LLVMBuildGlobalStringPtr(ast_context.builder, c_str!("true"), c_str!("true_str"));
            if let false = self.value {
                llvm_value_str = LLVMBuildGlobalStringPtr(
                    ast_context.builder,
                    c_str!("false"),
                    c_str!("false_str"),
                );
            }
            let value_is_str =
                LLVMBuildGlobalStringPtr(ast_context.builder, c_str!("%s\n"), c_str!(""));
            let print_args = [value_is_str, llvm_value_str].as_mut_ptr();
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