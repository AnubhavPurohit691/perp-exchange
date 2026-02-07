use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct TradeEvent {
    pub symbol: String,
    pub price: Decimal,
    pub quantity: Decimal,
    pub buyuser: String,
    pub selluser: String,
    pub timestamp: DateTime<Utc>,
}

pub struct Trade {
    price: Decimal,
    quantity: Decimal,
    buyuser: String,
    selluser: String,
    symbol: String,
    timestamp: DateTime<Utc>,
}
impl Trade {
    pub fn new(
        price: Decimal,
        quantity: Decimal,
        buyuser: String,
        selluser: String,
        symbol: String,
    ) -> Trade {
        Trade {
            price: price,
            quantity: quantity,
            buyuser: buyuser,
            timestamp: Utc::now(),
            selluser: selluser,
            symbol: symbol,
        }
    }

    pub fn to_event(&self) -> TradeEvent {
        TradeEvent {
            symbol: self.symbol.clone(),
            price: self.price,
            quantity: self.quantity,
            buyuser: self.buyuser.clone(),
            selluser: self.selluser.clone(),
            timestamp: self.timestamp,
        }
    }
}

pub struct Trades {
    trades: Vec<Trade>,
}
impl Trades {
    pub fn add_new_trades(&mut self, trade: Trade) -> TradeEvent {
        let event = trade.to_event();
        self.trades.push(trade);
        event
    }
    pub fn new() -> Trades {
        Trades { trades: Vec::new() }
    }
}
