#![feature(proc_macro_span)]
#![feature(proc_macro_diagnostic)]
mod chimera;
mod node;

use proc_macro::TokenStream;

#[proc_macro]
pub fn chimera(input: TokenStream) -> TokenStream {
    chimera::chimera(input.into()).into()
}
