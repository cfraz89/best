use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, ItemStruct};

pub fn page(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(item as ItemStruct);

    let ident = item_struct.ident.clone();
    let lower_ident = ident.to_string().to_ascii_lowercase();
    let hydrate_ident = Ident::new(&format!("hydrate_{lower_ident}"), ident.span());
    let context_ident = Ident::new(&format!("_context_{ident}").to_uppercase(), ident.span());
    let selector_ident = Ident::new(&format!("_selector_{ident}").to_uppercase(), ident.span());
    let ident_string = format!("page-{lower_ident}",);

    quote! {
        #[cfg_attr(target_arch = "wasm32", derive(serde::Deserialize))]
        #[cfg_attr(not(target_arch = "wasm32"), derive(serde::Serialize, serde::Deserialize))]
        #item_struct

        static #context_ident: std::sync::OnceLock<elementary_rs_lib::context::Context> = std::sync::OnceLock::new();

        impl elementary_rs_lib::context::ComponentContext for #ident {
            fn context(&self) -> &elementary_rs_lib::context::Context {
              &#context_ident.get_or_init(|| Default::default())
            }
        }

        static #selector_ident: std::sync::OnceLock<elementary_rs_lib::selector::Selector> = std::sync::OnceLock::new();

        impl elementary_rs_lib::node::ComponentTag for #ident {
            fn selector(&self) -> &elementary_rs_lib::selector::Selector {
                &#selector_ident.get_or_init(|| elementary_rs_lib::selector::Selector::Id(#ident_string.to_string()))
            }

            fn tag(&self) -> &'static str {
                #ident_string
            }
        }

        impl elementary_rs_lib::node::Component for #ident {}

        #[cfg(target_arch = "wasm32")]
        #[wasm_bindgen::prelude::wasm_bindgen]
        pub async fn #hydrate_ident(serial_page: wasm_bindgen::JsValue, serial_server_data_map: wasm_bindgen::JsValue) -> Result<(), wasm_bindgen::JsValue> {
          elementary_rs_lib::page::hydrate::<#ident>(serial_page, serial_server_data_map).await
        }
      }
      .into()
}
