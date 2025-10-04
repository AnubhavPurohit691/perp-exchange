use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

enum OrderType {
    BUY(String),
    SELL(String),
}
pub struct Orderbook {
    pub userid: String,
    pub order_type: OrderType,
    pub price: Decimal,
    pub quantity: Decimal,
    pub timestamp: DateTime<Utc>,
}

impl Orderbook {
    fn new() {}
}
