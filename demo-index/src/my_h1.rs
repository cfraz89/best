use elementary_rs_lib::{
    node::{NodeRef, View},
    signal::Signal,
};
use elementary_rs_macros::{hydrate, view, Component};
use serde::{Deserialize, Serialize};

#[derive(Component, bevy_ecs::component::Component, Clone, Serialize, Deserialize, Debug)]
pub struct MyH1 {
    pub name: String,
}

impl View for MyH1 {
    async fn build(self) -> NodeRef {
        let title = self.my_title().await;
        view! {
            <div>
                <h1 style="color: red;">{title}</h1>
                <div>Hi {self.name}</div>
                <slot />
            </div>
        }
    }
}

impl MyH1 {
    #[hydrate]
    async fn my_title(&self) -> String {
        "Server title".to_string()
    }
}
