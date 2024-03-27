//! Type aliases and order data-structures

use serde::{Deserialize, Serialize};

/// Order-id
pub type OrderId = usize;
/// Order lookup key
pub type OrderKey = (Side, u32, u64);
/// Simulated time
pub type Nanos = u64;
/// Prices
pub type Price = u32;
/// Order/trade volumes
pub type Vol = u32;
/// Id of an agent/trader
pub type TraderId = u32;
/// Count of orders
pub type OrderCount = u32;

/// Market side
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Side {
    Bid,
    Ask,
}

impl From<bool> for Side {
    fn from(side: bool) -> Side {
        match side {
            true => Self::Bid,
            false => Self::Ask,
        }
    }
}

impl From<Side> for bool {
    fn from(side: Side) -> bool {
        match side {
            Side::Bid => true,
            Side::Ask => false,
        }
    }
}

/// Order status
#[derive(Clone, PartialEq, Eq, Copy, Debug, Serialize, Deserialize)]
pub enum Status {
    /// Newly created, not placed
    New,
    /// Active (i.e. on the market)
    Active,
    /// Filled
    Filled,
    /// Cancelled
    Cancelled,
    /// Rejected, e.g. a market order
    /// placed in a no-trading period
    Rejected,
}

impl From<Status> for u8 {
    fn from(status: Status) -> u8 {
        match status {
            Status::New => 0,
            Status::Active => 1,
            Status::Filled => 2,
            Status::Cancelled => 3,
            Status::Rejected => 4,
        }
    }
}

/// Order data
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Order {
    /// Order side
    pub side: Side,
    /// Status of the order
    pub status: Status,
    /// Arrival time of the order
    pub arr_time: Nanos,
    /// End time of the order (filled,
    /// cancelled etc.)
    pub end_time: Nanos,
    /// Current volume of the order
    pub vol: Vol,
    /// Original volume when the
    /// order was placed
    pub start_vol: Vol,
    /// Price of the order
    pub price: Price,
    /// Id of the trader/agent who
    /// placed the order
    pub trader_id: TraderId,
    /// Id of the order
    pub order_id: OrderId,
}

/// Trade record
#[derive(Serialize, Deserialize)]
pub struct Trade {
    /// Trade time
    pub t: Nanos,
    /// Trade side
    pub side: Side,
    /// trade price
    pub price: Price,
    /// Trade volume
    pub vol: Vol,
    /// Id of the aggressive order
    pub active_order_id: OrderId,
    /// Id of the passive order
    pub passive_order_id: OrderId,
}

impl Order {
    /// Initialise a buy limit-order
    ///
    /// # Arguments
    ///
    /// - `t` - Order creation time
    /// - `vol` - Order volume
    /// - `price` - Limit price of the order
    /// - `trader_id` - Id of the agent/trader
    /// - `order_id` - Id of the order
    ///
    pub fn buy_limit(
        t: Nanos,
        vol: Vol,
        price: Price,
        trader_id: TraderId,
        order_id: OrderId,
    ) -> Order {
        Order {
            side: Side::Bid,
            status: Status::New,
            arr_time: t,
            end_time: Nanos::MAX,
            vol,
            start_vol: vol,
            price,
            trader_id,
            order_id,
        }
    }

    /// Initialise a buy market-order
    ///
    /// # Arguments
    ///
    /// - `t` - Order creation time
    /// - `vol` - Order volume
    /// - `trader_id` - Id of the agent/trader
    /// - `order_id` - Id of the order
    ///
    pub fn buy_market(t: Nanos, vol: Vol, trader_id: TraderId, order_id: OrderId) -> Order {
        Order {
            side: Side::Bid,
            status: Status::New,
            arr_time: t,
            end_time: Nanos::MAX,
            vol,
            start_vol: vol,
            price: Price::MAX,
            trader_id,
            order_id,
        }
    }

    /// Initialise a sell limit-order
    ///
    /// # Arguments
    ///
    /// - `t` - Order creation time
    /// - `vol` - Order volume
    /// - `price` - Limit price of the order
    /// - `trader_id` - Id of the agent/trader
    /// - `order_id` - Id of the order
    ///
    pub fn sell_limit(
        t: Nanos,
        vol: Vol,
        price: Price,
        trader_id: TraderId,
        order_id: OrderId,
    ) -> Order {
        Order {
            side: Side::Ask,
            status: Status::New,
            arr_time: t,
            end_time: Nanos::MAX,
            vol,
            start_vol: vol,
            price,
            trader_id,
            order_id,
        }
    }

    /// Initialise a sell market-order
    ///
    /// # Arguments
    ///
    /// - `t` - Order creation time
    /// - `vol` - Order volume
    /// - `trader_id` - Id of the agent/trader
    /// - `order_id` - Id of the order
    ///
    pub fn sell_market(t: Nanos, vol: Vol, trader_id: TraderId, order_id: OrderId) -> Order {
        Order {
            side: Side::Ask,
            status: Status::New,
            arr_time: t,
            end_time: Nanos::MAX,
            vol,
            start_vol: vol,
            price: 0,
            trader_id,
            order_id,
        }
    }
}

/// Order/transaction instruction
pub enum Event {
    /// Place an order on the market
    New {
        /// Id of the order to place
        order_id: OrderId,
    },
    /// Cancel an order
    Cancellation {
        /// Id of the order to cancel
        order_id: OrderId,
    },
    /// Modify an order
    Modify {
        // Id of the order to modify
        order_id: OrderId,
        /// New price of the order
        new_price: Option<Price>,
        /// New volume of the order
        new_vol: Option<Vol>,
    },
}

/// Level 1 market data
pub struct Level1Data {
    /// Bid touch price
    pub bid_price: Price,
    /// Ask touch price
    pub ask_price: Price,
    /// Bid total volume
    pub bid_vol: Vol,
    /// Ask total volume
    pub ask_vol: Vol,
    /// Bid touch volume
    pub bid_touch_vol: Vol,
    /// Ask touch volume
    pub ask_touch_vol: Vol,
    /// Number of bid orders at touch
    pub bid_touch_orders: OrderCount,
    /// Number of ask orders at touch
    pub ask_touch_orders: OrderCount,
}

/// Level 2 market data
pub struct Level2Data<const N: usize> {
    /// Bid touch price
    pub bid_price: Price,
    /// Ask touch price
    pub ask_price: Price,
    /// Bid total volume
    pub bid_vol: Vol,
    /// Ask total volume
    pub ask_vol: Vol,
    /// Volume and number of bid orders at price-levels
    pub bid_price_levels: [(Vol, OrderCount); N],
    /// Volume and number of ask orders at price-levels
    pub ask_price_levels: [(Vol, OrderCount); N],
}
