use crate::node::Node;
use std::future::Future;

#[cfg(not(any(target = "wasm32", feature = "web")))]
pub trait Page: serde::Serialize {
    fn node(self) -> impl Future<Output = Node> + Send;

    fn wasm_path(&self) -> &'static str;
}

#[cfg(any(target = "wasm32", feature = "web"))]
pub trait Page: serde_lite::Deserialize {
    fn node(self) -> impl Future<Output = Node> + Send;
}

#[cfg(not(any(target_arch = "wasm32", feature = "web")))]
pub async fn render_page(page: impl Page) -> Result<String, serde_json::Error> {
    Ok(format!(
        "<!doctype html><html><head><script type=\"module\">import start, {{ render }} from \"{}\"; await start(); await render({});</script></head><body>{}</body></html>",
         page.wasm_path(), serde_json::to_string(&page)?, page.node().await.render().await.unwrap()
    ))
}
