mod my_h1;

use elementary_rs_lib::{
    node::{Node, View},
    page::Page,
    signal::Signal,
};
use elementary_rs_macros::{node, page};
use my_h1::MyH1;

#[page(js_path = "./wasm/demo_index.js")]
pub struct IndexPage {
    pub x: i32,
}

impl View for IndexPage {
    async fn build(&self) -> Entity {
        view!(
            <div>
                <MyH1>
                    // {*self.x.get() * 10}
                </MyH1>
            </div>
        )
    }
}
