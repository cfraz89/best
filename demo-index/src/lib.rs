mod my_h1;

use std::sync::Arc;

use elementary_rs_lib::{
    node::{Node, NodeRef, View},
    page::Page,
    signal::Signal,
};
use elementary_rs_macros::{view, Component};
use my_h1::MyH1;
use serde::{Deserialize, Serialize};

#[derive(Component, bevy_ecs::component::Component, Serialize, Deserialize, Clone)]
#[page(js_path = "./wasm/demo_index.js")]
pub struct IndexPage {
    #[serde(skip)]
    pub x: Signal<i32>,
}

impl Page for IndexPage {}

impl View for IndexPage {
    async fn build(self: Arc<Self>) -> NodeRef {
        let s = self.as_ref().clone();
        view!(
            <div>
                <MyH1>
                    {s.x.get() * 10}
                    {s.x.get() * 20}
                </MyH1>
            </div>
        )
    }
}
