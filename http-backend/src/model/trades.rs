use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

pub struct Trade {
    price: Decimal,
    quantity: Decimal,
    buyuser: String,
    selluser: String,
    timestamp: DateTime<Utc>,
}
impl Trade {
    pub fn new(price: Decimal, quantity: Decimal, buyuser: String, selluser: String) -> Trade {
        Trade {
            price: price,
            quantity: quantity,
            buyuser: buyuser,
            timestamp: Utc::now(),
            selluser: selluser,
        }
    }
}

pub struct Trades {
    trades: Vec<Trade>,
}
impl Trades {
    pub fn add_new_trades(&mut self, Trade: Trade) {
        self.trades.push(Trade);
    }
    pub fn new() -> Trades {
        Trades { trades: Vec::new() }
    }
}
