use crate::node::Node;
use std::future::Future;

#[cfg(not(target_arch = "wasm32"))]
pub trait Page: serde::Serialize {
    fn build(self) -> impl Future<Output = Node> + Send;

    fn wasm_path(&self) -> &'static str;
}

#[cfg(target_arch = "wasm32")]
pub trait Page: serde::de::DeserializeOwned {
    fn build(self) -> impl Future<Output = Node> + Send;
}

#[cfg(not(any(target_arch = "wasm32", feature = "web")))]
pub async fn render_page(page: impl Page) -> Result<String, serde_json::Error> {
    let wasm_path = page.wasm_path();
    let serial_page = serde_json::to_string(&page)?;
    let node = page.build().await;
    //Todo, macro out a serialise for pages and components, including server data
    // let serial_node = serde_json::to_string(&node)?;
    Ok(format!(
        "<!doctype html><html><head></head><body>{}<script type=\"module\">import start, {{ render }} from \"{wasm_path}\"; await start(); await render({serial_page});</script></body></html>",
         node.render().await.expect("Render didnt give any output!")
    ))
}
