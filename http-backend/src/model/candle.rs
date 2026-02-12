use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Candle {
    pub symbol: String,
    pub timeframe: String,
    pub bucket_start: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub trade_count: i64,
    pub open_ts: DateTime<Utc>,
    pub close_ts: DateTime<Utc>,
}
