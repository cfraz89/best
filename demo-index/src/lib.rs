mod my_h1;

use elementary_rs_lib::{
    node::{Node, View},
    page::Page,
};
use elementary_rs_macros::{node, page};
use my_h1::MyH1;

#[page(js_path = "./wasm/demo_index.js")]
pub struct IndexPage {
    pub x: i32,
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
