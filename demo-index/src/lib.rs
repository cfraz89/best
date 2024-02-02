use std::future::Future;

use async_trait::async_trait;
use elementary_rs_lib::{
    node::{Component, Node},
    page::Page,
};
use elementary_rs_macros::{node, ComponentData};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub async fn init_document() -> Result<(), JsValue> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    Ok(())
}

#[cfg_attr(any(target = "wasm32", feature = "web"), derive(serde::Deserialize))]
#[cfg_attr(not(any(target = "wasm32", feature = "web")), derive(serde::Serialize))]
pub struct IndexPage {
    pub x: i32,
}

cfg_if::cfg_if! {
    if #[cfg(any(target = "wasm32", feature = "web"))] {

use gloo_utils::format::JsValueSerdeExt;
#[wasm_bindgen]
pub async fn render(value: JsValue) -> Result<(), JsValue> {
    let page: IndexPage = value.into_serde().unwrap();
    let node = page.node().await;
    node.bind()
}
    }
}

impl Page for IndexPage {
    async fn node(self) -> Node {
        node!(
            <div>
                <MyH1>
                    {self.x * 10}
                </MyH1>
            </div>
        )
    }

    #[cfg(not(any(target = "wasm32", feature = "web")))]
    fn wasm_path(&self) -> &'static str {
        "./wasm/demo_index.js"
    }
}
#[derive(ComponentData)]
pub struct MyH1 {}

#[async_trait]
impl Component for MyH1 {
    async fn view(&self) -> Node {
        node! {
            <h1>
            <slot />
            </h1>
        }
    }
}
