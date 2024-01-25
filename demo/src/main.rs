#![recursion_limit = "512"]

use std::{collections::HashMap, sync::Arc};

use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use elementary_rs_lib::node::{self, Component, HtmlElement, Node};
use elementary_rs_macros::{node, render_node, Component};
use quote::{quote, ToTokens};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> String {
    render_node!(
        <div>
            <MyH1>
                Hello, world!
            </MyH1>
        </div>
    )
}

#[derive(Component)]
#[component(tag = "my-h1")]
struct MyH1 {
    child_nodes: Arc<Vec<Node>>,
}

impl Component for MyH1 {
    fn node(&self) -> Node {
        node! {
            <h1>
                #for child_node in self.child_nodes.iter() {
                    #child_node
                }
            </h1>
        }
    }
}

async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    // insert your application logic here
    let user = User {
        id: 1337,
        username: payload.username,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}
