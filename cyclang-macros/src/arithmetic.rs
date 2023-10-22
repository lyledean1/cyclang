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
                BaseTypes::Number => unsafe {
                    match self.get_ptr() {
                        Some(_p) => {
                            let lhs_value = LLVMBuildLoad2(
                                context.builder,
                                self.get_llvm_type(),
                                self.get_ptr().unwrap(),
                                self.get_name(),
                            );
                            let rhs_value = LLVMBuildLoad2(
                                context.builder,
                                _rhs.get_llvm_type(),
                                _rhs.get_ptr().unwrap(),
                                _rhs.get_name(),
                            );
                            let result =
                                #llvm_fn_name(context.builder, lhs_value, rhs_value, cstr_from_string(#add_name).as_ptr());
                            LLVMBuildStore(context.builder, result, self.get_ptr().unwrap());
                            let c_str_ref = CStr::from_ptr(self.get_name());
                            // Convert the CStr to a String (handles invalid UTF-8)
                            let name = c_str_ref.to_string_lossy().to_string();
                            Box::new(#struct_name {
                                name,
                                llmv_value: result,
                                llmv_value_pointer: self.get_ptr(),
                                cname: self.get_name(),
                            })
                        }
                        None => {
                            let result = #llvm_fn_name(
                                context.builder,
                                self.get_value(),
                                _rhs.get_value(),
                                cstr_from_string(#name).as_ptr(),
                            );
                            let alloca = LLVMBuildAlloca(
                                context.builder,
                                self.get_llvm_ptr_type(),
                                cstr_from_string("param_add").as_ptr(),
                            );
                            LLVMBuildStore(context.builder, result, alloca);
                            let c_str_ref = CStr::from_ptr(self.get_name());

                            // Convert the CStr to a String (handles invalid UTF-8)
                            let name = c_str_ref.to_string_lossy().to_string();
                            //TODO: fix the new instruction
                            Box::new(#struct_name {
                                name,
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
                        stringify!(#operation),
                        self.get_type(),
                        _rhs.get_type()
                    )
                }
            }
        }
    }
}
