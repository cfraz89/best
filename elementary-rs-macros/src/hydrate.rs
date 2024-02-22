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
            use bevy_ecs::prelude::*;
            let data = self.#hidden_ident(#hidden_args)#await_tokens;
            let mut world = elementary_rs_lib::world::WORLD.write().unwrap();
            //Self is a elementary component and bevy component
                    println!("Self: {:?}", self);
            for (mut entity, view, mut server_data) in world.query::<(Entity, &elementary_rs_lib::node::AnyView, Option<&mut elementary_rs_lib::server_data::ServerData>)>().iter_mut(&mut world) {
                // if std::ptr::eq(*view.0.as_ref() as *const _, *self as *const _) {
                    println!("Found server data");
                    if let Some(mut server_data) = server_data {
                        server_data.insert(#hidden_name.to_string(), serde_json::to_value(&data).unwrap());
                    } else {
                        let mut new_server_data = elementary_rs_lib::server_data::ServerData::default();
                        new_server_data.insert(#hidden_name.to_string(), serde_json::to_value(&data).unwrap());
                        world.entity_mut(entity).insert(new_server_data);
                    }
                    return data;
                // } 
            }
            panic!("Unable to store server data!");
        }

        //On the client, load the serialized data
        #[cfg(target_arch = "wasm32")]
        #sig {
            let mut world = elementary_rs_lib::world::WORLD.write().unwrap();
            //Self is a elementary component and bevy component
            for (page,  server_data) in world.query::<(&Self, &elementary_rs_lib::server_data::ServerData)>().iter(&mut world) {
                if std::ptr::eq(page, self) {
                    if let Some(data) = server_data.get(#hidden_name) {
                        return serde_json::from_value(data.clone()).expect("No server data to load!")
                    } else {
                        panic!("No server data")
                    }
                }
            }
            panic!("Did not find server data!");
        }
    }
    .into()
}
