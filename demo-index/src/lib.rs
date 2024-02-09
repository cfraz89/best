use elementary_rs_lib::node::Component;
use elementary_rs_lib::{
    node::{Node, ServerDataMap, View},
    page::Page,
};
use elementary_rs_macros::{component, node, server};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub async fn init_document() -> Result<(), JsValue> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    Ok(())
}

#[component]
pub struct IndexPage {
    pub x: i32,
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use gloo_utils::format::JsValueSerdeExt;
        #[wasm_bindgen]
        pub async fn render(serial_page: JsValue, serial_server_data_map: JsValue) -> Result<(), JsValue> {
            let page: IndexPage = serial_page.into_serde().expect("Could not deserialize initial value!");
            let server_data_map: ServerDataMap = serial_server_data_map.into_serde().expect("Could not deserialize initial value!");
            web_sys::console::log_1(&"Got here".into());
            page.reified_view(Some(&server_data_map)).await;
            page.bind();
            Ok(())
        }
    }
}

impl View for IndexPage {
    async fn build(&self) -> Node {
        let x = self.x;
        node!(
            <div>
                <MyH1>
                    {x * 10}
                </MyH1>
            </div>
        )
    }
}

impl Page for IndexPage {
    #[cfg(not(target_arch = "wasm32"))]
    fn wasm_path(&self) -> &'static str {
        "./wasm/demo_index.js"
    }
}
// #[derive(ComponentSupport)]
#[component]
pub struct MyH1 {}

// #[async_trait]
impl View for MyH1 {
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
