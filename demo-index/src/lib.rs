use elementary_rs_lib::node::{Component, Node};
use elementary_rs_macros::{node, CustomElement};
use wasm_bindgen::prelude::*;

// basic handler that responds with a static string
pub fn root_node() -> Node {
    let x = 10;
    my_node(x)
}

#[wasm_bindgen(start)]
pub async fn init_document() -> Result<(), JsValue> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    root_node().bind()
}

fn my_node(x: i32) -> Node {
    node!(
        <div>
            <MyH1>
                {x * 10}
            </MyH1>
        </div>
    )
}

#[derive(CustomElement)]
struct MyH1 {}

impl Component for MyH1 {
    fn node(&self) -> Node {
        node! {
            <h1>
            <slot />
            </h1>
        }
    }
}
