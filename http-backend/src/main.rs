mod inmemory;

use std::sync::Arc;

use crate::inmemory::orderbook::{Order_Book, Orderbook};
use crate::inmemory::user::Users;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, debug_handler};

use axum::{Router, response::IntoResponse, routing::get};

use serde::{Deserialize, Serialize};
#[derive(Deserialize)]
struct UserReq {
    name: String,
}
#[derive(Clone)]
struct App_struct {
    users: Users,
    order_book: Order_Book,
}

#[derive(Serialize)]
struct UserResponse {
    message: String,
    error: String,
}
#[tokio::main]
async fn main() {
    let Appstruct = App_struct {
        users: Users::new(),
        order_book: Order_Book::new(),
    };
    let app = Router::new()
        .route("/get", get(health_check))
        .route("/orderBook", post(handleorder_book))
        .route("/user", post(userhandler))
        .with_state(Appstruct);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
async fn health_check() -> impl IntoResponse {
    "health_check"
}
#[debug_handler]
async fn handleorder_book(
    State(state): State<App_struct>,
    Json(payload): Json<Orderbook>,
) -> Json<UserResponse> {
    if payload.userid.is_empty() || payload.price.is_zero() || payload.quantity.is_zero() {
        return Json(UserResponse {
            message: String::from("name is cannot empty"),
            error: "BAD REQUEST".to_string(),
        });
    }
    state.order_book.addnew_orderbook(
        payload.userid,
        payload.order_type,
        payload.price,
        payload.quantity,
        payload.timestamp,
    );
    Json(UserResponse {
        message: String::from("orderbook_Created"),
        error: "user doesn't created".to_string(),
    })
}

async fn userhandler(
    State(state): State<App_struct>,
    Json(payload): Json<UserReq>,
) -> Json<UserResponse> {
    if payload.name.is_empty() {
        return Json(UserResponse {
            message: String::from("name is cannot empty"),
            error: "BAD REQUEST".to_string(),
        });
    }

    state.users.add_new_user(payload.name);
    Json(UserResponse {
        message: String::from("user created"),
        error: "user doesn't created".to_string(),
    })
}
