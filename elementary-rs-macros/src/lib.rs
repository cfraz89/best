#![feature(proc_macro_span)]
#![feature(proc_macro_diagnostic)]
mod component;
mod hydrate;
// mod page;
mod template_node;
mod view;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn hydrate(attr: TokenStream, item: TokenStream) -> TokenStream {
    hydrate::hydrate(attr, item)
}

#[proc_macro_derive(Component, attributes(page))]
pub fn component(input: TokenStream) -> TokenStream {
    component::component(input)
}

// #[proc_macro_attribute]
// pub fn page(attr: TokenStream, item: TokenStream) -> TokenStream {
//     page::page(attr, item)
// }

#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    view::view(input)
}
