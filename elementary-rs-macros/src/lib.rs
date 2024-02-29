#![feature(proc_macro_span)]
#![feature(proc_macro_diagnostic)]
mod ecn;
mod entity_node;

use proc_macro::TokenStream;

#[proc_macro]
pub fn ecn(input: TokenStream) -> TokenStream {
    ecn::ecn(input.into()).into()
}
