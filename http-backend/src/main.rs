use axum::{Json, Router, extract::Extension, response::IntoResponse, routing::post};
mod model;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::{str::FromStr, sync::mpsc, thread};
use tokio::sync::oneshot;

use crate::model::{OrderBook, Ordertype, User, Users};

#[derive(Debug)]
pub enum Command {
    CreateUser {
        name: String,
        responder: oneshot::Sender<Result<User, String>>,
    },
    CreateOrderbook {
        price: Decimal,
        symbol: String,
        quantity: Decimal,
        ordertype: String,
        userid: String,
        leverage: Decimal,
        responder: oneshot::Sender<Result<String, String>>,
    },
}

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel::<Command>();
    thread::spawn(move || orderbook_thread(rx));

    let app = Router::new()
        .route("/user", post(create_user_handler))
        .route("/orderbook", post(orderbook_handler))
        .layer(Extension(tx));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct UserReq {
    name: String,
}
#[derive(Deserialize)]
struct Orderreq {
    price: Decimal,
    symbol: String,
    quantity: Decimal,
    ordertype: String,
    userid: String,
    leverage: Decimal,
}

async fn orderbook_handler(
    Extension(tx): Extension<mpsc::Sender<Command>>,
    Json(payload): Json<Orderreq>,
) -> impl IntoResponse {
    let (resp_tx, resp_rx) = oneshot::channel();
    let send_result = tokio::task::spawn_blocking({
        let tx = tx.clone();
        move || {
            tx.send(Command::CreateOrderbook {
                price: payload.price,
                symbol: payload.symbol,
                quantity: payload.quantity,
                ordertype: payload.ordertype,
                userid: payload.userid,
                leverage: payload.leverage,
                responder: resp_tx,
            })
        }
    })
    .await;
    if send_result.is_err() {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to send command orderbook",
        )
            .into_response();
    }
    match resp_rx.await {
        Ok(result) => Json(result).into_response(),
        Err(_) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "worker thread failed",
        )
            .into_response(),
    }
}

async fn create_user_handler(
    Extension(tx): Extension<mpsc::Sender<Command>>,
    Json(payload): Json<UserReq>,
) -> impl IntoResponse {
    let (resp_tx, resp_rx) = oneshot::channel();

    let send_result = tokio::task::spawn_blocking({
        let tx = tx.clone();
        let name = payload.name.clone();
        move || {
            tx.send(Command::CreateUser {
                name,
                responder: resp_tx,
            })
        }
    })
    .await;

    if send_result.is_err() {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to send command",
        )
            .into_response();
    }

    match resp_rx.await {
        Ok(result) => Json(result).into_response(),
        Err(_) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Worker thread failed",
        )
            .into_response(),
    }
}

fn orderbook_thread(mut rx: mpsc::Receiver<Command>) {
    let mut users = Users::new();
    let mut orderbook = OrderBook::new();
    while let Ok(cmd) = rx.recv() {
        match cmd {
            Command::CreateUser { name, responder } => {
                let user = users.add_new_user(name);
                let _ = responder.send(Ok(user));
            }
            Command::CreateOrderbook {
                price,
                symbol,
                quantity,
                ordertype,
                userid,
                leverage,
                responder,
            } => {
                let user = if let Some(user) = users.getuser(&userid) {
                    user
                } else {
                    let _ = responder.send(Err(String::from("user not found ")));
                    continue;
                };

                match Ordertype::from_str(&ordertype) {
                    Ok(Ordertype) => {
                        let ordervalue = quantity * price;
                        let marginprice = ordervalue / leverage;
                        if user.balance >= marginprice {
                            orderbook.add_new_order(
                                price, quantity, Ordertype, leverage, symbol, userid,
                            );
                            let _ = responder.send(Ok(String::from("order created succesfully")));
                        } else {
                            let _ =
                                responder.send(Err(String::from("user don't have enough balance")));
                        }
                    }
                    Err(err) => {
                        let _ = responder.send(Err(err));
                    }
                }
            }
        }
    }
}
