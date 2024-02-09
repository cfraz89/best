use crate::{node::Component, selector::Selector};

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(target_arch = "wasm32")]
pub trait Page: Component + serde::de::DeserializeOwned {}
