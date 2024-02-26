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

#[proc_macro_derive(BuildComponent, attributes(page))]
pub fn build_component(input: TokenStream) -> TokenStream {
    component::build_component(input)
}

// #[proc_macro_attribute]
// pub fn page(attr: TokenStream, item: TokenStream) -> TokenStream {
//     page::page(attr, item)
// }

#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    view::view(input)
}
