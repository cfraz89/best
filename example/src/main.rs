#![feature(async_closure)]
use axum::{response::IntoResponse, routing::get, Router};
use hevy::axum_html::AxumHtmlApp;
use hevy::r#async::AsyncCallbacks;
use tower_http::services::ServeDir;

use bevy::prelude::*;
use hevy::html::*;
use hevy::prelude::*;

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
    AxumHtmlApp::new((init_page, replace_yolo).chain())
}

pub fn init_page(mut commands: Commands) {
    hevy!(commands,
        <Div Page> {
            "Hello"
            <Div Styles(hash_map! {"color" => "red"})> {
                "Yolo"
            }
            <Div NotYolo>
        }
    );
}

#[derive(Component)]
struct NotYolo;

fn replace_yolo(query: Query<Entity, With<NotYolo>>, mut async_tasks: ResMut<AsyncTasks>) {
    for entity in &query {
        async_tasks.run(entity, async move |cbs: AsyncCallbacks| {
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            cbs.with_commands(move |commands| {
                let ent = hevy!(commands,<H1>{"Not yolo"}).id();
                commands.entity(entity).add_child(ent);
                // set_child(
                //     commands,
                //     entity,
                //     Text("Not yolo".to_string()),
                // );
            })
            .await;
        });
    }
}
