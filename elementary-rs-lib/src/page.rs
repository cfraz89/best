use bevy_ecs::entity::Entity;

use crate::{
    component::Component,
    hydration_fn_name::{self, HydrationFnName},
    js_path::{self, JSPath},
    node::{construct_entity_view, AnyView, Node, View},
    server_data::{SerialServerData, ServerData},
    world::WORLD,
};

cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        use crate::selector::Selector;

        pub trait Page: Component {
            async fn render(self) -> Result<String, serde_json::Error> {
                let serial_page = serde_json::to_string(&self)?;
                let world = WORLD.read().unwrap();
                //written out by macro
                let entity = self.build_entity();
                let entity_ref = world.entity(entity);
                let JSPath(js_path) = entity_ref.get().expect("Entity needs a js path");
                let HydrationFnName(hydration_fn_name) = entity_ref.get().expect("Entity needs a js path");
                let selector = entity_ref.get().expect("Entity needs a selector");
                construct_entity_view(&entity, None).await.expect("failed constructing view");
                let rendered_node = entity_ref.get::<Node>().expect("page has no node").render().await.expect("Render didnt give any output!");
                let SerialServerData(server_data) = ServerData::get_serial_server_data(&entity);
                let server_data = serde_json::to_string(&server_data)?;
                let selector_attr = match selector {
                                        Selector::Id(id) => format!("id=\"_{id}\""),
                                        Selector::Class(class) => format!("class=\"_{class}\""),
                                    };
                Ok(format!(
                    "<!doctype html><html><head></head><body {selector_attr}>{rendered_node}<script type=\"module\">import start, {{ {hydration_fn_name} }} from \"{js_path}\"; await start(); await {hydration_fn_name}({serial_page}, {server_data});</script></body></html>",

                ))
            }
        }

    } else {
    pub trait Page: Component + serde::de::DeserializeOwned {}

        use crate::node::{ServerDataMap};
        use gloo_utils::format::JsValueSerdeExt;
        use serde::Deserialize;
        use wasm_bindgen::JsValue;

        pub async fn hydrate<T: Component + for<'a> Deserialize<'a>>(
            serial_page: JsValue,
            serial_server_data_map: JsValue,
        ) -> Result<(), JsValue> {
            let page: T = serial_page
                .into_serde()
                .expect("Could not deserialize initial value!");
            let server_data_map: ServerDataMap = serial_server_data_map
                .into_serde()
                .expect("Could not deserialize initial value!");
            page.reified_view(Some(&server_data_map)).await?;
            page.bind()?;
            Ok(())
        }
    }
}
