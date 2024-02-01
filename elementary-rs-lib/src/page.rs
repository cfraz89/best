use crate::node::Node;
use serde::{Deserialize, Serialize};
use std::future::Future;

pub trait Page: Serialize + Deserialize<'static> {
    fn node(self) -> impl Future<Output = Node> + Send;

    fn wasm_path(&self) -> &'static str;
}

pub async fn render_page(page: impl Page) -> Result<String, serde_json::Error> {
    Ok(format!(
        "<!doctype html><html><head><script type=\"module\">import start, {{ render }} from \"{}\"; await start(); await render({});</script></head><body>{}</body></html>",
         page.wasm_path(), serde_json::to_value(&page)?, page.node().await
    ))
}
