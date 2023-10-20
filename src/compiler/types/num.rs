use crate::c_str;

use crate::compiler::llvm::*;
use std::any::Any;
use std::ffi::{CStr, CString};

use crate::compiler::llvm::context::ASTContext;
use crate::compiler::types::bool::BoolType;
use crate::compiler::types::{Arithmetic, Base, BaseTypes, Comparison, Debug, Func, TypeBase};

extern crate llvm_sys;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::LLVMIntPredicate;

#[derive(Debug, Clone)]
pub struct NumberType {
    //TODO: remove pub use of these
    pub llmv_value: LLVMValueRef,
    pub llmv_value_pointer: Option<LLVMValueRef>,
    pub name: String,
    pub cname: *const i8,
}

impl Base for NumberType {
    fn get_type(&self) -> BaseTypes {
        BaseTypes::Number
    }
}

macro_rules! generate_arithmetic_operation_fn {
    ($number_type:ident, $llvm_fn:ident, $operation:ident) => {
        fn $operation(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
            match _rhs.get_type() {
                BaseTypes::Number => unsafe {
                    match self.get_ptr() {
                        Some(_p) => {
                            let lhs_value = LLVMBuildLoad2(
                                context.builder,
                                self.get_llvm_type(),
                                self.get_ptr().unwrap(),
                                self.get_name(),
                            );
                            let rhs_value: *mut llvm_sys::LLVMValue = LLVMBuildLoad2(
                                context.builder,
                                self.get_llvm_type(),
                                _rhs.get_ptr().unwrap(),
                                _rhs.get_name(),
                            );
                            let result =
                                $llvm_fn(context.builder, lhs_value, rhs_value, c_str!("add_num"));
                            LLVMBuildStore(context.builder, result, self.get_ptr().unwrap());
                            //TODO: fix the new instruction
                            let c_str_ref = CStr::from_ptr(self.get_name());
                            // Convert the CStr to a String (handles invalid UTF-8)
                            let name = c_str_ref.to_string_lossy().to_string();
                            Box::new(NumberType {
                                name: name,
                                llmv_value: result,
                                llmv_value_pointer: self.get_ptr(),
                                cname: self.get_name(),
                            })
                        }
                        None => {
                            let result = $llvm_fn(
                                context.builder,
                                self.get_value(),
                                _rhs.get_value(),
                                c_str!("add_num"),
                            );
                            let alloca = LLVMBuildAlloca(
                                context.builder,
                                self.get_llvm_ptr_type(),
                                c_str!("param_add"),
                            );
                            LLVMBuildStore(context.builder, result, alloca);
                            let c_str_ref = CStr::from_ptr(self.get_name());

                            // Convert the CStr to a String (handles invalid UTF-8)
                            let name = c_str_ref.to_string_lossy().to_string();
                            //TODO: fix the new instruction
                            Box::new($number_type {
                                name: name,
                                llmv_value: result,
                                llmv_value_pointer: Some(alloca),
                                cname: self.get_name(),
                            })
                        }
                    }
                },
                _ => {
                    unreachable!(
                        "Can't {} type {:?} and type {:?}",
                        stringify!($operation),
                        self.get_type(),
                        _rhs.get_type()
                    )
                }
            }
        }
    };
}
macro_rules! generate_arithmetic_trait {
    ($number_type:ident, $llvm_add_fn:ident, $llvm_sub_fn:ident, $llvm_mul_fn:ident, $llvm_div_fn:ident) => {
        impl Arithmetic for $number_type {
            generate_arithmetic_operation_fn!($number_type, $llvm_add_fn, add);
            generate_arithmetic_operation_fn!($number_type, $llvm_sub_fn, sub);
            generate_arithmetic_operation_fn!($number_type, $llvm_mul_fn, mul);
            generate_arithmetic_operation_fn!($number_type, $llvm_div_fn, div);
        }
    };
}

generate_arithmetic_trait!(NumberType, LLVMBuildAdd, LLVMBuildSub, LLVMBuildMul, LLVMBuildSDiv);

impl Debug for NumberType {
    fn print(&self, ast_context: &mut ASTContext) {
        // Load Value from Value Index Ptr
        match self.get_ptr() {
            Some(v) => unsafe {
                let value: *mut llvm_sys::LLVMValue = LLVMBuildLoad2(
                    ast_context.builder,
                    self.get_llvm_type(),
                    v,
                    self.get_name(),
                );
                let mut print_args: Vec<LLVMValueRef> =
                    vec![ast_context.printf_str_num_value, value];
                match ast_context.llvm_func_cache.get("printf") {
                    Some(print_func) => {
                        LLVMBuildCall2(
                            ast_context.builder,
                            print_func.func_type,
                            print_func.function,
                            print_args.as_mut_ptr(),
                            2,
                            c_str!(""),
                        );
                    }
                    _ => {
                        unreachable!()
                    }
                }
            },
            None => match ast_context.llvm_func_cache.get("printf") {
                Some(print_func) => unsafe {
                    let mut print_args: Vec<LLVMValueRef> =
                        vec![ast_context.printf_str_num_value, self.get_value()];

                    LLVMBuildCall2(
                        ast_context.builder,
                        print_func.func_type,
                        print_func.function,
                        print_args.as_mut_ptr(),
                        2,
                        c_str!(""),
                    );
                },
                _ => {
                    unreachable!()
                }
            },
        }
    }
}

impl TypeBase for NumberType {
    fn new(_value: Box<dyn Any>, _name: String, _context: &mut ASTContext) -> Box<dyn TypeBase> {
        let value_as_i32 = match _value.downcast_ref::<i32>() {
            Some(val) => *val,
            None => panic!("The input value must be an i32"),
        };
        unsafe {
            let value = LLVMConstInt(int32_type(), value_as_i32.try_into().unwrap(), 0);
            let c_string = CString::new(_name.clone()).unwrap();
            let c_pointer: *const i8 = c_string.as_ptr();
            // Check if the global variable already exists
            let ptr = LLVMBuildAlloca(_context.builder, int32_ptr_type(), c_pointer);
            LLVMBuildStore(_context.builder, value, ptr);
            let cname = c_str!("var_num_var");
            Box::new(NumberType {
                name: _name,
                llmv_value: value,
                llmv_value_pointer: Some(ptr),
                cname,
            })
        }
    }
    unsafe fn get_name(&self) -> *const i8 {
        self.cname
    }
    fn assign(&mut self, _ast_context: &mut ASTContext, _rhs: Box<dyn TypeBase>) {
        match _rhs.get_type() {
            BaseTypes::Number => unsafe {
                let alloca = self.get_ptr().unwrap();
                let name = LLVMGetValueName(self.get_value());
                let new_value =
                    LLVMBuildLoad2(_ast_context.builder, self.get_llvm_type(), alloca, name);
                LLVMBuildStore(_ast_context.builder, new_value, alloca);
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
        self.llmv_value_pointer
    }
}

unsafe fn get_int32_arg_for_comp_self(
    val: NumberType,
    rhs: Box<dyn TypeBase>,
) -> [LLVMValueRef; 2] {
    match val.get_ptr() {
        Some(v) => [v, rhs.get_ptr().unwrap()],
        None => [val.get_value(), rhs.get_value()],
    }
}

impl Comparison for NumberType {
    fn eqeq(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        unsafe {
            get_comparison_number_type(
                self.name.clone(),
                context,
                _rhs.clone(),
                self.clone(),
                LLVMIntPredicate::LLVMIntEQ,
            )
        }
    }

    fn ne(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        unsafe {
            get_comparison_number_type(
                self.name.clone(),
                context,
                _rhs.clone(),
                self.clone(),
                LLVMIntPredicate::LLVMIntNE,
            )
        }
    }

    fn gt(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        unsafe {
            get_comparison_number_type(
                self.name.clone(),
                context,
                _rhs.clone(),
                self.clone(),
                LLVMIntPredicate::LLVMIntSGT,
            )
        }
    }

    fn gte(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        unsafe {
            get_comparison_number_type(
                self.name.clone(),
                context,
                _rhs.clone(),
                self.clone(),
                LLVMIntPredicate::LLVMIntSGE,
            )
        }
    }

    fn lt(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        unsafe {
            get_comparison_number_type(
                self.name.clone(),
                context,
                _rhs.clone(),
                self.clone(),
                LLVMIntPredicate::LLVMIntSLT,
            )
        }
    }

    fn lte(&self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        unsafe {
            get_comparison_number_type(
                self.name.clone(),
                context,
                _rhs.clone(),
                self.clone(),
                LLVMIntPredicate::LLVMIntSLE,
            )
        }
    }
}

impl Func for NumberType {}

unsafe fn get_comparison_number_type(
    _name: String,
    _context: &mut ASTContext,
    rhs: Box<dyn TypeBase>,
    lhs: NumberType,
    comparison: LLVMIntPredicate,
) -> Box<dyn TypeBase> {
    // First check type
    match rhs.get_type() {
        BaseTypes::Number => {}
        _ => {
            unreachable!(
                "Can't add type {:?} and type {:?}",
                lhs.get_type(),
                rhs.get_type()
            )
        }
    }
    // then do comparison
    match lhs.get_ptr() {
        Some(lhs_ptr) => {
            // If loading a pointer
            let lhs_val =
                LLVMBuildLoad2(_context.builder, int8_type(), lhs_ptr, c_str!("lhs_bool"));
            let rhs_val = LLVMBuildLoad2(
                _context.builder,
                int8_type(),
                rhs.get_ptr().unwrap(),
                c_str!("rhs_bool"),
            );
            let cmp = LLVMBuildICmp(
                _context.builder,
                comparison,
                lhs_val,
                rhs_val,
                c_str!("result"),
            );
            let alloca = LLVMBuildAlloca(_context.builder, int1_type(), c_str!("bool_cmp"));
            LLVMBuildStore(_context.builder, cmp, alloca);
            Box::new(BoolType {
                name: _name,
                builder: _context.builder,
                llmv_value: cmp,
                llmv_value_pointer: alloca,
            })
        }
        None => {
            // If loading a raw value
            let cmp = LLVMBuildICmp(
                _context.builder,
                comparison,
                lhs.get_value(),
                rhs.get_value(),
                c_str!("result"),
            );
            let alloca = LLVMBuildAlloca(_context.builder, int1_type(), c_str!("bool_cmp"));
            LLVMBuildStore(_context.builder, cmp, alloca);
            Box::new(BoolType {
                name: _name,
                builder: _context.builder,
                llmv_value: cmp,
                llmv_value_pointer: alloca,
            })
        }
    }
}
