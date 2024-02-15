use elementary_rs_lib::node::{Node, View};
use elementary_rs_macros::{hydrate, view, Component};
use serde::{Deserialize, Serialize};

#[derive(Component, bevy_ecs::component::Component, Serialize, Deserialize)]
pub struct MyH1 {}

// #[async_trait]
impl View for MyH1 {
    async fn build(&self) -> Node {
        let title = self.my_title().await;
        view! {
            <div>
                <h1>{title}</h1>
                <div>Hi</div>
                <slot />
            </div>
        }
    }
}

impl MyH1 {
    // #[hydrate]
    async fn my_title(&self) -> String {
        "Server title".to_string()
    }
}
