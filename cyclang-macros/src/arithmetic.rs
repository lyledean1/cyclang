use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

pub fn generate_arithmetic_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let add_impl = generate_arithmetic_operation(struct_name, "LLVMBuildAdd", "add");
    let sub_impl = generate_arithmetic_operation(struct_name, "LLVMBuildSub", "sub");
    let mul_impl = generate_arithmetic_operation(struct_name, "LLVMBuildMul", "mul");
    let div_impl = generate_arithmetic_operation(struct_name, "LLVMBuildSDiv", "div");

    let imports = quote! {
        use std::ffi::CStr;
    };
    let expanded = quote! {
        #imports
        impl Arithmetic for #struct_name {
            #add_impl
            #sub_impl
            #mul_impl
            #div_impl
        }
    };
    // Return the generated code as a TokenStream
    TokenStream::from(expanded)
}

fn generate_arithmetic_operation(
    struct_name: &syn::Ident,
    llvm_fn_name_str: &str,
    operation: &str,
) -> proc_macro2::TokenStream {
    let name = operation.to_string();
    let fn_name = Ident::new(operation, proc_macro2::Span::call_site());
    let llvm_fn_name = Ident::new(llvm_fn_name_str, proc_macro2::Span::call_site());
    let add_name = format!("add{}", struct_name);

    quote! {
        fn #fn_name(
            &self,
            context: &mut ASTContext,
            _rhs: Box<dyn TypeBase>,
        ) -> Box<dyn TypeBase> {
            match _rhs.get_type() {
                BaseTypes::Number | BaseTypes::Number64 => unsafe {
                    match (self.get_ptr(), _rhs.get_ptr()) {
                        (Some(ptr), Some(rhs_ptr)) => {
                            let mut lhs_val = context.codegen.build_load(
                                ptr,
                                self.get_llvm_type(),
                                "rhs", //todo: fix with function to get name
                            );
                            let mut rhs_val = context.codegen.build_load(
                                rhs_ptr,
                                _rhs.get_llvm_type(),
                                "lhs", //todo: fix with function to get name
                            );

                            // convert to i64 if mismatched types
                            lhs_val = context.codegen.cast_i32_to_i64(lhs_val, rhs_val);
                            rhs_val = context.codegen.cast_i32_to_i64(rhs_val, lhs_val);

                            let result =
                                #llvm_fn_name(context.codegen.builder, lhs_val, rhs_val, cstr_from_string(#add_name).as_ptr());
                            let alloca = context.codegen.build_alloca_store(result, self.get_llvm_ptr_type(), "param_add");
                            let name = self.get_name_as_str().to_string();
                            Box::new(#struct_name {
                                name,
                                llvm_value: result,
                                llvm_value_pointer: Some(alloca),
                            })
                        }
                        _ => {
                            let mut lhs_val = self.get_value();
                            let mut rhs_val = _rhs.get_value();
                            // convert to i64 if mismatched types
                            lhs_val = context.codegen.cast_i32_to_i64(lhs_val, rhs_val);
                            rhs_val = context.codegen.cast_i32_to_i64(rhs_val, lhs_val);

                            let result = #llvm_fn_name(
                                context.codegen.builder,
                                lhs_val,
                                rhs_val,
                                cstr_from_string(#name).as_ptr(),
                            );
                            let alloca = context.codegen.build_alloca_store(result, self.get_llvm_ptr_type(), "param_add");
                            let name = self.get_name_as_str().to_string();

                            //TODO: fix the new instruction
                            Box::new(#struct_name {
                                name,
                                llvm_value: result,
                                llvm_value_pointer: Some(alloca),
                            })
                        }
                    }
                },
                _ => {
                    unreachable!(
                        "Can't {} type {:?} and type {:?}",
                        stringify!(#operation),
                        self.get_type(),
                        _rhs.get_type()
                    )
                }
            }
        }
    }
}
