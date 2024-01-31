#![recursion_limit = "512"]

use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use demo_index::root_node;
use elementary_rs_lib::{
    node::{Component, Node},
    page::Page,
};
use serde::{Deserialize, Serialize};
use std::fmt::Write;
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
        .nest_service("/wasm", ServeDir::new("target-wasm/pkg"));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> impl IntoResponse {
    Html(Page(root_node()).render("./wasm/demo_index.js"))
}
