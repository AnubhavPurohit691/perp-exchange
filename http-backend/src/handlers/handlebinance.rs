use std::{str::FromStr, sync::mpsc};

use futures_util::StreamExt;
use rust_decimal::Decimal;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::{Command, model::binancemsg};

pub async fn start_binance(tx: mpsc::Sender<Command>) {
    // let (res_tx, res_rx) = oneshot::channel();
    let url = "wss://stream.binance.com:9443/ws/btcusdt@trade";
    let tx = tx.clone();
    let (ws_stream, _) = connect_async(url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();
    while let Some(msg) = read.next().await {
        if let Ok(Message::Text(text)) = msg {
            println!("{}", text);
            match serde_json::from_str::<binancemsg>(&text) {
                Ok(trade) => {
                    // println!("{:?}", trade);
                    let price = Decimal::from_str(&trade.p).unwrap();
                    let quantity = Decimal::from_str(&trade.q).unwrap();
                    println!("{}", price);
                    tx.send(Command::CreateLiquidation {
                        price: price,
                        quantity: quantity,
                    })
                    .unwrap();
                }
                Err(e) => {
                    println!("error{}", e)
                }
            }
        }
    }
}
