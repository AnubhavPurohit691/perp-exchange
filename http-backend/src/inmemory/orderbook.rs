use std::sync::Arc;

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Deserialize, serde::Serialize, Clone)]
pub enum OrderType {
    BUY(String),
    SELL(String),
}
#[derive(Deserialize, serde::Serialize, Clone)]
pub struct Orderbook {
    pub userid: String,
    pub order_type: OrderType,
    pub price: Decimal,
    pub quantity: Decimal,
    pub timestamp: DateTime<Utc>,
}

impl Orderbook {
    pub fn new(
        userid: String,
        order_type: OrderType,
        price: Decimal,
        quantity: Decimal,
        timestamp: DateTime<Utc>,
    ) -> Orderbook {
        Orderbook {
            userid,
            order_type,
            price,
            quantity,
            timestamp,
        }
    }
}
#[derive(Clone)]
pub struct Order_Book {
    pub orderbook_map: Arc<DashMap<String, Orderbook>>,
}

impl Order_Book {
    pub fn new() -> Order_Book {
        Order_Book {
            orderbook_map: Arc::new(DashMap::new()),
        }
    }

    pub fn addnew_orderbook(
        &self,
        userid: String,
        order_type: OrderType,
        price: Decimal,
        quantity: Decimal,
        timestamp: DateTime<Utc>,
    ) {
        let orderbook = Orderbook::new(userid, order_type, price, quantity, timestamp);
        let userid = orderbook.userid.clone();
        self.orderbook_map.insert(userid, orderbook);
    }
}
