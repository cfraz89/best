use bevy_ecs::entity::Entity;

use crate::{
    component::Component,
    hydration_fn_name::{self, HydrationFnName},
    js_path::{self, JSPath},
    node::{construct_entity_view, AnyView, Node, NodeRef, View},
    server_data::{SerialServerData, ServerData},
    world::WORLD,
};

cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        use crate::selector::Selector;

        pub trait Page: Component {
            async fn render(self) -> Result<String, serde_json::Error> {
                let serial_page = serde_json::to_string(&self)?;
                //written out by macro
                let entity = self.build_entity();
                let hydration_fn_name: Option<HydrationFnName>;
                let js_path: Option<JSPath>;
                {
                    let world = WORLD.read().unwrap();
                    let entity_ref = world.entity(entity);
                    js_path = entity_ref.get::<JSPath>().cloned();
                    hydration_fn_name = entity_ref.get::<HydrationFnName>().cloned();
                }
                construct_entity_view(&entity, None).await.expect("failed constructing view");
                {
                    let world = WORLD.read().unwrap();
                    let entity_ref = world.entity(entity);

                    let rendered_node = entity_ref.get::<NodeRef>().expect("page has no node").render().expect("Render didnt give any output!");
                    let selector = entity_ref.get::<Selector>().to_owned().expect("Entity needs a selector");
                    let selector_attr = match selector {
                                            Selector::Id(id) => format!("id=\"_{id}\""),
                                            Selector::Class(class) => format!("class=\"_{class}\""),
                                        };
                    let script = match (js_path, hydration_fn_name) {
                        (Some(JSPath(js_path)), Some(HydrationFnName(hydration_fn_name))) => {
                            let serial_server_data = ServerData::get_serial_server_data(&entity);
                            let server_data_string = serde_json::to_string(&serial_server_data)?;
                            Ok(format!("<script type=\"module\">import start, {{ {hydration_fn_name} }} from \"{js_path}\"; await start(); await {hydration_fn_name}({serial_page}, {server_data_string});</script>"))
                        }
                        _ => Ok("".to_string())
                    }?;
                    Ok(format!(
                        "<!doctype html><html><head></head><body {selector_attr}>{rendered_node}{script}</body></html>",

                    ))
                }
            }
        }

    } else {
        pub trait Page: Component {

            // use gloo_utils::format::JsValueSerdeExt;
            // use serde::Deserialize;
            // use wasm_bindgen::JsValue;

            // pub async fn hydrate<T: Component + for<'a> Deserialize<'a>>(
            //     serial_page: JsValue,
            //     serial_server_data: JsValue,
            // ) -> Result<(), JsValue> {
                // let page: T = serial_page
                //     .into_serde()
                //     .expect("Could not deserialize initial value!");
                // let serial_server_data: SerialServerData = serial_server_data
                //     .into_serde()
                //     .expect("Could not deserialize server data!");
                // construct_entity_view(entity, serial_server_data).await?;
                // page.bind()?;
                // Ok(())
            // }
        }
    }
}
