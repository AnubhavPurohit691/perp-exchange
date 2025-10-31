use std::collections::HashMap;

use rust_decimal::Decimal;

use crate::model::{Ordertype, order};
#[derive(Debug, Clone)]
pub struct Position {
    symbol: String,
    quantity: Decimal,
    entry_price: Decimal,
    leverage: Decimal,
    total_margin: Decimal,
    un_pnl: Decimal,
    real_pnl: Decimal,
}
impl Position {
    pub fn new(symbol: String) -> Position {
        Position {
            symbol,
            quantity: Decimal::ZERO,
            entry_price: Decimal::ZERO,
            leverage: Decimal::ZERO,
            un_pnl: Decimal::ZERO,
            real_pnl: Decimal::ZERO,
            total_margin: Decimal::ZERO,
        }
    }
}

pub struct Positions {
    pub positions: HashMap<String, HashMap<String, Position>>,
}
impl Positions {
    pub fn new() -> Positions {
        Positions {
            positions: HashMap::new(),
        }
    }
    pub fn update_position(
        &mut self,
        userid: String,
        symbol: String,
        side: Ordertype,
        quantity: Decimal,
        trade_price: Decimal,
        leverage: Decimal,
    ) {
        // let position = Position::new(
        //     symbol,
        //     side,
        //     quantity,
        //     entry_price,
        //     leverage,
        //     Decimal::ZERO,
        //     Decimal::ZERO,
        // );
        // self.positions.entry(userid).or_default().push(position);
    //     let user_position = self.positions.entry(userid.clone()).or_default();
    //     let position = user_position
    //         .entry(symbol.clone())
    //         .or_insert_with(|| Position::new(symbol.clone()));
    //     let signed_quantity = match side {
    //         Ordertype::Buy => quantity,
    //         Ordertype::Sell => -quantity,
    //     };
    //     let old_quantity = position.quantity;
    //     let new_quantity = position.quantity + signed_quantity;
    //     if old_quantity == Decimal::ZERO {
    //         position.quantity = new_quantity;
    //         position.entry_price = trade_price;
    //         position.leverage = leverage;
    //         return;
    //     }
    //     let is_samedirection = (old_quantity > Decimal::ZERO && side == Ordertype::Buy)
    //         || (old_quantity < Decimal::ZERO && side == Ordertype::Sell);
    //     if is_samedirection {
    //         let oldvalue = old_quantity.abs() * position.entry_price;
    //         let trade_value = quantity * trade_price;
    //         let new_absolute_quantity = new_quantity.abs();
    //         position.entry_price = (oldvalue + trade_value) / new_absolute_quantity
    //     }
    // }
}
