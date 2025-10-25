use std::{
    collections::{BTreeMap, VecDeque},
    str::FromStr,
};

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
#[derive(Clone, Copy)]
pub enum Ordertype {
    Buy,
    Sell,
}
impl FromStr for Ordertype {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "buy" => Ok(Ordertype::Buy),
            "sell" => Ok(Ordertype::Sell),
            _ => Err(format!("invalid order type")),
        }
    }
}

pub struct Order {
    price: Decimal,
    symbol: String,
    quantity: Decimal,
    timestamp: DateTime<Utc>,
    ordertype: Ordertype,
    userid: String,
    leverage: Decimal,
}
impl Order {
    pub fn new(
        price: Decimal,
        quantity: Decimal,
        // timestamp: DateTime<Utc>,
        ordertype: Ordertype,
        leverage: Decimal,
        userid: String,
        symbol: String,
    ) -> Order {
        Order {
            price: price,
            timestamp: Utc::now(),
            ordertype: ordertype,
            userid: userid,
            leverage: leverage,
            symbol: symbol,
            quantity: quantity,
        }
    }
}
pub struct OrderBook {
    bid: BTreeMap<Decimal, VecDeque<Order>>,
    ask: BTreeMap<Decimal, VecDeque<Order>>,
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            bid: BTreeMap::new(),
            ask: BTreeMap::new(),
        }
    }
    pub fn add_new_order(
        &mut self,
        price: Decimal,
        quantity: Decimal,
        ordertype: Ordertype,
        leverage: Decimal,
        symbol: String,
        userid: String,
    ) {
        let order = Order::new(price, quantity, ordertype, leverage, userid, symbol);
        match ordertype {
            Ordertype::Buy => {
                self.bid
                    .entry(price)
                    .or_insert_with(VecDeque::new)
                    .push_back(order);
            }
            Ordertype::Sell => self
                .bid
                .entry(price)
                .or_insert_with(VecDeque::new)
                .push_back(order),
        }
    }
}
