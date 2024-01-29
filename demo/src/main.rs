#![recursion_limit = "512"]

use std::{collections::HashMap, sync::Arc};

use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use elementary_rs_lib::node::{Component, Node};
use elementary_rs_macros::{node, CustomElement};
use serde::{Deserialize, Serialize};
use std::fmt::Write;

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

struct Document(Node);

impl From<Node> for Document {
    fn from(node: Node) -> Self {
        Self(node)
    }
}

impl IntoResponse for Document {
    fn into_response(self) -> Response<Body> {
        let mut output = String::new();
        write!(
            &mut output,
            "<!doctype html><html><body>{}</body></html>",
            self.0
        )
        .expect("couldn't write");
        Html(output).into_response()
    }
}

const x: i32 = 10;
// basic handler that responds with a static string
async fn root() -> Document {
    node!(
        <div>
            <MyH1>
                {x * 10}
            </MyH1>
        </div>
    )
    .into()
}

#[derive(CustomElement)]
#[custom_element(tag = "my-h1")]
struct MyH1 {}

impl Component for MyH1 {
    fn node(&self) -> Node {
        node! {
            <h1>
            <slot />
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
