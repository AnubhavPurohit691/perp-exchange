use std::{
    collections::{BTreeMap, VecDeque},
    str::FromStr,
};

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;

use crate::model::{Positions, Trade, Trades};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
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

#[derive(Debug)]
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

#[derive(Debug)]
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
                .ask
                .entry(price)
                .or_insert_with(VecDeque::new)
                .push_back(order),
        }
    }
    pub fn midprice(&self) -> Option<Decimal> {
        let bestbid = self.bid.keys().last()?;
        let bestask = self.ask.keys().next()?;
        Some((bestask + bestbid) / Decimal::from(2))
    }

    pub fn matching_engine(
        &mut self,
        order: Order,
        trades: &mut Trades,
        positions: &mut Positions,
    ) {
        let mut remaining_quantity = order.quantity;
        let mut to_remove = vec![];
        match order.ordertype {
            Ordertype::Buy => {
                for (&sellprice, sellorders) in &mut self.ask {
                    if sellprice > order.price {
                        break;
                    }
                    if remaining_quantity <= Decimal::ZERO {
                        break;
                    }
                    for sellorder in sellorders.iter_mut() {
                        if remaining_quantity <= Decimal::ZERO {
                            break;
                        }
                        let trade_quantity = remaining_quantity.min(sellorder.quantity);
                        remaining_quantity -= trade_quantity;
                        sellorder.quantity -= trade_quantity;
                        let tradeprice = sellprice;
                        let trade = Trade::new(
                            tradeprice,
                            trade_quantity,
                            order.userid.clone(),
                            sellorder.userid.clone(),
                        );
                        trades.add_new_trades(trade);
                        positions.add_position(
                            order.userid.clone(),
                            order.symbol.clone(),
                            Ordertype::Buy,
                            trade_quantity,
                            tradeprice,
                            order.leverage,
                        );
                        positions.add_position(
                            sellorder.userid.clone(),
                            sellorder.symbol.clone(),
                            sellorder.ordertype,
                            trade_quantity,
                            tradeprice,
                            sellorder.leverage,
                        );
                    }
                    sellorders.retain(|o| o.quantity > Decimal::ZERO);
                    if sellorders.is_empty() {
                        to_remove.push(sellprice);
                    }
                }
                for p in to_remove {
                    self.ask.remove(&p);
                }
                if remaining_quantity > Decimal::ZERO {
                    let orderquantity = remaining_quantity;
                    self.add_new_order(
                        order.price,
                        orderquantity,
                        order.ordertype,
                        order.leverage,
                        order.symbol,
                        order.userid,
                    );
                }
            }
            Ordertype::Sell => {
                for (&buyprice, buyorders) in &mut self.bid.iter_mut().rev() {
                    if order.price > buyprice {
                        break;
                    }
                    if remaining_quantity <= Decimal::ZERO {
                        break;
                    }
                    for buyorder in buyorders.iter_mut() {
                        if remaining_quantity <= Decimal::ZERO {
                            break;
                        }
                        let tradequan = remaining_quantity.min(buyorder.quantity);
                        remaining_quantity -= tradequan;
                        buyorder.quantity -= tradequan;
                        let tradeprice = buyprice;
                        let trade = Trade::new(
                            tradeprice,
                            tradequan,
                            buyorder.userid.clone(),
                            order.userid.clone(),
                        );
                        trades.add_new_trades(trade);
                        positions.add_position(
                            order.userid.clone(),
                            order.symbol.clone(),
                            Ordertype::Sell,
                            tradequan,
                            tradeprice,
                            order.leverage,
                        );
                        positions.add_position(
                            buyorder.userid.clone(),
                            buyorder.symbol.clone(),
                            buyorder.ordertype,
                            tradequan,
                            tradeprice,
                            buyorder.leverage,
                        );
                    }
                    buyorders.retain(|o| o.quantity > Decimal::ZERO);
                    if buyorders.is_empty() {
                        to_remove.push(buyprice);
                    }
                }
                for p in to_remove {
                    self.bid.remove(&p);
                }
                if remaining_quantity > Decimal::ZERO {
                    let orderquantity = remaining_quantity;
                    self.add_new_order(
                        order.price,
                        orderquantity,
                        order.ordertype,
                        order.leverage,
                        order.symbol,
                        order.userid,
                    );
                }
            }
        }
    }
}
