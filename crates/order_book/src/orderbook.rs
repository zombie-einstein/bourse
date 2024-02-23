//! Order book implementation
//!
//! # Examples
//!
//! ```
//! use bourse_book;
//! use bourse_book::types;
//!
//! let mut book = bourse_book::OrderBook::new(0, true);
//! let order_id = book.create_order(
//!     types::Side::Bid, 50, 101, Some(50)
//! );
//! book.place_order(order_id);
//! let (bid, ask) = book.bid_ask();
//! book.cancel_order(order_id);
//! ```
//!
use std::cmp::min;

use crate::types::Status;

use super::side::{get_ask_key, get_bid_key, AskSide, BidSide, SideFunctionality};
use super::types::{
    Event, Nanos, Order, OrderCount, OrderId, OrderKey, Price, Side, Trade, TraderId, Vol,
};

/// Order data combined with key
///
/// Orders are linked with a key
/// used to track keys in a price-time
/// priority map used for order matching.
#[derive(Copy, Clone)]
pub struct OrderEntry {
    /// Order data
    order: Order,
    /// Key associated with order
    key: OrderKey,
}

/// Order book with order and trade history
///
/// # Examples
///
/// ```
/// use bourse_book;
/// use bourse_book::types;
///
/// let mut book = bourse_book::OrderBook::new(0, true);
/// let order_id = book.create_order(
///     types::Side::Bid, 50, 101, Some(50)
/// );
/// book.place_order(order_id);
/// let (bid, ask) = book.bid_ask();
/// book.cancel_order(order_id);
/// ```
///
pub struct OrderBook {
    /// Simulated time, intended to represent
    /// nano-seconds, but arbitrary units can
    /// be used without effecting functionality
    t: Nanos,
    /// Cumulative trade volume
    trade_vol: Vol,
    /// Ask side of the book data structure
    ask_side: AskSide,
    /// Bid side of the book data structure
    bid_side: BidSide,
    /// Orders created on the market, once
    /// created orders persist in this vector
    /// with their state updated in-place
    orders: Vec<OrderEntry>,
    /// History of trades
    trades: Vec<Trade>,
    /// Flag if `true` placed orders will be
    /// matched, if `false` no trades will be
    /// executed (but orders can still be
    /// placed and modified)
    trading: bool,
}

impl OrderBook {
    /// Initialise a new orderbook
    ///
    /// Creates a new order empty order book (
    /// i.e. one with 0 volume and 0-MAX_PRICE
    /// prices)
    ///
    /// # Arguments
    ///
    /// - `start_time` - Simulated time to assign to the
    ///   order book
    /// - `trading` - Flag to indicate if trades will be
    ///   executed
    pub fn new(start_time: Nanos, trading: bool) -> Self {
        Self {
            t: start_time,
            trade_vol: 0,
            ask_side: AskSide::new(),
            bid_side: BidSide::new(),
            orders: Vec::new(),
            trades: Vec::new(),
            trading,
        }
    }

    /// Get the order book time
    pub fn get_time(&self) -> Nanos {
        self.t
    }

    /// Manually set the time of the orderbook
    ///
    /// - `t` - Time to set
    pub fn set_time(&mut self, t: Nanos) {
        self.t = t;
    }

    /// Enable trade execution
    pub fn enable_trading(&mut self) {
        self.trading = true;
    }

    /// Disable trade execution
    ///
    /// > **_NOTE:_** Currently there is not
    ///   a un-crossing algorithm implemented
    pub fn disable_trading(&mut self) {
        self.trading = false;
    }

    /// Get the current cumulative trade_volume
    pub fn get_trade_vol(&self) -> Vol {
        self.trade_vol
    }

    /// Reset cumulative trade vol to 0
    pub fn reset_trade_vol(&mut self) {
        self.trade_vol = 0;
    }

    /// Get the current total ask volume
    pub fn ask_vol(&self) -> Vol {
        self.ask_side.vol()
    }

    /// Get the current touch ask volume
    pub fn ask_best_vol(&self) -> Vol {
        self.ask_side.best_vol()
    }

    /// Get the current touch ask volume and order count
    pub fn ask_best_vol_and_orders(&self) -> (Vol, OrderCount) {
        self.ask_side.best_vol_and_orders()
    }

    /// Get the current total bid volume
    pub fn bid_vol(&self) -> Vol {
        self.bid_side.vol()
    }

    /// Get current touch bid volume
    pub fn bid_best_vol(&self) -> Vol {
        self.bid_side.best_vol()
    }

    /// Get the current touch bid volume and order count
    pub fn bid_best_vol_and_orders(&self) -> (Vol, OrderCount) {
        self.bid_side.best_vol_and_orders()
    }

    /// Get current bid-ask price
    pub fn bid_ask(&self) -> (Price, Price) {
        (self.bid_side.best_price(), self.ask_side.best_price())
    }

    /// Get the next order-id in the sequence
    fn current_order_id(&self) -> OrderId {
        self.orders.len()
    }

    /// Get a reference to the order data stored at the id
    ///
    /// # Arguments
    ///
    /// - `order_id` - Id of the order
    ///
    pub fn order(&self, order_id: OrderId) -> &Order {
        &self.orders[order_id].order
    }

    /// Create a new order
    ///
    /// Create a new order in the order list, but
    /// this order is not automatically placed on
    /// the market. Returns the id of the newly
    /// created order.
    ///
    /// # Arguments
    ///
    /// - `side` - Order side
    /// - `vol` - Order volume
    /// - `trader_id` - Id of the trader placing the order
    /// - `price` -  Price of the order, if `None` the
    ///   order is treated as a market order
    ///
    pub fn create_order(
        &mut self,
        side: Side,
        vol: Vol,
        trader_id: TraderId,
        price: Option<Price>,
    ) -> OrderId {
        let order_id = self.current_order_id();

        let order = match (side, price) {
            (Side::Bid, Some(p)) => Order::buy_limit(self.t, vol, p, trader_id, order_id),
            (Side::Bid, None) => Order::buy_market(self.t, vol, trader_id, order_id),
            (Side::Ask, Some(p)) => Order::sell_limit(self.t, vol, p, trader_id, order_id),
            (Side::Ask, None) => Order::sell_market(self.t, vol, trader_id, order_id),
        };

        let key = match side {
            Side::Bid => get_bid_key(0, order.price),
            Side::Ask => get_ask_key(0, order.price),
        };

        self.orders.push(OrderEntry { order, key });

        order_id
    }

    /// Match an aggressive buy order
    ///
    /// # Arguments
    ///
    /// - `order_entry` - Aggressive order details
    ///
    fn match_bid(&mut self, order_entry: &mut OrderEntry) {
        while (order_entry.order.vol > 0) & (order_entry.order.price >= self.ask_side.best_price())
        {
            let next_order_id = self.ask_side.best_order_idx();
            match next_order_id {
                Some(id) => {
                    let match_order = &mut self.orders.get_mut(id).unwrap();
                    let trade_vol = match_orders(
                        self.t,
                        &mut order_entry.order,
                        &mut match_order.order,
                        &mut self.trades,
                    );
                    self.trade_vol += trade_vol;
                    if match_order.order.status == Status::Filled {
                        self.ask_side.remove_order(match_order.key, trade_vol);
                    } else {
                        self.ask_side.remove_vol(match_order.key.1, trade_vol);
                    }
                }
                None => {
                    break;
                }
            }
        }
    }

    /// Match an aggressive sell order
    ///
    /// # Arguments
    ///
    /// - `order_entry` - Aggressive order details
    ///
    fn match_ask(&mut self, order_entry: &mut OrderEntry) {
        while (order_entry.order.vol > 0) & (order_entry.order.price <= self.bid_side.best_price())
        {
            let next_order_id = self.bid_side.best_order_idx();
            match next_order_id {
                Some(id) => {
                    let match_order = &mut self.orders.get_mut(id).unwrap();
                    let trade_vol = match_orders(
                        self.t,
                        &mut order_entry.order,
                        &mut match_order.order,
                        &mut self.trades,
                    );
                    self.trade_vol += trade_vol;
                    if match_order.order.status == Status::Filled {
                        self.bid_side.remove_order(match_order.key, trade_vol);
                    } else {
                        self.bid_side.remove_vol(match_order.key.1, trade_vol);
                    }
                }
                None => {
                    break;
                }
            }
        }
    }

    /// Place a buy limit order on the market
    ///
    /// # Arguments
    ///
    /// - `order_entry` - Order details
    ///
    fn place_bid_limit(&mut self, order_entry: &mut OrderEntry) {
        if self.trading {
            self.match_bid(order_entry);
        }
        if order_entry.order.status != Status::Filled {
            let key: OrderKey = (Side::Bid, order_entry.key.1, self.t);
            order_entry.key = key;
            self.bid_side
                .insert_order(key, order_entry.order.order_id, order_entry.order.vol)
        }
    }

    /// Place a buy market order on the market
    ///
    /// Note that market orders that cannot be completely filled
    /// (for example due to a lack of opposite volume) are not
    /// then placed passively on the book
    ///
    /// # Arguments
    ///
    /// - `order_entry` - Order details
    ///
    fn place_bid_market(&mut self, order_entry: &mut OrderEntry) {
        match self.trading {
            true => {
                self.match_bid(order_entry);
                if order_entry.order.status != Status::Filled {
                    order_entry.order.status = Status::Cancelled;
                    order_entry.order.end_time = self.t;
                }
            }
            false => {
                order_entry.order.status = Status::Rejected;
                order_entry.order.end_time = self.t;
            }
        }
    }

    /// Place a sell limit order on the market
    ///
    /// # Arguments
    ///
    /// - `order_entry` - O
    fn place_ask_limit(&mut self, order_entry: &mut OrderEntry) {
        if self.trading {
            self.match_ask(order_entry);
        }
        if order_entry.order.status != Status::Filled {
            let key: OrderKey = (Side::Ask, order_entry.key.1, self.t);
            order_entry.key = key;
            self.ask_side
                .insert_order(key, order_entry.order.order_id, order_entry.order.vol)
        }
    }

    /// Place a sell market order on the market
    ///
    /// Note that market orders that cannot be completely filled
    /// (for example due to a lack of opposite volume) are not
    /// then placed passively on the book
    ///
    /// # Arguments
    ///
    /// - `order_entry` - Order details
    ///
    fn place_ask_market(&mut self, order_entry: &mut OrderEntry) {
        match self.trading {
            true => {
                self.match_ask(order_entry);
                if order_entry.order.status != Status::Filled {
                    order_entry.order.status = Status::Cancelled;
                    order_entry.order.end_time = self.t;
                }
            }
            false => {
                order_entry.order.status = Status::Rejected;
                order_entry.order.end_time = self.t;
            }
        }
    }

    /// Place an order on the market
    ///
    /// Place an order that has been created on the market
    ///
    /// # Arguments
    ///
    /// - `order_id` - Id of the order to place
    pub fn place_order(&mut self, order_id: OrderId) {
        let mut order_entry = self.orders[order_id];

        order_entry.order.status = Status::Active;
        order_entry.order.arr_time = self.t;

        match order_entry.order.side {
            Side::Bid => {
                if order_entry.order.price == Price::MAX {
                    self.place_bid_market(&mut order_entry)
                } else {
                    self.place_bid_limit(&mut order_entry)
                }
            }
            Side::Ask => {
                if order_entry.order.price == 0 {
                    self.place_ask_market(&mut order_entry)
                } else {
                    self.place_ask_limit(&mut order_entry)
                }
            }
        }

        self.orders[order_id] = order_entry;
    }

    /// Cancel an order
    ///
    /// Attempts to cancel an order, if the order is
    /// already filled or rejected then no change is made
    ///
    /// # Arguments
    ///
    /// - `order_id` - Id of the order to cancel
    ///
    pub fn cancel_order(&mut self, order_id: OrderId) {
        let cancelled_order = self.orders.get_mut(order_id);

        match cancelled_order {
            Some(order_entry) => {
                if order_entry.order.status == Status::Active {
                    order_entry.order.status = Status::Cancelled;
                    order_entry.order.end_time = self.t;
                    match order_entry.key.0 {
                        Side::Bid => {
                            self.bid_side
                                .remove_order(order_entry.key, order_entry.order.vol);
                        }
                        Side::Ask => {
                            self.ask_side
                                .remove_order(order_entry.key, order_entry.order.vol);
                        }
                    }
                }
            }
            None => panic!("No order with id {} exists", order_id),
        }
    }

    /// Reduce order volume
    ///
    /// Reduces the volume of an order in-place
    ///
    /// # Arguments
    ///
    /// - `order_entry` - Order data
    /// - `reduce_vol` - Amount to reduce the available
    ///   volume of the order by
    ///
    fn reduce_order_vol(&mut self, order_entry: &mut OrderEntry, reduce_vol: Vol) {
        match order_entry.key.0 {
            Side::Bid => {
                order_entry.order.vol -= reduce_vol;
                self.bid_side.remove_vol(order_entry.key.1, reduce_vol)
            }
            Side::Ask => {
                order_entry.order.vol -= reduce_vol;
                self.ask_side.remove_vol(order_entry.key.1, reduce_vol)
            }
        }
    }

    /// Replace an order
    ///
    /// Replaces an order with new price and volume
    ///
    /// # Arguments
    ///
    /// - `order_entry` - order data
    /// - `new_price` - New price of the order
    /// - `new_vol` - New volume of the order
    ///
    fn replace_order(&mut self, order_entry: &mut OrderEntry, new_price: Price, new_vol: Vol) {
        match order_entry.key.0 {
            Side::Bid => self
                .bid_side
                .remove_order(order_entry.key, order_entry.order.vol),
            Side::Ask => self
                .ask_side
                .remove_order(order_entry.key, order_entry.order.vol),
        }

        order_entry.order.vol = new_vol;
        order_entry.order.price = new_price;

        if self.trading {
            match order_entry.key.0 {
                Side::Bid => self.match_bid(order_entry),
                Side::Ask => self.match_ask(order_entry),
            }
        }

        if order_entry.order.status != Status::Filled {
            match order_entry.key.0 {
                crate::types::Side::Bid => {
                    let key: OrderKey = get_bid_key(self.t, new_price);
                    order_entry.key = key;

                    self.bid_side.insert_order(
                        key,
                        order_entry.order.order_id,
                        order_entry.order.vol,
                    );
                }
                crate::types::Side::Ask => {
                    let key: OrderKey = get_ask_key(self.t, new_price);
                    order_entry.key = key;

                    self.ask_side.insert_order(
                        key,
                        order_entry.order.order_id,
                        order_entry.order.vol,
                    );
                }
            }
        }
    }

    /// Modify the price and/or volume of an order
    ///
    /// If only the volume is *reduced*, then the order
    /// maintains is price-time priority. Otherwise the
    /// order is replaced. The order modified order
    /// maintains the same id.
    ///
    /// If the price/vol are None then the original
    /// price/vol are kept.
    ///
    /// # Arguments
    ///
    /// - `order_id` - Id of the order to modify
    /// - `new_price` - New price of the order, `None``
    ///   keeps the same price
    /// - `new_vol` - New volume of the order, `None``
    ///   keeps the same volume
    ///
    pub fn modify_order(
        &mut self,
        order_id: OrderId,
        new_price: Option<Price>,
        new_vol: Option<Price>,
    ) {
        let mut order_entry = self.orders[order_id];

        if order_entry.order.status == Status::Active {
            match (new_price, new_vol) {
                (None, None) => (),
                (None, Some(v)) => {
                    if v < order_entry.order.vol {
                        let reduce_vol = order_entry.order.vol - v;
                        self.reduce_order_vol(&mut order_entry, reduce_vol);
                    } else {
                        let p = order_entry.order.price;
                        self.replace_order(&mut order_entry, p, v)
                    }
                }
                (Some(p), None) => {
                    let v = order_entry.order.vol;
                    self.replace_order(&mut order_entry, p, v);
                }
                (Some(p), Some(v)) => self.replace_order(&mut order_entry, p, v),
            }
        }

        self.orders[order_id] = order_entry;
    }

    /// Process an [Event] order instruction
    ///
    /// Processes an order instruction to place, cancel
    /// or modify an order
    ///
    /// # Arguments
    ///
    /// - `event` - Order instruction
    pub fn process_event(&mut self, event: Event) {
        match event {
            Event::New { order_id } => self.place_order(order_id),
            Event::Cancellation { order_id } => self.cancel_order(order_id),
            Event::Modify {
                order_id,
                new_price,
                new_vol,
            } => self.modify_order(order_id, new_price, new_vol),
        }
    }

    /// Reference to list of created orders
    pub fn get_orders(&self) -> Vec<&Order> {
        self.orders.iter().map(|x| &x.order).collect()
    }

    /// Reference to trade records
    pub fn get_trades(&self) -> &Vec<Trade> {
        &self.trades
    }
}

/// Match two orders and record the trade
///
/// # Arguments
///
/// - `agg_order` - Aggressive order data
/// - `pass_order` - Passive order data
/// - `trades` - Trade records
///
fn match_orders(
    t: Nanos,
    agg_order: &mut Order,
    pass_order: &mut Order,
    trades: &mut Vec<Trade>,
) -> Vol {
    let trade_vol = min(agg_order.vol, pass_order.vol);
    agg_order.vol -= trade_vol;
    pass_order.vol -= trade_vol;
    trades.push(Trade {
        t,
        side: pass_order.side,
        price: pass_order.price,
        vol: trade_vol,
        active_order_id: agg_order.order_id,
        passive_order_id: pass_order.order_id,
    });
    if pass_order.vol == 0 {
        pass_order.end_time = t;
        pass_order.status = Status::Filled;
    };
    if agg_order.vol == 0 {
        agg_order.end_time = t;
        agg_order.status = Status::Filled;
    };

    trade_vol
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let book = OrderBook::new(0, true);

        assert!(book.bid_vol() == 0);
        assert!(book.ask_vol() == 0);
        assert!(book.bid_best_vol() == 0);
        assert!(book.bid_best_vol_and_orders() == (0, 0));
        assert!(book.bid_best_vol() == 0);
        assert!(book.ask_best_vol_and_orders() == (0, 0));
        assert!(book.bid_ask() == (0, Price::MAX))
    }

    #[test]
    fn test_insert_order() {
        let mut book = OrderBook::new(0, true);

        book.create_order(Side::Ask, 10, 0, Some(100));
        book.create_order(Side::Bid, 10, 0, Some(50));

        book.place_order(0);
        book.place_order(1);

        assert!(book.bid_ask() == (50, 100));
        assert!(book.ask_vol() == 10);
        assert!(book.bid_vol() == 10);
        assert!(book.bid_best_vol() == 10);
        assert!(book.bid_best_vol_and_orders() == (10, 1));
        assert!(book.ask_best_vol() == 10);
        assert!(book.ask_best_vol_and_orders() == (10, 1));

        book.create_order(Side::Ask, 10, 0, Some(90));
        book.create_order(Side::Bid, 10, 0, Some(60));

        book.place_order(2);
        book.place_order(3);

        assert!(book.bid_ask() == (60, 90));
        assert!(book.ask_vol() == 20);
        assert!(book.bid_vol() == 20);
        assert!(book.bid_best_vol() == 10);
        assert!(book.bid_best_vol_and_orders() == (10, 1));
        assert!(book.ask_best_vol() == 10);
        assert!(book.ask_best_vol_and_orders() == (10, 1));

        book.create_order(Side::Ask, 10, 0, Some(110));
        book.create_order(Side::Bid, 10, 0, Some(40));

        book.place_order(4);
        book.place_order(5);

        assert!(book.bid_ask() == (60, 90));
        assert!(book.ask_vol() == 30);
        assert!(book.bid_vol() == 30);
        assert!(book.bid_best_vol() == 10);
        assert!(book.bid_best_vol_and_orders() == (10, 1));
        assert!(book.ask_best_vol() == 10);
        assert!(book.ask_best_vol_and_orders() == (10, 1));
    }

    #[test]
    fn test_cancel_order() {
        let mut book = OrderBook::new(0, true);

        book.create_order(Side::Ask, 10, 0, Some(100));
        book.create_order(Side::Bid, 10, 0, Some(50));
        book.create_order(Side::Ask, 10, 0, Some(90));
        book.create_order(Side::Bid, 10, 0, Some(60));

        book.place_order(0);
        book.place_order(1);
        book.place_order(2);
        book.place_order(3);

        assert!(book.bid_ask() == (60, 90));
        assert!(book.ask_vol() == 20);
        assert!(book.bid_vol() == 20);
        assert!(book.bid_best_vol() == 10);
        assert!(book.ask_best_vol() == 10);
        assert!(book.bid_best_vol_and_orders() == (10, 1));
        assert!(book.ask_best_vol_and_orders() == (10, 1));

        book.cancel_order(0);
        book.cancel_order(3);

        assert!(book.bid_ask() == (50, 90));
        assert!(book.ask_vol() == 10);
        assert!(book.bid_vol() == 10);
        assert!(book.bid_best_vol() == 10);
        assert!(book.ask_best_vol() == 10);
        assert!(book.bid_best_vol_and_orders() == (10, 1));
        assert!(book.ask_best_vol_and_orders() == (10, 1));

        book.cancel_order(1);
        book.cancel_order(2);

        assert!(book.bid_ask() == (0, Price::MAX));
        assert!(book.ask_vol() == 0);
        assert!(book.bid_vol() == 0);
        assert!(book.bid_best_vol() == 0);
        assert!(book.ask_best_vol() == 0);
        assert!(book.bid_best_vol_and_orders() == (0, 0));
        assert!(book.ask_best_vol_and_orders() == (0, 0));

        matches!(book.orders[0].order.status, Status::Cancelled);
        matches!(book.orders[1].order.status, Status::Cancelled);
        matches!(book.orders[2].order.status, Status::Cancelled);
        matches!(book.orders[3].order.status, Status::Cancelled);
    }

    #[test]
    fn test_mod_order_vol() {
        let mut book = OrderBook::new(0, true);

        book.create_order(Side::Ask, 10, 0, Some(100));
        book.create_order(Side::Bid, 10, 0, Some(50));

        book.place_order(0);
        book.place_order(1);

        book.modify_order(0, None, Some(8));
        book.modify_order(1, None, Some(5));

        assert!(book.ask_vol() == 8);
        assert!(book.ask_best_vol() == 8);
        assert!(book.ask_best_vol_and_orders() == (8, 1));
        assert!(book.bid_vol() == 5);
        assert!(book.bid_best_vol() == 5);
        assert!(book.bid_best_vol_and_orders() == (5, 1));

        assert!(book.orders[0].order.vol == 8);
        assert!(book.orders[1].order.vol == 5);
    }

    #[test]
    fn test_modify_order() {
        let mut book = OrderBook::new(0, true);

        book.create_order(Side::Ask, 10, 0, Some(100));
        book.create_order(Side::Bid, 10, 0, Some(50));

        book.place_order(0);
        book.place_order(1);

        assert!(book.bid_ask() == (50, 100));

        book.modify_order(0, Some(110), Some(15));
        book.modify_order(1, Some(60), Some(20));

        assert!(book.ask_vol() == 15);
        assert!(book.ask_best_vol() == 15);
        assert!(book.bid_vol() == 20);
        assert!(book.bid_best_vol() == 20);
        assert!(book.bid_ask() == (60, 110));
    }

    #[test]
    fn test_modify_order_crossing() {
        let mut book = OrderBook::new(0, true);

        book.create_order(Side::Ask, 10, 0, Some(100));
        book.create_order(Side::Bid, 10, 0, Some(50));

        book.place_order(0);
        book.place_order(1);

        assert!(book.bid_ask() == (50, 100));

        book.modify_order(1, Some(100), Some(20));

        assert!(book.ask_vol() == 0);
        assert!(book.ask_best_vol() == 0);
        assert!(book.ask_best_vol_and_orders() == (0, 0));
        assert!(book.bid_vol() == 10);
        assert!(book.bid_best_vol() == 10);
        assert!(book.bid_best_vol_and_orders() == (10, 1));
        assert!(book.bid_ask() == (100, Price::MAX));

        assert!(book.trades.len() == 1);
        assert!(book.trades[0].price == 100);
        assert!(book.trades[0].vol == 10);
    }

    #[test]
    fn test_trades() {
        let mut book = OrderBook::new(0, true);

        book.create_order(Side::Ask, 101, 101, Some(20));
        book.create_order(Side::Ask, 101, 101, Some(18));
        book.create_order(Side::Bid, 202, 101, Some(12));
        book.create_order(Side::Bid, 202, 101, Some(14));

        book.place_order(0);
        book.set_time(1);
        book.place_order(1);
        book.set_time(2);
        book.place_order(2);
        book.set_time(3);
        book.place_order(3);
        book.set_time(4);

        book.create_order(Side::Bid, 102, 101, None);
        book.place_order(4);

        assert!(book.ask_vol() == 100);
        assert!(book.bid_ask() == (14, 20));

        assert!(book.trades.len() == 2);
        assert!(book.trades[0].price == 18);
        assert!(book.trades[0].vol == 101);
        assert!(book.trades[1].price == 20);
        assert!(book.trades[1].vol == 1);
        assert!(book.get_trade_vol() == 102);

        book.create_order(Side::Ask, 204, 101, Some(14));
        book.place_order(5);

        assert!(book.bid_vol() == 202);
        assert!(book.ask_vol() == 102);
        assert!(book.bid_best_vol_and_orders() == (202, 1));
        assert!(book.ask_best_vol_and_orders() == (2, 1));
        assert!(book.bid_ask() == (12, 14));

        assert!(book.trades.len() == 3);
        assert!(book.trades[2].price == 14);
        assert!(book.trades[2].vol == 202);
        assert!(book.get_trade_vol() == 304);
    }

    #[test]
    fn test_market_order_no_trading() {
        let mut book = OrderBook::new(0, false);

        book.create_order(Side::Bid, 101, 101, None);
        book.place_order(0);

        assert!(book.bid_ask() == (0, Price::MAX));
        assert!(book.bid_vol() == 0);
        assert!(book.ask_vol() == 0);
        assert!(book.orders[0].order.status == Status::Rejected);
    }

    #[test]
    fn test_unfilled_market_order() {
        let mut book = OrderBook::new(0, true);

        book.create_order(Side::Ask, 10, 101, Some(50));
        book.place_order(0);

        book.create_order(Side::Bid, 20, 101, None);
        book.place_order(1);

        assert!(book.bid_ask() == (0, Price::MAX));
        assert!(book.bid_vol() == 0);
        assert!(book.ask_vol() == 0);
        assert!(book.orders[1].order.status == Status::Cancelled);
    }
}
