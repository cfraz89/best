mod my_h1;

use elementary_rs_lib::{
    node::{Node, NodeRef, View},
    page::Page,
};
use elementary_rs_macros::{view, Component};
use my_h1::MyH1;
use serde::{Deserialize, Serialize};

#[derive(Component, bevy_ecs::component::Component, Serialize, Deserialize)]
#[page(js_path = "./wasm/demo_index.js")]
pub struct IndexPage {
    pub x: i32,
}

impl Page for IndexPage {}

impl View for IndexPage {
    async fn build(&self) -> NodeRef {
        let x = self.x;
        view!(
            <div>
                <MyH1>
                    {x * 10}
                    {x * 20}
                </MyH1>
            </div>
        )
    }
}
