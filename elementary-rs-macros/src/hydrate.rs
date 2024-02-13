use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, ImplItemFn};

pub fn hydrate(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut item_fn = parse_macro_input!(item as ImplItemFn);
    let ident = item_fn.clone().sig.ident;
    let hidden_name = format!("_server_{}", ident);
    let hidden_ident = Ident::new(hidden_name.as_str(), ident.span());
    let sig = item_fn.sig.clone();
    item_fn.sig.ident = hidden_ident.clone();
    let hidden_args = item_fn
        .sig
        .inputs
        .clone()
        .into_iter()
        .skip(1)
        .collect::<Punctuated<syn::FnArg, syn::token::Comma>>();
    let await_tokens = match item_fn.sig.asyncness {
        Some(_) => quote! { .await },
        None => proc_macro2::TokenStream::new(),
    };
    quote! {
        #item_fn

        #[cfg(not(target_arch = "wasm32"))]
        #sig {
            let data = self.#hidden_ident(#hidden_args)#await_tokens;
            let mut server_data = self._context.server_data.lock().unwrap();
            server_data.insert(#hidden_name.to_string(), serde_json::to_value(&data).unwrap());
            data
        }

        //On the client, load the serialized data
        #[cfg(target_arch = "wasm32")]
        #sig {
            web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(&format!("{:?}", self._context)));
            if let Some(data) = self._context.server_data.lock().unwrap().get(#hidden_name) {
                serde_json::from_value(data.clone()).expect("No server data to load!")
            } else {
                panic!("No server data")
            }
        }
    }
    .into()
}
