use crate::compiler::llvm::context::ASTContext;
use crate::compiler::llvm::cstr_from_string;
use crate::compiler::llvm::*;
use crate::compiler::types::{Arithmetic, Base, BaseTypes, Comparison, Debug, Func, TypeBase};
use cyclang_macros::{ArithmeticMacro, BaseMacro, ComparisonMacro, DebugMacro};
use std::any::Any;
use std::ffi::CString;

extern crate llvm_sys;
use llvm_sys::core::*;
use llvm_sys::prelude::*;

#[derive(Debug, Clone, ArithmeticMacro, ComparisonMacro, DebugMacro, BaseMacro)]
#[base_type("BaseTypes::Number")]
pub struct NumberType {
    //TODO: remove pub use of these
    pub llmv_value: LLVMValueRef,
    pub llmv_value_pointer: Option<LLVMValueRef>,
    pub name: String,
    pub cname: *const i8,
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
            let ptr = _context.build_alloca_store(value, int32_ptr_type(), c_pointer);
            let cname = cstr_from_string(_name.as_str()).as_ptr();
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
        match self.get_type() {
            BaseTypes::Number => unsafe {
                let alloca = self.get_ptr().unwrap();
                let name = LLVMGetValueName(self.get_value());
                // Tests pass for this but might need to double check, operation before didn't look like it was doing anything
                _ast_context.build_load_store(
                    alloca,
                    _rhs.get_ptr().unwrap(),
                    self.get_llvm_type(),
                    name,
                )
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
impl Func for NumberType {}
