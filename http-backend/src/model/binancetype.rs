use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Debug)]
pub struct binancemsg {
    pub e: String,
    pub E: u64,
    pub s: String,
    pub t: u64,
    pub p: String,
    pub q: String,
    pub T: u64,
    pub m: bool,
    pub M: bool,
}
