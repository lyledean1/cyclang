use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

pub fn generate_comparison_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let eqeq_impl = generate_comparison_operation("LLVMIntEQ", "eqeq");
    let ne_impl = generate_comparison_operation("LLVMIntNE", "ne");
    let gt_impl = generate_comparison_operation("LLVMIntSGT", "gt");
    let gte_impl = generate_comparison_operation("LLVMIntSGE", "gte");
    let lt_impl = generate_comparison_operation("LLVMIntSLT", "lt");
    let lte_impl = generate_comparison_operation("LLVMIntSLE", "lte");

    let import_bool_type = if *struct_name != "BoolType" {
        quote! {
            use crate::compiler::BoolType;
        }
    } else {
        quote! {}
    };

    let imports = quote! {
        use llvm_sys::LLVMIntPredicate::{
            LLVMIntEQ, LLVMIntNE, LLVMIntSGE, LLVMIntSGT, LLVMIntSLE, LLVMIntSLT,
        };
        #import_bool_type
    };

    let expanded = quote! {
        #imports
        impl Comparison for #struct_name {
            #eqeq_impl
            #ne_impl
            #gt_impl
            #gte_impl
            #lt_impl
            #lte_impl
        }
    };

    TokenStream::from(expanded)
}

fn generate_comparison_operation(
    llvm_predicate_str: &str,
    operation: &str,
) -> proc_macro2::TokenStream {
    let fn_name = Ident::new(operation, proc_macro2::Span::call_site());
    let llvm_predicate_name = Ident::new(llvm_predicate_str, proc_macro2::Span::call_site());

    quote! {
        fn #fn_name(
            &self,
            context: &mut ASTContext,
            rhs: Box<dyn TypeBase>,
        ) -> Box<dyn TypeBase> {
            match rhs.get_type() {
                BaseTypes::Number | BaseTypes::Bool => {
                }
                _ => {
                    unreachable!(
                        "Can't do operation type {:?} and type {:?}",
                        self.get_type(),
                        rhs.get_type()
                    )
                }
            }
            unsafe {
                // then do comparison
                 match (self.get_ptr(), self.get_type()) {
                        (Some(lhs_ptr), BaseTypes::Number) => {
                                // If loading a pointer
                                let mut lhs_val = context.build_load(
                                    lhs_ptr,
                                    self.get_llvm_type(),
                                    cstr_from_string("lhs_bool").as_ptr(),
                                );
                                let mut rhs_val = context.build_load(
                                    rhs.get_ptr().unwrap(),
                                    rhs.get_llvm_type(),
                                    cstr_from_string("rhs_bool").as_ptr(),
                                );

                                lhs_val = context.cast_i32_to_i64(lhs_val, rhs_val);
                                rhs_val = context.cast_i32_to_i64(rhs_val, lhs_val);

                                let cmp = LLVMBuildICmp(
                                    context.builder,
                                    #llvm_predicate_name,
                                    lhs_val,
                                    rhs_val,
                                    cstr_from_string("result").as_ptr(),
                                );

                                let alloca = context.build_alloca_store(cmp, int1_type(), cstr_from_string("bool_cmp").as_ptr());
                                Box::new(BoolType {
                                    name: self.name.clone(),
                                    builder: context.builder,
                                    llvm_value: cmp,
                                    llvm_value_pointer: alloca,
                                })
                            }
                            _ => {
                                let mut lhs_val = self.get_value();
                                let mut rhs_val = rhs.get_value();

                                lhs_val = context.cast_i32_to_i64(lhs_val, rhs_val);
                                rhs_val = context.cast_i32_to_i64(rhs_val, lhs_val);

                                let cmp = LLVMBuildICmp(
                                    context.builder,
                                    #llvm_predicate_name,
                                    lhs_val,
                                    rhs_val,
                                    cstr_from_string("result").as_ptr(),
                                );
                                let alloca = context.build_alloca_store(cmp, int1_type(), cstr_from_string("bool_cmp").as_ptr());
                                Box::new(BoolType {
                                    name: self.name.clone(),
                                    builder: context.builder,
                                    llvm_value: cmp,
                                    llvm_value_pointer: alloca,
                                })
                            }
                        }
            }
        }
    }
}
