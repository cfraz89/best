#![recursion_limit = "512"]

use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use demo_index::IndexPage;
use elementary_rs_lib::{page::Page, signal::Signal};
use tower_http::services::ServeDir;
use wasm_bindgen::prelude::*;

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
    let page = IndexPage { x: Signal::new(20) }
        .render()
        .await
        .expect("Render page didnt return!");
    Html(page)
}

#[wasm_bindgen(start)]
async fn start() -> Result<(), JsValue> {
    elementary_rs_lib::init::elementary_init()
}
