use std::{
    str::FromStr,
    sync::{Arc, mpsc},
    time::Duration,
};

use futures_util::{StreamExt, lock::Mutex};
use rust_decimal::Decimal;
use tokio::time::interval;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::{Command, model::binancemsg};

pub async fn start_binance(tx: mpsc::Sender<Command>) {
    // let (res_tx, res_rx) = oneshot::channel();
    let url = "wss://stream.binance.com:9443/ws/btcusdt@trade";
    let tx = tx.clone();
    let (ws_stream, _) = connect_async(url).await.unwrap();
    let (mut write, mut read) = ws_stream.split();
    let mut indexprice = Arc::new(Mutex::new(Decimal::ZERO));
    // tokio::spawn({
    //     let tx = tx.clone();
    //     async move {
    //         let mut interval = interval(Duration::from_secs(10));
    //         loop {
    //             interval.tick().await;
    //             tx.send(Command::FundingRate {
    //                 price: *indexprice.lock().await,
    //             })
    //             .unwrap();
    //         }
    //     }
    // });

    while let Some(msg) = read.next().await {
        if let Ok(Message::Text(text)) = msg {
            // println!("{}", text);
            match serde_json::from_str::<binancemsg>(&text) {
                Ok(trade) => {
                    // println!("{:?}", trade);
                    let price = Decimal::from_str(&trade.p).unwrap();
                    // let quantity = Decimal::from_str(&trade.q).unwrap();
                    // indexprice = price;
                    // println!("{}", price);
                    if let Err(e) = tx.send(Command::UpdateMarkPrice {
                        symbol: String::from("btc"),
                        price: price,
                    }) {
                        eprintln!("failed to send mark price update: {}", e);
                        break; // Exit if receiver is dropped
                    }
                }
                Err(e) => {
                    println!("error{}", e)
                }
            }
        }
    }
}
