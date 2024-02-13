mod my_h1;

use elementary_rs_lib::{
    node::{Node, View},
    page::Page,
};
use elementary_rs_macros::{node, page};
use my_h1::MyH1;

#[page]
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

impl Page for IndexPage {
    #[cfg(not(target_arch = "wasm32"))]
    fn wasm_path(&self) -> &'static str {
        "./demo-index/pkg/demo_index.js"
    }
}
