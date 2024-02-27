// mod my_h1;

use bevy::prelude::*;
use bevy_ecs::system::EntityCommands;
use elementary_rs_lib::components::{BuildWebComponent, Page, WebComponent};
use elementary_rs_lib::node::NodeRef;
use elementary_rs_macros::{view, BuildComponent};
use serde::{Deserialize, Serialize};

//Called to setup the page
pub fn setup_page(world: &mut World) {
    let page = Index { x: 20 }.build_entity(world, vec![]);
    world.entity_mut(page).insert(Page);
}

#[derive(Component, BuildComponent, Clone, Serialize, Deserialize)]
#[page(js_path = "./wasm/demo_index.js")]
pub struct Index {
    pub x: i32,
}

impl WebComponent for Index {
    async fn template(self, world: &mut World) -> NodeRef {
        view!(world,
            <div>
                <MyH1 title="Hello">
                    <slot />
                </MyH1>
                {self.x * 10}
            </div>
        )
    }
}

#[derive(Component, BuildComponent, Debug, Clone)]
pub struct MyH1 {
    pub title: String,
}

impl WebComponent for MyH1 {
    async fn template(self, world: &mut World) -> NodeRef {
        view!(world,
            <div>
                <h1 style="color: red;">{self.title}</h1>
                <slot />
            </div>
        )
    }
}
