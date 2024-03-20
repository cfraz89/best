#![feature(async_closure)]
use axum::{response::IntoResponse, routing::get, Router};
use chimera_rs::axum_html::AxumHtmlApp;
use chimera_rs::r#async::WorldCallback;
use tower_http::services::ServeDir;

use bevy::prelude::*;
use chimera_rs::prelude::*;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .nest_service("/wasm", ServeDir::new("target-wasm"));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[axum::debug_handler]
async fn root() -> impl IntoResponse {
    AxumHtmlApp::new((init_page, handle_sleeps).chain())
}

pub fn init_page(mut commands: Commands) {
    let show_fred: bool = true;
    chimera!(
        <div>
            Hello
            <div Styles(hash_map! {"color" => "red"})>
                Yolo
            </div>
            <div Sleep(3)>
                #if show_fred {
                    <div Styles(hash_map! {"color" => "blue"})>
                        Fred
                    </div>
                }
            </div>
        </div>
    )
    .spawn(&mut commands);
}

#[derive(Component, Clone)]
struct Sleep(u64);

fn handle_sleeps(query: Query<(Entity, &Sleep)>, mut async_tasks: ResMut<AsyncTasks>) {
    for (entity, Sleep(duration)) in &query {
        let duration = duration.clone();
        async_tasks.run_async(entity, async move |cb: WorldCallback| {
            tokio::time::sleep(std::time::Duration::from_secs(duration)).await;
            cb.with_world(move |world| {
                let ent = chimera!(<h1>Slept for {duration} seconds</h1>).spawn_with_world(world);
                world.entity_mut(entity).add_child(ent);
            })
            .await;
        });
    }
}
