use bourse_de::types::{Nanos, Order, OrderId, Price, Side, Status, Trade, TraderId, Vol};

pub fn is_bid(side: &Side) -> bool {
    match side {
        Side::Bid => true,
        Side::Ask => false,
    }
}

pub type PyTrade = (Nanos, bool, Price, Vol, OrderId, OrderId);

pub fn cast_trade(trade: &Trade) -> PyTrade {
    (
        trade.t,
        is_bid(&trade.side),
        trade.price,
        trade.vol,
        trade.active_order_id,
        trade.passive_order_id,
    )
}

pub fn status_to_int(status: &Status) -> u8 {
    match status {
        Status::New => 0,
        Status::Active => 1,
        Status::Filled => 2,
        Status::Cancelled => 3,
        Status::Rejected => 4,
    }
}

pub type PyOrder = (bool, u8, Nanos, Nanos, Vol, Vol, Price, TraderId, OrderId);

pub fn cast_order(order: &Order) -> PyOrder {
    (
        is_bid(&order.side),
        status_to_int(&order.status),
        order.arr_time,
        order.end_time,
        order.vol,
        order.start_vol,
        order.price,
        order.trader_id,
        order.order_id,
    )
}
