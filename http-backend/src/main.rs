mod inmemory;

use crate::inmemory::user::{User, Users};
use axum::Json;
use axum::extract::State;
use axum::routing::post;

use axum::{Router, response::IntoResponse, routing::get};

#[derive(Serialize)]
#[serde(untagged)]
enum ResponseMessage {
    Text(String),
    User(User),
}
use serde::{Deserialize, Serialize};
#[derive(Deserialize)]
struct UserReq {
    name: String,
}

#[derive(Serialize)]
struct UserResponse {
    message: ResponseMessage,
    error: String,
}
#[tokio::main]
async fn main() {
    let users = Users::new();
    let app = Router::new()
        .route("/get", get(health_check))
        .route("/orderBook", post(handleorder_book))
        .route("/user", post(userhandler))
        .with_state(users);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
async fn health_check() -> impl IntoResponse {
    "health_check"
}
async fn handleorder_book() -> impl IntoResponse {}

async fn userhandler(
    State(users): State<Users>,
    Json(payload): Json<UserReq>,
) -> Json<UserResponse> {
    if payload.name.is_empty() {
        return Json(UserResponse {
            message: ResponseMessage::Text("name is cannot empty".to_string()),
            error: "BAD REQUEST".to_string(),
        });
    }

    let user = users.add_new_user(payload.name);
    Json(UserResponse {
        message: ResponseMessage::User(user),
        error: "".to_string(),
    })
}
