use crate::compiler::llvm::context::ASTContext;
use crate::compiler::llvm::cstr_from_string;
use crate::compiler::llvm::*;
use crate::compiler::types::{Arithmetic, Base, BaseTypes, Comparison, Debug, Func, TypeBase};
use cyclang_macros::{ArithmeticMacro, BaseMacro, ComparisonMacro, DebugMacro};
use std::any::Any;

extern crate llvm_sys;
use llvm_sys::core::*;
use llvm_sys::prelude::*;

#[derive(Debug, Clone, ArithmeticMacro, ComparisonMacro, DebugMacro, BaseMacro)]
#[base_type("BaseTypes::Number")]
pub struct NumberType {
    //TODO: remove pub use of these
    pub llvm_value: LLVMValueRef,
    pub llvm_value_pointer: Option<LLVMValueRef>,
    pub name: String,
}

impl TypeBase for NumberType {
    fn new(_value: Box<dyn Any>, name: String, context: &mut ASTContext) -> Box<dyn TypeBase> {
        let value_as_i32 = match _value.downcast_ref::<i32>() {
            Some(val) => *val,
            None => panic!("The input value must be an i32"),
        };
        let value = context.const_int(int32_type(), value_as_i32.try_into().unwrap(), 0);
        let ptr = context.build_alloca_store(value, int32_ptr_type(), &name);
        Box::new(NumberType {
            name: name,
            llvm_value: value,
            llvm_value_pointer: Some(ptr),
        })
    }
    fn assign(&mut self, context: &mut ASTContext, _rhs: Box<dyn TypeBase>) {
        match self.get_type() {
            BaseTypes::Number => context.build_load_store(
                _rhs.get_ptr().unwrap(),
                self.get_ptr().unwrap(),
                self.get_llvm_type(),
                self.get_name_as_str(),
            ),
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
        self.llvm_value
    }
    fn get_ptr(&self) -> Option<LLVMValueRef> {
        self.llvm_value_pointer
    }
}
impl Func for NumberType {}
