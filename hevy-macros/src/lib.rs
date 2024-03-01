#![feature(proc_macro_span)]
#![feature(proc_macro_diagnostic)]
mod entity_node;
mod hevy;

use proc_macro::TokenStream;

#[proc_macro]
pub fn hevy(input: TokenStream) -> TokenStream {
    hevy::hevy(input.into()).into()
}
