use elementary_rs_lib::{
    node::{Component, Node},
    page::Page,
};
use elementary_rs_macros::{node, CustomElement};
use gloo_utils::format::JsValueSerdeExt;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub async fn init_document() -> Result<(), JsValue> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct IndexPage {
    pub x: i32,
}

#[wasm_bindgen]
pub async fn render(value: JsValue) -> Result<(), JsValue> {
    let page: IndexPage = value.into_serde().unwrap();
    let node = page.node().await;
    node.bind()
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

    fn wasm_path(&self) -> &'static str {
        "./wasm/demo_index.js"
    }
}
#[derive(CustomElement)]
pub struct MyH1 {}

impl Component for MyH1 {
    fn node(&self) -> Node {
        node! {
            <h1>
            <slot />
            </h1>
        }
    }
}
