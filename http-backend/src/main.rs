use axum::{
    Json, Router,
    extract::Extension,
    middleware,
    response::IntoResponse,
    routing::{get, post},
};
mod handlers;
mod model;
use deadpool_redis::redis::AsyncCommands;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::{
    collections::HashMap,
    str::FromStr,
    sync::mpsc as std_mpsc,
    thread,
    time::{Duration, Instant},
};
use tokio::sync::{mpsc as tokio_mpsc, oneshot};
use tower_http::cors::{Any, CorsLayer};

use crate::{
    handlers::{get_candles, Redis, start_binance},
    model::{
        Order, OrderBook, Ordertype, Position, Positions, TradeEvent, Trades, User, Users,
        create_jwt, verify, verify_password,
    },
};

const SYMBOL_BTC: &str = "btc";

#[derive(Debug)]
pub enum Command {
    CreateUser {
        name: String,
        email: String,
        password: String,
        responder: oneshot::Sender<Result<User, String>>,
    },
    LoginUser {
        email: String,
        password: String,
        responder: oneshot::Sender<Result<User, String>>,
    },
    GetUser {
        userid: String,
        responder: oneshot::Sender<Result<User, String>>,
    },
    Getbalance {
        userid: String,
        responder: oneshot::Sender<Result<Decimal, String>>,
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
    GetOrderbookState {
        responder: oneshot::Sender<OrderbookSnapshot>,
    },
    UpdateMarkPrice {
        symbol: String,
        price: Decimal,
    },
    ClosePosition {
        positionid: String,
        responder: oneshot::Sender<Result<String, String>>,
    },
    OpenPosition {
        userid: String,
        responder: oneshot::Sender<Result<Vec<Position>, String>>,
    },
}

/// A single price level in the orderbook
#[derive(Debug, serde::Serialize, Clone)]
pub struct PriceLevel {
    pub price: Decimal,
    pub quantity: Decimal,
}

/// Snapshot of the current orderbook state
#[derive(Debug, serde::Serialize, Clone)]
pub struct OrderbookSnapshot {
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
}

#[tokio::main]
async fn main() {
    // Load .env from current dir or http-backend/ so TIMESCALE_URL is used
    dotenvy::dotenv().ok();
    if std::env::var("TIMESCALE_URL").is_err() && std::env::var("DATABASE_URL").is_err() {
        if let Ok(dir) = std::env::current_dir() {
            let env_in_backend = dir.join("http-backend").join(".env");
            if env_in_backend.exists() {
                let _ = dotenvy::from_path(env_in_backend);
            }
        }
    }

    let database_url = std::env::var("TIMESCALE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "postgres://localhost/perp".to_string());
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("connect to database");

    let redis = Redis::new("redis://127.0.0.1/");
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods(Any);
    let (tx, rx) = std_mpsc::channel::<Command>();
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        start_binance(tx_clone).await;
    });

    let (trade_tx, trade_rx) = tokio_mpsc::channel::<TradeEvent>(10_000);
    let redis_clone = redis.clone();
    tokio::spawn(async move { trade_redis_worker(redis_clone, trade_rx).await });

    thread::spawn(move || orderbook_thread(rx, trade_tx));
    let protected_routes = Router::new()
        .route("/orderbook", post(orderbook_handler).get(get_orderbook_state))
        .route("/openpositions", get(get_all_positions))
        .route("/closeposition", post(handlecloseposition))
        .route("/getbalance", get(gethandlebalance))
        .route("/me", get(get_me_handler))
        .layer(middleware::from_fn(auth_middleware));
    let app = Router::new()
        .route("/signup", post(signup_handler))
        .route("/login", post(login_handler))
        .route("/candles", get(get_candles))
        .merge(protected_routes)
        .layer(Extension(tx))
        .layer(Extension(pool))
        .layer(cors);
    // .route("/ws", any(handleliquidation))

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct UserReq {
    name: String,
    email: String,
    password: String,
}
#[derive(Deserialize)]
struct Orderreq {
    price: Decimal,
    quantity: Decimal,
    ordertype: String,
    leverage: Decimal,
}
#[derive(Deserialize)]
struct Closereq {
    positionid: String,
}
#[derive(Deserialize)]
struct LoginReq {
    email: String,
    password: String,
}

#[derive(serde::Serialize)]
struct AuthResponse {
    token: String,
    user: User,
}

#[derive(Clone)]
struct AuthUser {
    userid: String,
}

async fn auth_middleware(
    mut req: axum::http::Request<axum::body::Body>,
    next: middleware::Next,
) -> impl IntoResponse {
    let auth_header = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");

    let token = auth_header
        .strip_prefix("Bearer ")
        .map(str::to_string)
        .unwrap_or_default();

    if token.is_empty() {
        return (axum::http::StatusCode::UNAUTHORIZED, "missing bearer token").into_response();
    }

    match verify(&token) {
        Ok(claims) => {
            req.extensions_mut().insert(AuthUser { userid: claims.sub });
            next.run(req).await
        }
        Err(_) => (axum::http::StatusCode::UNAUTHORIZED, "invalid token").into_response(),
    }
}

/// GET /me - returns the current authenticated user's info
async fn get_me_handler(
    Extension(tx): Extension<std_mpsc::Sender<Command>>,
    Extension(auth): Extension<AuthUser>,
) -> impl IntoResponse {
    let (resp_tx, resp_rx) = oneshot::channel();
    let send_result = tokio::task::spawn_blocking({
        let userid = auth.userid.clone();
        move || {
            tx.send(Command::GetUser {
                userid,
                responder: resp_tx,
            })
        }
    })
    .await;

    if send_result.is_err() {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to send command",
        )
            .into_response();
    }
    match resp_rx.await {
        Ok(Ok(user)) => Json(user).into_response(),
        Ok(Err(e)) => (axum::http::StatusCode::NOT_FOUND, e).into_response(),
        Err(_) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "worker failed").into_response(),
    }
}

/// GET /orderbook - returns a snapshot of the current orderbook
async fn get_orderbook_state(
    Extension(tx): Extension<std_mpsc::Sender<Command>>,
    Extension(_auth): Extension<AuthUser>,
) -> impl IntoResponse {
    let (resp_tx, resp_rx) = oneshot::channel();
    let send_result = tokio::task::spawn_blocking({
        move || {
            tx.send(Command::GetOrderbookState {
                responder: resp_tx,
            })
        }
    })
    .await;

    if send_result.is_err() {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to send command",
        )
            .into_response();
    }
    match resp_rx.await {
        Ok(snapshot) => Json(snapshot).into_response(),
        Err(_) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "worker failed").into_response(),
    }
}

async fn gethandlebalance(
    Extension(tx): Extension<std_mpsc::Sender<Command>>,
    Extension(auth): Extension<AuthUser>,
) -> impl IntoResponse {
    let (resp_tx, resp_rx) = oneshot::channel();
    let send_result = tokio::task::spawn_blocking({
        move || {
            tx.send(Command::Getbalance {
                userid: auth.userid,
                responder: resp_tx,
            })
        }
    })
    .await;

    if send_result.is_err() {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed in sending to other thread",
        )
            .into_response();
    }
    match resp_rx.await {
        Ok(Ok(k)) => Json(k).into_response(),
        Ok(Err(e)) => e.into_response(),
        Err(_) => String::from("failed").into_response(),
    }
}
async fn handlecloseposition(
    Extension(tx): Extension<std_mpsc::Sender<Command>>,
    Json(payload): Json<Closereq>,
) -> impl IntoResponse {
    let (resp_tx, resp_rx) = oneshot::channel();
    let send_result = tokio::task::spawn_blocking({
        move || {
            tx.send(Command::ClosePosition {
                positionid: payload.positionid,
                responder: resp_tx,
            })
        }
    })
    .await;
    if send_result.is_err() {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed in sending to other thread",
        )
            .into_response();
    }
    match resp_rx.await {
        Ok(e) => Json(e).into_response(),
        Err(_) => String::from("not closing").into_response(),
    }
}
async fn get_all_positions(
    Extension(tx): Extension<std_mpsc::Sender<Command>>,
    Extension(auth): Extension<AuthUser>,
) -> impl IntoResponse {
    let (resp_tx, resp_rx) = oneshot::channel();
    let send_result = tokio::task::spawn_blocking({
        let tx = tx.clone();
        move || {
            tx.send(Command::OpenPosition {
                userid: auth.userid,
                responder: resp_tx,
            })
        }
    })
    .await;
    if send_result.is_err() {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to send message",
        )
            .into_response();
    }
    match resp_rx.await {
        Ok(Ok(r)) => Json(r).into_response(),
        Ok(Err(_)) => String::from("failed").into_response(),
        Err(_) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "failed").into_response(),
    }
}

async fn orderbook_handler(
    Extension(tx): Extension<std_mpsc::Sender<Command>>,
    Extension(auth): Extension<AuthUser>,
    Json(payload): Json<Orderreq>,
) -> impl IntoResponse {
    let (resp_tx, resp_rx) = oneshot::channel();
    let send_result = tokio::task::spawn_blocking({
        let tx = tx.clone();
        move || {
            tx.send(Command::CreateOrderbook {
                price: payload.price,
                symbol: SYMBOL_BTC.to_string(),
                quantity: payload.quantity,
                ordertype: payload.ordertype,
                userid: auth.userid,
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

async fn signup_handler(
    Extension(tx): Extension<std_mpsc::Sender<Command>>,
    Json(payload): Json<UserReq>,
) -> impl IntoResponse {
    let (resp_tx, resp_rx) = oneshot::channel();

    let send_result = tokio::task::spawn_blocking({
        let tx = tx.clone();
        let name = payload.name.clone();
        let email = payload.email.clone();
        let password = payload.password.clone();
        move || {
            tx.send(Command::CreateUser {
                name,
                email,
                password,
                responder: resp_tx,
            })
        }
    })
    .await;

    if send_result.is_err() {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to send command",
        )
            .into_response();
    }

    match resp_rx.await {
        Ok(Ok(user)) => {
            let token = create_jwt(&user.userid);
            Json(AuthResponse { token, user }).into_response()
        }
        Ok(Err(err)) => (axum::http::StatusCode::BAD_REQUEST, err).into_response(),
        Err(_) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Worker thread failed",
        )
            .into_response(),
    }
}

async fn login_handler(
    Extension(tx): Extension<std_mpsc::Sender<Command>>,
    Json(payload): Json<LoginReq>,
) -> impl IntoResponse {
    let (resp_tx, resp_rx) = oneshot::channel();

    let send_result = tokio::task::spawn_blocking({
        let tx = tx.clone();
        let email = payload.email.clone();
        let password = payload.password.clone();
        move || {
            tx.send(Command::LoginUser {
                email,
                password,
                responder: resp_tx,
            })
        }
    })
    .await;

    if send_result.is_err() {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "failed to send command",
        )
            .into_response();
    }

    match resp_rx.await {
        Ok(Ok(user)) => {
            let token = create_jwt(&user.userid);
            Json(AuthResponse { token, user }).into_response()
        }
        Ok(Err(err)) => (axum::http::StatusCode::UNAUTHORIZED, err).into_response(),
        Err(_) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Worker thread failed",
        )
            .into_response(),
    }
}

async fn trade_redis_worker(redis: Redis, mut rx: tokio_mpsc::Receiver<TradeEvent>) {
    while let Some(event) = rx.recv().await {
        let payload = match serde_json::to_string(&event) {
            Ok(json) => json,
            Err(_) => continue,
        };
        let mut conn = match redis.pool.get().await {
            Ok(conn) => conn,
            Err(_) => continue,
        };
        let queue_key = format!("trades:{}", event.symbol);
        let _: usize = match conn.rpush(queue_key, payload).await {
            Ok(len) => len,
            Err(_) => continue,
        };
    }
}

fn orderbook_thread(rx: std_mpsc::Receiver<Command>, trade_tx: tokio_mpsc::Sender<TradeEvent>) {
    let mut users = Users::new();
    let mut orderbook = OrderBook::new();
    let mut trades = Trades::new();
    let mut positions = Positions::new();
    let mut funding_time_rate = Instant::now();
    let mut latestmarkprice: HashMap<String, Decimal> = HashMap::new();
    while let Ok(cmd) = rx.recv() {
        match cmd {
            Command::GetUser { userid, responder } => {
                if let Some(user) = users.getuser(&userid) {
                    let _ = responder.send(Ok(user.clone()));
                } else {
                    let _ = responder.send(Err(String::from("user not found")));
                }
            }
            Command::GetOrderbookState { responder } => {
                // Aggregate bids: highest price first
                let bids: Vec<PriceLevel> = orderbook
                    .bid
                    .iter()
                    .rev()
                    .map(|(price, orders)| {
                        let total_qty: Decimal = orders.iter().map(|o| o.quantity).sum();
                        PriceLevel {
                            price: *price,
                            quantity: total_qty,
                        }
                    })
                    .collect();
                // Aggregate asks: lowest price first
                let asks: Vec<PriceLevel> = orderbook
                    .ask
                    .iter()
                    .map(|(price, orders)| {
                        let total_qty: Decimal = orders.iter().map(|o| o.quantity).sum();
                        PriceLevel {
                            price: *price,
                            quantity: total_qty,
                        }
                    })
                    .collect();
                let _ = responder.send(OrderbookSnapshot { bids, asks });
            }
            Command::Getbalance { userid, responder } => {
                if let Some(user) = users.getusermut(&userid) {
                    let balance = user.balance;
                    let _ = responder.send(Ok(balance));
                } else {
                    let _ = responder.send(Err(String::from("can't found user")));
                }
            }
            Command::ClosePosition {
                positionid,
                responder,
            } => {
                if let Some(markprice) = latestmarkprice.get("btc") {
                    if let Some(closeposi) =
                        positions.close_position(&positionid, markprice, &mut users)
                    {
                        let _ = responder.send(Ok(closeposi));
                    } else {
                        let _ = responder
                            .send(Err(String::from("error in sending it to other thread")));
                    }
                } else {
                    let _ = responder.send(Err(String::from("mark price not found")));
                }
            }
            Command::OpenPosition { userid, responder } => {
                let userid = userid;
                let open_positions = positions.open_position(userid);
                match open_positions {
                    Some(open_) => {
                        let _ = responder.send(Ok(open_));
                    }
                    None => {
                        let _ = responder.send(Err(String::from("position not found")));
                    }
                }
            }
            Command::CreateUser {
                name,
                responder,
                email,
                password,
            } => {
                let user = users.add_new_user(name, email, password);
                let _ = responder.send(user);
            }
            Command::LoginUser {
                email,
                password,
                responder,
            } => {
                let user = users
                    .find_by_email(&email)
                    .ok_or_else(|| String::from("invalid credentials"));

                let result = match user {
                    Ok(user) => {
                        if verify_password(&user.password_hash, &password) {
                            Ok(user.clone())
                        } else {
                            Err(String::from("invalid credentials"))
                        }
                    }
                    Err(err) => Err(err),
                };

                let _ = responder.send(result);
            }
            Command::UpdateMarkPrice { symbol, price } => {
                let symbol = SYMBOL_BTC.to_string();
                latestmarkprice.insert(symbol.clone(), price);
                positions.update_all_position(&symbol, price);
                let liquidations = positions.process_liquidation();
                if !liquidations.is_empty() {
                    println!(
                        "liquidation event: {} position(s) liquidated",
                        liquidations.len()
                    );
                    for liq in &liquidations {
                        println!("  position ID: {}", liq.positionid);
                        println!("  user ID: {}", liq.userid);
                        println!("  symbol: {}", liq.symbol);
                        println!("  side: {:?}", liq.side);
                        println!("  quantity: {}", liq.quantity);
                        println!("  entry Price: {}", liq.entry_price);
                        println!("  liquidation Price: {}", liq.liquidation_price);
                        println!("  margin Lost: {}", liq.margin_lost);
                        println!("  loss: {}", liq.loss);
                        if let Some(user) = users.users.get_mut(&liq.userid) {
                            if liq.loss > Decimal::ZERO {
                                user.balance += liq.loss;
                                println!("  returned {} to user balance", liq.loss);
                            }
                        }
                    }
                }

                if funding_time_rate.elapsed() >= Duration::from_secs(5) {
                    funding_time_rate = Instant::now();

                    if let Some(midprice) = orderbook.midprice() {
                        // Calculate premium index: (midprice - mark_price) / mark_price
                        if price > Decimal::ZERO {
                            let p_index = (midprice - price) / price; // should use binance markprice - indexprice where mark is there perp btc price and index is there spot price btc
                            let mut funding_rate = p_index;
                            let max_rate = Decimal::from_str("0.0075").unwrap();
                            if funding_rate >= max_rate {
                                funding_rate = max_rate
                            } else if funding_rate <= -max_rate {
                                funding_rate = -max_rate;
                            }
                            Positions::manipulate_market_under_fundingrate(
                                &mut positions,
                                funding_rate,
                            );
                        }
                    }
                }
            }

            Command::CreateOrderbook {
                price,
                symbol: _,
                quantity,
                ordertype,
                userid,
                leverage,
                responder,
            } => {
                let user = if let Some(user_ref) = users.users.get_mut(&userid) {
                    user_ref
                } else {
                    let _ = responder.send(Err(String::from("user not found ")));
                    continue;
                };

                match Ordertype::from_str(&ordertype) {
                    Ok(order_type) => {
                        // Avoid division by zero
                        if leverage.is_zero() {
                            let _ = responder.send(Err(String::from("leverage cannot be zero")));
                            continue;
                        }
                        let ordervalue = quantity * price;
                        let marginprice = ordervalue / leverage;
                        let order = Order::new(
                            price,
                            quantity,
                            order_type,
                            leverage,
                            userid,
                            SYMBOL_BTC.to_string(),
                        );
                        if user.balance >= marginprice {
                            user.balance -= marginprice;
                            orderbook.matching_engine(
                                order,
                                &mut trades,
                                &mut positions,
                                &trade_tx,
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
