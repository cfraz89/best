use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(item as ItemStruct);

    if let syn::Fields::Named(ref mut fields) = item_struct.fields {
        let context_field: syn::Field = syn::parse_quote! {
            #[serde(skip)]
            pub _context: elementary_rs_lib::context::Context
        };
        let selector_field: syn::Field = syn::parse_quote! {
            pub _selector: elementary_rs_lib::selector::Selector
        };
        fields.named.push(context_field);
        fields.named.push(selector_field);
    }

    let ident = item_struct.ident.clone();
    let ident_string = format!("component-{}", ident.to_string().to_ascii_lowercase());

    quote! {
        #[cfg_attr(target_arch = "wasm32", derive(serde::Deserialize))]
        #[cfg_attr(not(target_arch = "wasm32"), derive(serde::Serialize))]
        #item_struct

        impl elementary_rs_lib::context::ComponentContext for #ident {
            fn context(&self) -> &elementary_rs_lib::context::Context {
                &self._context
            }
        }

        impl elementary_rs_lib::node::ComponentTag for #ident {
            fn selector(&self) -> &elementary_rs_lib::selector::Selector {
                &self._selector
            }

            fn tag(&self) -> &'static str {
                #ident_string
            }
        }

        impl elementary_rs_lib::node::Component for #ident {}
    }
    .into()
}
