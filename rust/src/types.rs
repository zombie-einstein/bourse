use bourse_de::types::{Nanos, Order, OrderId, Price, Trade, TraderId, Vol};
use numpy::PyArray1;

pub type PyTrade = (Nanos, bool, Price, Vol, OrderId, OrderId);

pub fn cast_trade(trade: &Trade) -> PyTrade {
    (
        trade.t,
        trade.side.into(),
        trade.price,
        trade.vol,
        trade.active_order_id,
        trade.passive_order_id,
    )
}

pub type PyOrder = (bool, u8, Nanos, Nanos, Vol, Vol, Price, TraderId, OrderId);

pub fn cast_order(order: &Order) -> PyOrder {
    (
        order.side.into(),
        order.status.into(),
        order.arr_time,
        order.end_time,
        order.vol,
        order.start_vol,
        order.price,
        order.trader_id,
        order.order_id,
    )
}

pub type NumpyInstructions<'a> = (
    &'a PyArray1<u32>,
    &'a PyArray1<bool>,
    &'a PyArray1<Vol>,
    &'a PyArray1<TraderId>,
    &'a PyArray1<Price>,
    &'a PyArray1<OrderId>,
);
