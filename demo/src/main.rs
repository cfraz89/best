#![recursion_limit = "512"]

use axum::{
    body::Body,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use bevy::{prelude::*, tasks::futures_lite::stream};
// use demo_index::setup_page;
use elementary_rs_lib::{components::Page, html_render::render_component_instance, signal::Signal};
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
    let mut app = App::new();
    app.add_plugins(render_html_plugin);
    // let mut app = App::new();
    // app.add_systems(Startup, setup_page).update();
    // let page = app
    //     .world
    //     .query_filtered::<Entity, With<Page>>()
    //     .get_single(&app.world)
    //     .unwrap();
    // let response =
    //     render_component_instance(&app.world, page).expect("Couldn't generate a response");
    // Html(response.to_string())
    Body::from_stream(stream::poll_fn(f))
}

#[wasm_bindgen(start)]
async fn start() -> Result<(), JsValue> {
    elementary_rs_lib::init::elementary_init()
}
