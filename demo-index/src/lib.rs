use async_trait::async_trait;
use elementary_rs_lib::{
    node::{Component, Node},
    page::Page,
};
use elementary_rs_macros::{component, node, server, ComponentSupport};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub async fn init_document() -> Result<(), JsValue> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    Ok(())
}

#[cfg_attr(target_arch = "wasm32", derive(serde::Deserialize))]
#[cfg_attr(not(target_arch = "wasm32"), derive(serde::Serialize))]
pub struct IndexPage {
    pub x: i32,
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use gloo_utils::format::JsValueSerdeExt;
        #[wasm_bindgen]
        pub async fn render(value: JsValue) -> Result<(), JsValue> {
            let page: IndexPage = value.into_serde().expect("Could not deserialize initial value!");
            let node = page.build().await;
            node.bind()
        }
    }
}

impl Page for IndexPage {
    async fn build(self) -> Node {
        node!(
            <div>
                <MyH1>
                    {self.x * 10}
                </MyH1>
            </div>
        )
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn wasm_path(&self) -> &'static str {
        "./wasm/demo_index.js"
    }
}
#[derive(ComponentSupport)]
#[component]
pub struct MyH1 {}

// #[async_trait]
impl Component for MyH1 {
    async fn build(&self) -> Node {
        let title = self.my_title().await;
        node! {
            <div>
                <h1>{title}</h1>
                <div>Hi</div>
                <slot />
            </div>
        }
    }
}

impl MyH1 {
    #[server]
    async fn my_title(&self) -> String {
        "Server title".to_string()
    }
}
