#![allow(dead_code)]

use std::any::Any;


use std::ffi::CString;
use dyn_clone::DynClone;
use crate::parser::Expression;
use crate::context::ASTContext;
use std::os::raw::c_ulonglong;

extern crate llvm_sys;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::LLVMIntPredicate;

macro_rules! c_str {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const i8
    };
}

const LLVM_FALSE: LLVMBool = 0;
const LLVM_TRUE: LLVMBool = 1;

// Types

fn create_string_type(context: LLVMContextRef) -> LLVMTypeRef {
    unsafe {
        // Create an LLVM 8-bit integer type (i8) to represent a character
        let i8_type = LLVMInt8TypeInContext(context);

        // Create a pointer type to the i8 type to represent a string
        LLVMPointerType(i8_type, 0)
    }
}
/// Convert this integer to LLVM's representation of a constant
/// integer.
unsafe fn int8(val: c_ulonglong) -> LLVMValueRef {
    LLVMConstInt(LLVMInt8Type(), val, LLVM_FALSE)
}
/// Convert this integer to LLVM's representation of a constant
/// integer.
// TODO: this should be a machine word size rather than hard-coding 32-bits.
fn int32(val: c_ulonglong) -> LLVMValueRef {
    unsafe { LLVMConstInt(LLVMInt32Type(), val, LLVM_FALSE) }
}

fn int1_type() -> LLVMTypeRef {
    unsafe { LLVMInt1Type() }
}

fn int8_type() -> LLVMTypeRef {
    unsafe { LLVMInt8Type() }
}

fn int32_type() -> LLVMTypeRef {
    unsafe { LLVMInt32Type() }
}

fn int8_ptr_type() -> LLVMTypeRef {
    unsafe { LLVMPointerType(LLVMInt8Type(), 0) }
}

fn bool_type(context: LLVMContextRef, boolean: bool) -> LLVMValueRef {
    unsafe {
        let bool_type = LLVMInt1TypeInContext(context);

        // Create a LLVM value for the bool
        let bool_value = LLVMConstInt(bool_type, boolean as u64, 0);

        // Return the LLVMValueRef for the bool
        bool_value
    }
}


#[derive(Debug)]
pub enum BaseTypes {
    String,
    Number,
    Bool,
    Block,
}
// Types
pub trait TypeBase: DynClone {
    fn new(_value: Box<dyn Any>, _context: &mut ASTContext) -> Box<dyn TypeBase>
    where
        Self: Sized,
    {
        unimplemented!("new has not been implemented for this type");
    }
    fn print(&self, ast_context: &mut ASTContext);
    fn get_type(&self) -> BaseTypes;
    fn get_value(&self) -> LLVMValueRef;
    fn set_value(&mut self, _value: LLVMValueRef) {
        unimplemented!("{:?} type does not implement set_value", self.get_type())
    }
    fn get_ptr(&self) -> LLVMValueRef {
        unimplemented!(
            "get_ptr is not implemented for this type {:?}",
            self.get_type()
        )
    }
    fn set_ptr(&mut self, _value: LLVMValueRef) {
        unimplemented!("{:?} type does not implement set_ptr", self.get_type())
    }

    // TODO: make this a raw value
    fn get_str(&self) -> String {
        unimplemented!("{:?} type does not implement get_cstr", self.get_type())
    }
    fn get_length(&self) -> *mut usize {
        unimplemented!("{:?} type does not implement get_length", self.get_type())
    }
    fn add(&self, _ast_context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        unimplemented!("{:?} type does not implement add", self.get_type())
    }
    fn sub(&self, _context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        unimplemented!("{:?} type does not implement sub", self.get_type())
    }
    fn mul(&self, _context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        unimplemented!("{:?} type does not implement mul", self.get_type())
    }
    fn div(&self, _context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        unimplemented!("{:?} type does not implement div", self.get_type())
    }
    fn eqeq(&self, _context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        unimplemented!("{:?} type does not implement eqeq", self.get_type())
    }
    fn ne(&self, _context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        unimplemented!("{:?} type does not implement eqeq", self.get_type())
    }
    fn gt(&self, _context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        unimplemented!("{:?} type does not implement gt", self.get_type())
    }
    fn gte(&self, _context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        unimplemented!("{:?} type does not implement gte", self.get_type())
    }
    fn lt(&self, _context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        unimplemented!("{:?} type does not implement lt", self.get_type())
    }
    fn lte(&self, _context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        unimplemented!("{:?} type does not implement lte", self.get_type())
    }
}

dyn_clone::clone_trait_object!(TypeBase);

#[derive(Debug, Clone)]
pub struct StringType {
    llmv_value: LLVMValueRef,
    length: *mut usize,
    llmv_value_pointer: Option<LLVMValueRef>,
    str_value: String,
}

impl TypeBase for StringType {
    fn new(_value: Box<dyn Any>, _context: &mut ASTContext) -> Box<dyn TypeBase>
    where
        Self: Sized,
    {
        let value_as_string = match _value.downcast_ref::<String>() {
            Some(val) => val.to_string(),
            None => panic!("The input value must be a bool"),
        };
        let string = CString::new(value_as_string.clone()).unwrap();
        unsafe {
            let value = LLVMConstStringInContext(
                _context.context,
                string.as_ptr(),
                string.as_bytes().len() as u32,
                0,
            );
            let mut len_value: usize = string.as_bytes().len() as usize;
            let ptr: *mut usize = (&mut len_value) as *mut usize;
            let buffer_ptr = LLVMBuildPointerCast(
                _context.builder,
                value,
                LLVMPointerType(LLVMInt8Type(), 0),
                c_str!("buffer_ptr"),
            );
            return Box::new(StringType {
                length: ptr,
                llmv_value: value,
                llmv_value_pointer: Some(buffer_ptr),
                str_value: value_as_string, // fix
            });
        }
    }
    fn get_type(&self) -> BaseTypes {
        BaseTypes::String
    }
    fn get_value(&self) -> LLVMValueRef {
        self.llmv_value
    }
    fn get_length(&self) -> *mut usize {
        self.length
    }
    fn get_ptr(&self) -> LLVMValueRef {
        match self.llmv_value_pointer {
            Some(v) => {
                return v;
            }
            None => {
                unreachable!("No pointer for this value")
            }
        }
    }
    fn get_str(&self) -> String {
        self.str_value.clone()
    }

    fn add(&self, _ast_context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::String => match _ast_context.llvm_func_cache.get("sprintf") {
                Some(_sprintf_func) => unsafe {
                    // TODO: Use sprintf to concatenate two strings
                    // Remove extra quotes
                    let new_string = format!(
                        "{}{}",
                        self.get_str().to_string(),
                        _rhs.get_str().to_string()
                    )
                    .replace("\"", "");

                    let string = CString::new(new_string.clone()).unwrap();
                    let value = LLVMConstStringInContext(
                        _ast_context.context,
                        string.as_ptr(),
                        string.as_bytes().len() as u32,
                        0,
                    );
                    let mut len_value: usize = string.as_bytes().len() as usize;
                    let ptr: *mut usize = (&mut len_value) as *mut usize;
                    let buffer_ptr = LLVMBuildPointerCast(
                        _ast_context.builder,
                        value,
                        LLVMPointerType(LLVMInt8Type(), 0),
                        c_str!("buffer_ptr"),
                    );
                    return Box::new(StringType {
                        length: ptr,
                        llmv_value: value,
                        llmv_value_pointer: Some(buffer_ptr),
                        str_value: new_string,
                    });
                },
                _ => {
                    unreachable!()
                }
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
            // Set Value
            // create string vairables and then function
            // This is the Main Print Func
            let llvm_value_to_cstr = LLVMGetAsString(self.llmv_value, self.length);

            let value_is_str =
                LLVMBuildGlobalStringPtr(ast_context.builder, c_str!("%s\n"), c_str!(""));

            // Load Value from Value Index Ptr
            let val = LLVMBuildGlobalStringPtr(
                ast_context.builder,
                llvm_value_to_cstr,
                llvm_value_to_cstr,
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

#[derive(Debug, Clone)]
pub struct NumberType {
    //TODO: remove pub use of these
    pub llmv_value: LLVMValueRef,
    pub llmv_value_pointer: LLVMValueRef,
}

impl TypeBase for NumberType {
    fn new(_value: Box<dyn Any>, _context: &mut ASTContext) -> Box<dyn TypeBase> {
        let value_as_i32 = match _value.downcast_ref::<i32>() {
            Some(val) => *val,
            None => panic!("The input value must be an i32"),
        };
        unsafe {
            let value = LLVMConstInt(
                LLVMInt32TypeInContext(_context.context),
                value_as_i32.try_into().unwrap(),
                0,
            );
            let ptr = LLVMBuildAlloca(
                _context.builder,
                LLVMInt32TypeInContext(_context.context),
                c_str!("ptr"),
            );
            return Box::new(NumberType {
                llmv_value: value,
                llmv_value_pointer: ptr,
            });
        }
    }
    fn get_value(&self) -> LLVMValueRef {
        self.llmv_value
    }
    fn get_type(&self) -> BaseTypes {
        BaseTypes::Number
    }
    fn add(&self, _ast_context: &mut ASTContext, _rhs: Box<dyn TypeBase>) -> Box<dyn TypeBase> {
        match _rhs.get_type() {
            BaseTypes::Number => {
                unsafe {
                    let result = LLVMBuildAdd(
                        _ast_context.builder,
                        self.get_value(),
                        _rhs.get_value(),
                        c_str!("result"),
                    );
                    let ptr = LLVMBuildAlloca(_ast_context.builder, int32_type(), c_str!("result"));

                    // let result_str = LLVMBuildIntToPtr(builder, result, int8_ptr_type(), c_str!(""));
                    return Box::new(NumberType {
                        llmv_value: result,
                        llmv_value_pointer: ptr,
                    });
                }
            }
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
            BaseTypes::Number => {
                unsafe {
                    let result = LLVMBuildSub(
                        context.builder,
                        self.get_value(),
                        _rhs.get_value(),
                        c_str!("result"),
                    );
                    let ptr = LLVMBuildAlloca(context.builder, int32_type(), c_str!("result"));
                    // let result_str = LLVMBuildIntToPtr(builder, result, int8_ptr_type(), c_str!(""));
                    return Box::new(NumberType {
                        llmv_value: result,
                        llmv_value_pointer: ptr,
                    });
                }
            }
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
            BaseTypes::Number => {
                unsafe {
                    let result = LLVMBuildMul(
                        context.builder,
                        self.get_value(),
                        _rhs.get_value(),
                        c_str!("result"),
                    );
                    let ptr = LLVMBuildAlloca(context.builder, int32_type(), c_str!("result"));
                    // let result_str = LLVMBuildIntToPtr(builder, result, int8_ptr_type(), c_str!(""));
                    return Box::new(NumberType {
                        llmv_value: result,
                        llmv_value_pointer: ptr,
                    });
                }
            }
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
            BaseTypes::Number => {
                unsafe {
                    let result = LLVMBuildFDiv(
                        context.builder,
                        self.get_value(),
                        _rhs.get_value(),
                        c_str!("result"),
                    );
                    let ptr = LLVMBuildAlloca(context.builder, int32_type(), c_str!("result"));
                    // let result_str = LLVMBuildIntToPtr(builder, result, int8_ptr_type(), c_str!(""));
                    return Box::new(NumberType {
                        llmv_value: result,
                        llmv_value_pointer: ptr,
                    });
                }
            }
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
            let value_index_ptr =
                LLVMBuildAlloca(ast_context.builder, int32_type(), c_str!("value"));
            // First thing is to set initial value

            LLVMBuildStore(ast_context.builder, self.llmv_value, value_index_ptr);

            // Set Value
            // create string vairables and then function
            // This is the Main Print Func

            let value_is_str =
                LLVMBuildGlobalStringPtr(ast_context.builder, c_str!("%d\n"), c_str!(""));
            // Load Value from Value Index Ptr
            let val = LLVMBuildLoad2(
                ast_context.builder,
                int8_ptr_type(),
                value_index_ptr,
                c_str!("value"),
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

    return BoolType::new(Box::new(bool_value), _context);
}

#[derive(Debug, Clone)]
pub struct BoolType {
    pub builder: LLVMBuilderRef,
    value: bool,
    llmv_value: LLVMValueRef,
    llmv_value_pointer: LLVMValueRef,
}

impl TypeBase for BoolType {
    fn new(_value: Box<dyn Any>, _context: &mut ASTContext) -> Box<dyn TypeBase>
    where
        Self: Sized,
    {
        let value_as_bool = match _value.downcast_ref::<bool>() {
            Some(val) => *val,
            None => panic!("The input value must be a bool"),
        };
        unsafe {
            let mut num = 0;
            match value_as_bool {
                true => num = 1,
                _ => {}
            }
            let bool_value = LLVMConstInt(int1_type(), num, 0);
            let var_name = c_str!("bool_type");
            // Check if the global variable already exists
            let alloca = LLVMBuildAlloca(_context.builder, int1_type(), var_name);
            return Box::new(BoolType {
                builder: _context.builder,
                value: value_as_bool,
                llmv_value: bool_value,
                llmv_value_pointer: alloca,
            });
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
            match self.value {
                false => {
                    llvm_value_str = LLVMBuildGlobalStringPtr(
                        ast_context.builder,
                        c_str!("false"),
                        c_str!("false_str"),
                    );
                }
                _ => {}
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
    return BoolType::new(Box::new(bool_value), _context);
}

//TODO: create new functon
#[derive(Debug, Clone)]
pub struct BlockType {
    pub values: Vec<Expression>,
}

impl TypeBase for BlockType {
    fn get_value(&self) -> LLVMValueRef {
        unimplemented!("No value ref for block type")
    }
    fn get_type(&self) -> BaseTypes {
        BaseTypes::Block
    }
    fn print(&self, _ast_context: &mut ASTContext) {
        unreachable!("Shouldn't be able to print block type")
    }
}