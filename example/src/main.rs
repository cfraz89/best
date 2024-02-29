use axum::{body::Body, response::IntoResponse, routing::get, Router};
use hevy::html::{plugin::RenderHtmlPlugin, stream::AppHtmlStream};
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
    let mut app = App::new();
    app.add_plugins(RenderHtmlPlugin);
    app.add_systems(Startup, init_page);
    Body::from_stream(AppHtmlStream::new(app))
}

pub fn init_page(mut commands: Commands) {
    ecn!(commands,
    <Div Page> {
        "Hello"
        <Div StyleAttr(hash_map! {"color" => "red"})> {
            "Yolo"
        }
    });
}
