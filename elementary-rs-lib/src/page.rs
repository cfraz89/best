use crate::node::Node;
use std::future::Future;

#[cfg(not(target_arch = "wasm32"))]
pub trait Page: serde::Serialize {
    fn node(self) -> impl Future<Output = Node> + Send;

    fn wasm_path(&self) -> &'static str;
}

#[cfg(target_arch = "wasm32")]
pub trait Page: serde::de::DeserializeOwned {
    fn node(self) -> impl Future<Output = Node> + Send;
}

#[cfg(not(any(target_arch = "wasm32", feature = "web")))]
pub async fn render_page(page: impl Page) -> Result<String, serde_json::Error> {
    let wasm_path = page.wasm_path();
    let serial_page = serde_json::to_string(&page)?;
    Ok(format!(
        "<!doctype html><html><head></head><body>{}<script type=\"module\">import start, {{ render }} from \"{wasm_path}\"; await start(); await render({serial_page});</script></body></html>",
         page.node().await.render().await.expect("Render didnt give any output!")
    ))
}
