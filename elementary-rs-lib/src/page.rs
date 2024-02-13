use crate::node::Component;

cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        use crate::selector::Selector;

        pub trait Page: Component + serde::Serialize + Sized {
            fn wasm_path(&self) -> &'static str;

            async fn render(&self) -> Result<String, serde_json::Error> {
                let wasm_path = self.wasm_path();
                let rendered_node = self.reified_view(None).await.unwrap().render().await.expect("Render didnt give any output!");
                let serial_page = serde_json::to_string(self)?;
                let server_data = serde_json::to_string(&self.serialize_server_data())?;
                let selector = match self.selector() {
                                        Selector::Id(id) => format!("id=\"_{id}\""),
                                        Selector::Class(class) => format!("class=\"_{class}\""),
                                    };
                Ok(format!(
                    "<!doctype html><html><head></head><body {selector}>{rendered_node}<script type=\"module\">import start, {{ render }} from \"{wasm_path}\"; await start(); await render({serial_page}, {server_data});</script></body></html>",

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
