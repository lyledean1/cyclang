extern crate proc_macro;
mod arithmetic;
mod base;
mod comparison;

use crate::arithmetic::generate_arithmetic_derive;
use crate::base::generate_base_derive;
use crate::comparison::generate_comparison_derive;
use proc_macro::TokenStream;

#[proc_macro_derive(ArithmeticMacro)]
pub fn export_arithmetic_derive(input: TokenStream) -> TokenStream {
    generate_arithmetic_derive(input)
}

#[proc_macro_derive(ComparisonMacro)]
pub fn export_comparison_derive(input: TokenStream) -> TokenStream {
    generate_comparison_derive(input)
}

#[proc_macro_derive(BaseMacro, attributes(base_type))]
pub fn export_base_derive(input: TokenStream) -> TokenStream {
    generate_base_derive(input)
}
