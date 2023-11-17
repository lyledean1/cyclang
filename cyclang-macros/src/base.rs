use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse::ParseStream, Path};
use syn::{parse_macro_input, Ident, LitStr};

struct MacroInput {
    struct_name: Ident,
    base_type: LitStr,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let derived_item: syn::DeriveInput = input.parse()?;
        let struct_name = derived_item.ident;

        // Look for the `base_type` attribute
        let base_type = derived_item
            .attrs
            .iter()
            .find_map(|attr| {
                if attr.path().is_ident("base_type") {
                    match attr.parse_args() {
                        Ok(lit) => Some(lit),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            })
            .expect("Expected a `base_type` attribute!");

        Ok(MacroInput {
            struct_name,
            base_type,
        })
    }
}

pub fn generate_base_derive(input: TokenStream) -> TokenStream {
    let MacroInput {
        struct_name,
        base_type,
    } = parse_macro_input!(input as MacroInput);

    let base_type_path: Path = syn::parse_str(&base_type.value()).expect("Invalid base type");

    let generated_code = quote! {
        impl Base for #struct_name {
            fn get_type(&self) -> BaseTypes {
                #base_type_path
            }
        }
    };

    generated_code.into()
}
