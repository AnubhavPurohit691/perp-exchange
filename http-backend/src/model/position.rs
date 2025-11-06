use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
    str::FromStr,
};

use nanoid::nanoid;
use rust_decimal::Decimal;
use serde::Serialize;

use crate::model::{Ordertype, order, user};
#[derive(Debug, Clone)]
pub struct Position {
    pub positionid: String,
    pub userid: String,
    pub symbol: String,
    pub side: Ordertype,
    pub quantity: Decimal,
    pub entry_price: Decimal,
    pub leverage: Decimal,
    pub margin: Decimal,
    pub un_pnl: Decimal,
    pub real_pnl: Decimal,
    pub liquidated: bool,
    pub equity: Decimal,
}
impl Position {
    pub fn new(
        symbol: String,
        quantity: Decimal,
        entry_price: Decimal,
        leverage: Decimal,
        side: Ordertype,
        userid: String,
    ) -> Position {
        let margin = (quantity * entry_price) / leverage;
        Position {
            positionid: nanoid!(),
            userid,
            symbol,
            quantity: quantity,
            entry_price: entry_price,
            leverage: leverage,
            un_pnl: Decimal::ZERO,
            real_pnl: Decimal::ZERO,
            margin: margin,
            liquidated: false,
            side: side,
            equity: Decimal::ZERO,
        }
    }
    fn update_unrealized_pnl(&mut self, markprice: Decimal) {
        match self.side {
            Ordertype::Buy => self.un_pnl = (markprice - self.entry_price) * self.quantity,
            Ordertype::Sell => self.un_pnl = (self.entry_price - markprice) * self.quantity,
        }
        self.equity = self.margin + self.un_pnl;
    }
    fn margin_ratio(&self) -> Decimal {
        if self.equity.is_zero() || self.equity < Decimal::ZERO {
            return Decimal::ZERO;
        }
        self.equity / (self.quantity * self.entry_price)
        //basically if margin rotion is equal maintenance ration then it should liquidate if it is below 1 then it is taking loss or if 1 se uper hai toh profit hai baale baale
    }
    fn isliquidatable(&self, maintenance_margin_ratio: Decimal) -> bool {
        !self.liquidated && self.margin_ratio() <= maintenance_margin_ratio
    }
    fn liquidation_price(&self) -> Decimal {
        let maintainance_rate = Decimal::from_str_exact("0.005").unwrap();
        let actualprice = self.entry_price * self.quantity;
        match self.side {
            Ordertype::Buy => {
                self.entry_price - ((self.margin - actualprice * maintainance_rate) / self.quantity)
            }
            Ordertype::Sell => {
                self.entry_price + ((self.margin - actualprice * maintainance_rate) / self.quantity)
            }
        }
    }
}
#[derive(Debug, Clone)]
struct LiquidationCandidate {
    pub positionid: String,
    pub userid: String,
    pub symbol: String,
    pub margin_ratio: Decimal,
}
impl LiquidationCandidate {
    pub fn new(position: &mut Position) -> Self {
        let margin_ratio = position.margin_ratio();
        Self {
            userid: position.userid.clone(),
            symbol: position.symbol.clone(),
            margin_ratio: margin_ratio,
            positionid: position.positionid.clone(),
        }
    }
}
impl PartialEq for LiquidationCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.margin_ratio == other.margin_ratio
    }
}
impl Eq for LiquidationCandidate {}

impl PartialOrd for LiquidationCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for LiquidationCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        other.margin_ratio.cmp(&self.margin_ratio)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LiquidatedPosition {
    pub positionid: String,
    pub userid: String,
    pub symbol: String,
    pub side: Ordertype,
    pub quantity: Decimal,
    pub entry_price: Decimal,
    pub liquidation_price: Decimal,
    pub loss: Decimal,
    pub margin_lost: Decimal,
    pub leverage: Decimal,
}
pub struct Positions {
    pub positions: HashMap<String, Position>,
    liquidation_queue: BinaryHeap<LiquidationCandidate>,
    pub user_position: HashMap<String, Vec<String>>,
    pub maintenance_margin_ratio: Decimal,
}

impl Positions {
    pub fn new() -> Positions {
        Positions {
            positions: HashMap::new(),
            liquidation_queue: BinaryHeap::new(),
            maintenance_margin_ratio: Decimal::from_str_exact("0.01").unwrap(),
            user_position: HashMap::new(),
        }
    }
    pub fn add_position(
        &mut self,
        userid: String,
        symbol: String,
        side: Ordertype,
        quantity: Decimal,
        trade_price: Decimal,
        leverage: Decimal,
    ) -> String {
        let position = Position::new(
            symbol,
            quantity,
            trade_price,
            leverage,
            side,
            userid.clone(),
        );
        let positionid = position.positionid.clone();
        self.positions.insert(positionid.clone(), position);
        self.user_position
            .entry(userid)
            .or_insert_with(Vec::new)
            .push(positionid.clone());

        positionid
    }
    pub fn update_all_position(&mut self, symbol: &str, markprice: Decimal) {
        for position in self.positions.values_mut() {
            if position.symbol == symbol && !position.liquidated {
                position.update_unrealized_pnl(markprice);
                if position.isliquidatable(self.maintenance_margin_ratio) {
                    self.liquidation_queue
                        .push(LiquidationCandidate::new(position));
                }
            }
        }
    }
    pub fn process_liquidation(&mut self) -> Vec<LiquidatedPosition> {
        let mut liquidated_position = Vec::new();
        while let Some(candidate) = self.liquidation_queue.pop() {
            if let Some(position) = self.positions.get_mut(&candidate.positionid) {
                if position.isliquidatable(self.maintenance_margin_ratio) {
                    let liquidation_result = LiquidatedPosition {
                        positionid: position.positionid.clone(),
                        userid: position.userid.clone(),
                        symbol: position.symbol.clone(),
                        side: position.side,
                        quantity: position.quantity,
                        entry_price: position.entry_price,
                        leverage: position.leverage,
                        liquidation_price: position.liquidation_price(),
                        loss: position.equity,
                        margin_lost: position.margin,
                    };
                    position.liquidated = true;
                    position.quantity = Decimal::ZERO;
                    position.equity = Decimal::ZERO;
                    liquidated_position.push(liquidation_result);
                }
            }
        }
        liquidated_position
    }
}
