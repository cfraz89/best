#![feature(proc_macro_span)]
#![feature(proc_macro_diagnostic)]
mod component;
mod hydrate;
mod node;
mod page;
mod template_node;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn hydrate(attr: TokenStream, item: TokenStream) -> TokenStream {
    hydrate::hydrate(attr, item)
}

#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    component::component(attr, item)
}

#[proc_macro_attribute]
pub fn page(attr: TokenStream, item: TokenStream) -> TokenStream {
    page::page(attr, item)
}

#[proc_macro]
pub fn node(input: TokenStream) -> TokenStream {
    node::node(input)
}
