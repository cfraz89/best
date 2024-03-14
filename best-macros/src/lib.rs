#![feature(proc_macro_span)]
#![feature(proc_macro_diagnostic)]
mod best;
mod node;

use proc_macro::TokenStream;

#[proc_macro]
pub fn best(input: TokenStream) -> TokenStream {
    best::best(input.into()).into()
}
