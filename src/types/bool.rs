use crate::types::llvm::*;
use std::any::Any;

use crate::context::ASTContext;
use crate::parser::Expression;
use dyn_clone::DynClone;
use std::os::raw::c_ulonglong;

extern crate llvm_sys;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::LLVMIntPredicate;
