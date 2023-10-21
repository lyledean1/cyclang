use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub fn generate_debug_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let expanded = quote! {
        impl Debug for #struct_name {
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
                                vec![ast_context.get_printf_str(self.get_type()), value];
                            match ast_context.llvm_func_cache.get("printf") {
                                Some(print_func) => {
                                    LLVMBuildCall2(
                                        ast_context.builder,
                                        print_func.func_type,
                                        print_func.function,
                                        print_args.as_mut_ptr(),
                                        2,
                                        cstr_from_string(""),
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
                                    cstr_from_string(""),
                                );
                            },
                            _ => {
                                unreachable!()
                            }
                        },
                    }
                }
        }
    };

    TokenStream::from(expanded)
}
