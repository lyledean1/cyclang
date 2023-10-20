extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro]
pub fn generate_arithmetic_trait(input: TokenStream) -> TokenStream {
    // Parse the input into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Extract the struct or enum name
    let struct_name = &input.ident;

    // Generate code for each operation
    let add_impl = generate_arithmetic_operation(struct_name, "add");
    let sub_impl = generate_arithmetic_operation(struct_name, "sub");
    let mul_impl = generate_arithmetic_operation(struct_name, "mul");
    let div_impl = generate_arithmetic_operation(struct_name, "div");

    // Combine all generated code into a single TokenStream
    let expanded = quote! {
        // Trait implementation for Arithmetic
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

// Helper function to generate code for an arithmetic operation
fn generate_arithmetic_operation(struct_name: &syn::Ident, operation: &str) -> proc_macro2::TokenStream {
    let llvm_fn_ident = format!("LLVMBuild{}", operation.to_uppercase());
    let name = format!("\"{}\"", operation);

    quote! {
        fn #operation(
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
                            let rhs_value: *mut llvm_sys::LLVMValue = LLVMBuildLoad2(
                                context.builder,
                                self.get_llvm_type(),
                                _rhs.get_ptr().unwrap(),
                                _rhs.get_name(),
                            );
                            let result =
                                #llvm_fn_ident(context.builder, lhs_value, rhs_value, c_str!(#name));
                            LLVMBuildStore(context.builder, result, self.get_ptr().unwrap());
                            //TODO: fix the new instruction
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
                            let result = #llvm_fn_ident(
                                context.builder,
                                self.get_value(),
                                _rhs.get_value(),
                                c_str!(#name),
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
