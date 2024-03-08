//! Order book implementation
//!
//! # Examples
//!
//! ```
//! use bourse_book;
//! use bourse_book::{types, OrderBook};
//!
//! let mut book: OrderBook  = OrderBook::new(0, 1, true);
//! let order_id = book.create_order(
//!     types::Side::Bid, 50, 101, Some(50)
//! ).unwrap();
//! book.place_order(order_id);
//! let (bid, ask) = book.bid_ask();
//! book.cancel_order(order_id);
//! ```
//!
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::fmt;
use std::path::Path;

use super::side::{get_ask_key, get_bid_key, AskSide, BidSide, SideFunctionality};
use super::types::{
    Event, Level1Data, Level2Data, Nanos, Order, OrderCount, OrderId, OrderKey, Price, Side,
    Status, Trade, TraderId, Vol,
};

/// Order data combined with key
///
/// Orders are linked with a key
/// used to track keys in a price-time
/// priority map used for order matching.
#[derive(Copy, Clone, Serialize, Deserialize)]
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
/// use bourse_book::{types, OrderBook};
///
/// let mut book: OrderBook = OrderBook::new(0, 1, true);
/// let order_id = book.create_order(
///     types::Side::Bid, 50, 101, Some(50)
/// ).unwrap();
/// book.place_order(order_id);
/// let (bid, ask) = book.bid_ask();
/// book.cancel_order(order_id);
/// ```
///
#[derive(Serialize, Deserialize)]
#[serde(try_from = "OrderBookState<N>")]
pub struct OrderBook<const N: usize = 10> {
    /// Simulated time, intended to represent
    /// nano-seconds, but arbitrary units can
    /// be used without effecting functionality
    t: Nanos,
    // Market tick size
    tick_size: Price,
    /// Cumulative trade volume
    trade_vol: Vol,
    /// Ask side of the book data structure
    #[serde(skip_serializing)]
    ask_side: AskSide,
    /// Bid side of the book data structure
    #[serde(skip_serializing)]
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

/// Order rejection errors
///
/// Errors raised when error creation fails.
#[derive(Debug, Clone)]
pub enum OrderError {
    /// Price not a multiple of market tick-size
    PriceError { price: Price, tick_size: Price },
}

impl fmt::Display for OrderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OrderError::PriceError { price, tick_size } => write!(
                f,
                "Price {} was not a multiple of tick-size {}",
                price, tick_size
            ),
        }
    }
}

impl<const N: usize> OrderBook<N> {
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
    /// - `tick_size` - Tick size
    /// - `trading` - Flag to indicate if trades will be
    ///   executed
    pub fn new(start_time: Nanos, tick_size: Price, trading: bool) -> Self {
        Self {
            t: start_time,
            tick_size,
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

    /// Get volumes and number of orders at N price levels
    ///
    /// Returns an array of tuples containing the volume and
    /// number of orders at each price level from the touch.
    pub fn ask_levels(&self) -> [(Vol, OrderCount); N] {
        let start = self.bid_ask().1;
        core::array::from_fn(|i| {
            self.ask_side.vol_and_orders_at_price(
                start.wrapping_add(Price::try_from(i).unwrap() * self.tick_size),
            )
        })
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

    /// Get volumes and number of orders at N price levels
    ///
    /// Returns an array of tuples containing the volume and
    /// number of orders at each price level from the touch.
    pub fn bid_levels(&self) -> [(Vol, OrderCount); N] {
        let start = self.bid_ask().0;
        core::array::from_fn(|i| {
            self.bid_side.vol_and_orders_at_price(
                start.wrapping_sub(Price::try_from(i).unwrap() * self.tick_size),
            )
        })
    }

    /// Get current bid-ask price
    pub fn bid_ask(&self) -> (Price, Price) {
        (self.bid_side.best_price(), self.ask_side.best_price())
    }

    /// Get current mid-price (as a float)
    pub fn mid_price(&self) -> f64 {
        let (bid, ask) = self.bid_ask();
        let spread = ask - bid;
        f64::from(bid) + 0.5 * f64::from(spread)
    }

    /// Get current level 1 market data
    ///
    /// Returns level 1 data which includes
    ///
    /// - Best bid and ask prices
    /// - Total bid and ask side volumes
    /// - Bid and ask volumes at the touch
    /// - Number of bid and ask orders at the touch
    ///
    pub fn level_1_data(&self) -> Level1Data {
        let (bid_price, ask_price) = self.bid_ask();
        let (bid_touch_vol, bid_touch_orders) = self.bid_best_vol_and_orders();
        let (ask_touch_vol, ask_touch_orders) = self.ask_best_vol_and_orders();
        Level1Data {
            bid_price,
            ask_price,
            bid_vol: self.bid_vol(),
            ask_vol: self.ask_vol(),
            bid_touch_vol,
            ask_touch_vol,
            bid_touch_orders,
            ask_touch_orders,
        }
    }

    /// Get current level 2 market data
    ///
    /// In this case level 2 data contains
    /// additional order information at a fixed number
    /// of ticks from the best price
    ///
    /// - Best bid and ask prices
    /// - Total bid and ask volumes
    /// - Volume and number of orders at N levels (ticks)
    ///   above/below the bid ask (where N is 10 by default)
    ///
    pub fn level_2_data(&self) -> Level2Data<N> {
        let (bid_price, ask_price) = self.bid_ask();
        Level2Data {
            bid_price,
            ask_price,
            bid_vol: self.bid_vol(),
            ask_vol: self.ask_vol(),
            bid_price_levels: self.bid_levels(),
            ask_price_levels: self.ask_levels(),
        }
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
    ) -> Result<OrderId, OrderError> {
        let order_id = self.current_order_id();

        let order = match (side, price) {
            (Side::Bid, Some(p)) => {
                if p % self.tick_size != 0 {
                    return Err(OrderError::PriceError {
                        price: p,
                        tick_size: self.tick_size,
                    });
                }
                Order::buy_limit(self.t, vol, p, trader_id, order_id)
            }
            (Side::Bid, None) => Order::buy_market(self.t, vol, trader_id, order_id),
            (Side::Ask, Some(p)) => {
                if p % self.tick_size != 0 {
                    return Err(OrderError::PriceError {
                        price: p,
                        tick_size: self.tick_size,
                    });
                }
                Order::sell_limit(self.t, vol, p, trader_id, order_id)
            }
            (Side::Ask, None) => Order::sell_market(self.t, vol, trader_id, order_id),
        };

        let key = match side {
            Side::Bid => get_bid_key(0, order.price),
            Side::Ask => get_ask_key(0, order.price),
        };

        self.orders.push(OrderEntry { order, key });

        Ok(order_id)
    }

    /// Convenience function to create and immediately place an order
    ///
    /// Create a new order in the order list and place it on the market.
    /// Returns the id of the newly created order.
    ///
    /// # Arguments
    ///
    /// - `side` - Order side
    /// - `vol` - Order volume
    /// - `trader_id` - Id of the trader placing the order
    /// - `price` -  Price of the order, if `None` the
    ///   order is treated as a market order
    ///
    pub fn create_and_place_order(
        &mut self,
        side: Side,
        vol: Vol,
        trader_id: TraderId,
        price: Option<Price>,
    ) -> Result<OrderId, OrderError> {
        let order_id = self.create_order(side, vol, trader_id, price)?;
        self.place_order(order_id);
        Ok(order_id)
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

        if order_entry.order.status != Status::New {
            return;
        }

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
    /// maintains its price-time priority. Otherwise the
    /// order is replaced. The modified order
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

    /// Save a snapshot of the order-book to JSON
    ///
    /// # Argument
    ///
    /// - `path` - Path to write snapshot JSON to
    /// - `pretty` - If `True` JSON will be pretty printed
    ///
    pub fn save_json<P: AsRef<Path>>(&self, path: P, pretty: bool) -> std::io::Result<()> {
        let file = std::fs::File::create(path)?;
        let file = std::io::BufWriter::new(file);
        match pretty {
            true => serde_json::to_writer_pretty(file, self)?,
            false => serde_json::to_writer(file, self)?,
        }
        Ok(())
    }

    /// Load an order-book from a JSON snapshot
    ///
    /// # Argument
    ///
    /// - `path` - Path to read snapshot JSON from
    ///
    pub fn load_json<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = std::fs::File::open(path)?;
        let file = std::io::BufReader::new(file);
        let order_book: Self = serde_json::from_reader(file)?;
        Ok(order_book)
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

/// Dummy order book to enable deserialization
#[derive(Deserialize)]
struct OrderBookState<const N: usize = 10> {
    t: Nanos,
    tick_size: Price,
    trade_vol: Vol,
    orders: Vec<OrderEntry>,
    trades: Vec<Trade>,
    trading: bool,
}

struct OrderBookConversionErrror;

impl fmt::Display for OrderBookConversionErrror {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to convert OrderBookState to an OrderBook")
    }
}

impl<const N: usize> std::convert::TryFrom<OrderBookState<N>> for OrderBook<{ N }> {
    type Error = OrderBookConversionErrror;

    fn try_from(state: OrderBookState<N>) -> Result<Self, Self::Error> {
        let mut bid_side = BidSide::default();
        let mut ask_side = AskSide::default();

        for OrderEntry { order, key } in state.orders.iter() {
            if order.status == Status::Active {
                match order.side {
                    Side::Bid => bid_side.insert_order(*key, order.order_id, order.vol),
                    Side::Ask => ask_side.insert_order(*key, order.order_id, order.vol),
                }
            }
        }

        Ok(Self {
            t: state.t,
            tick_size: state.tick_size,
            trade_vol: state.trade_vol,
            ask_side,
            bid_side,
            orders: state.orders,
            trades: state.trades,
            trading: state.trading,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_init() {
        let book: OrderBook = OrderBook::new(0, 1, true);

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
        let mut book: OrderBook = OrderBook::new(0, 1, true);

        book.create_and_place_order(Side::Ask, 10, 0, Some(100))
            .unwrap();
        book.create_and_place_order(Side::Bid, 10, 0, Some(50))
            .unwrap();

        assert!(book.bid_ask() == (50, 100));
        assert!(book.ask_vol() == 10);
        assert!(book.bid_vol() == 10);
        assert!(book.bid_best_vol() == 10);
        assert!(book.bid_best_vol_and_orders() == (10, 1));
        assert!(book.ask_best_vol() == 10);
        assert!(book.ask_best_vol_and_orders() == (10, 1));

        book.create_and_place_order(Side::Ask, 10, 0, Some(90))
            .unwrap();
        book.create_and_place_order(Side::Bid, 10, 0, Some(60))
            .unwrap();

        assert!(book.bid_ask() == (60, 90));
        assert!(book.ask_vol() == 20);
        assert!(book.bid_vol() == 20);
        assert!(book.bid_best_vol() == 10);
        assert!(book.bid_best_vol_and_orders() == (10, 1));
        assert!(book.ask_best_vol() == 10);
        assert!(book.ask_best_vol_and_orders() == (10, 1));

        book.create_and_place_order(Side::Ask, 10, 0, Some(110))
            .unwrap();
        book.create_and_place_order(Side::Bid, 10, 0, Some(40))
            .unwrap();

        assert!(book.bid_ask() == (60, 90));
        assert!(book.ask_vol() == 30);
        assert!(book.bid_vol() == 30);
        assert!(book.bid_best_vol() == 10);
        assert!(book.bid_best_vol_and_orders() == (10, 1));
        assert!(book.ask_best_vol() == 10);
        assert!(book.ask_best_vol_and_orders() == (10, 1));
    }

    #[test]
    fn test_level_data() {
        let mut book = OrderBook::<4>::new(0, 2, true);

        let bid_levels = book.bid_levels();

        assert!(bid_levels.len() == 4);
        assert!(bid_levels == [(0, 0), (0, 0), (0, 0), (0, 0)]);

        let ask_levels = book.ask_levels();

        assert!(ask_levels.len() == 4);
        assert!(ask_levels == [(0, 0), (0, 0), (0, 0), (0, 0)]);

        book.create_and_place_order(Side::Bid, 10, 0, Some(100))
            .unwrap();
        book.create_and_place_order(Side::Bid, 10, 0, Some(100))
            .unwrap();
        book.create_and_place_order(Side::Bid, 12, 0, Some(98))
            .unwrap();
        book.create_and_place_order(Side::Bid, 14, 0, Some(94))
            .unwrap();

        book.create_and_place_order(Side::Ask, 11, 0, Some(102))
            .unwrap();
        book.create_and_place_order(Side::Ask, 11, 0, Some(102))
            .unwrap();
        book.create_and_place_order(Side::Ask, 13, 0, Some(104))
            .unwrap();
        book.create_and_place_order(Side::Ask, 15, 0, Some(108))
            .unwrap();

        let bid_levels = book.bid_levels();

        assert!(bid_levels.len() == 4);
        assert!(bid_levels == [(20, 2), (12, 1), (0, 0), (14, 1)]);

        let ask_levels = book.ask_levels();

        assert!(ask_levels.len() == 4);
        assert!(ask_levels == [(22, 2), (13, 1), (0, 0), (15, 1)]);

        assert!(matches!(
            book.level_1_data(),
            Level1Data {
                bid_price: 100,
                ask_price: 102,
                bid_vol: 46,
                ask_vol: 50,
                bid_touch_vol: 20,
                ask_touch_vol: 22,
                bid_touch_orders: 2,
                ask_touch_orders: 2,
            }
        ));

        assert!(matches!(
            book.level_2_data(),
            Level2Data {
                bid_price: 100,
                ask_price: 102,
                bid_vol: 46,
                ask_vol: 50,
                bid_price_levels: [(20, 2), (12, 1), (0, 0), (14, 1)],
                ask_price_levels: [(22, 2), (13, 1), (0, 0), (15, 1)],
            }
        ));
    }

    #[test]
    fn test_cancel_order() {
        let mut book: OrderBook = OrderBook::new(0, 1, true);

        book.create_and_place_order(Side::Ask, 10, 0, Some(100))
            .unwrap();
        book.create_and_place_order(Side::Bid, 10, 0, Some(50))
            .unwrap();
        book.create_and_place_order(Side::Ask, 10, 0, Some(90))
            .unwrap();
        book.create_and_place_order(Side::Bid, 10, 0, Some(60))
            .unwrap();

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

        assert!(matches!(book.orders[0].order.status, Status::Cancelled));
        assert!(matches!(book.orders[1].order.status, Status::Cancelled));
        assert!(matches!(book.orders[2].order.status, Status::Cancelled));
        assert!(matches!(book.orders[3].order.status, Status::Cancelled));
    }

    #[test]
    fn test_mod_order_vol() {
        let mut book: OrderBook = OrderBook::new(0, 1, true);

        book.create_and_place_order(Side::Ask, 10, 0, Some(100))
            .unwrap();
        book.create_and_place_order(Side::Bid, 10, 0, Some(50))
            .unwrap();

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
        let mut book: OrderBook = OrderBook::new(0, 1, true);

        book.create_and_place_order(Side::Ask, 10, 0, Some(100))
            .unwrap();
        book.create_and_place_order(Side::Bid, 10, 0, Some(50))
            .unwrap();

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
        let mut book: OrderBook = OrderBook::new(0, 1, true);

        book.create_and_place_order(Side::Ask, 10, 0, Some(100))
            .unwrap();
        book.create_and_place_order(Side::Bid, 10, 0, Some(50))
            .unwrap();

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
        let mut book: OrderBook = OrderBook::new(0, 1, true);

        book.create_order(Side::Ask, 101, 101, Some(20)).unwrap();
        book.create_order(Side::Ask, 101, 101, Some(18)).unwrap();
        book.create_order(Side::Bid, 202, 101, Some(12)).unwrap();
        book.create_order(Side::Bid, 202, 101, Some(14)).unwrap();

        book.place_order(0);
        book.set_time(1);
        book.place_order(1);
        book.set_time(2);
        book.place_order(2);
        book.set_time(3);
        book.place_order(3);
        book.set_time(4);

        book.create_order(Side::Bid, 102, 101, None).unwrap();
        book.place_order(4);

        assert!(book.ask_vol() == 100);
        assert!(book.bid_ask() == (14, 20));

        assert!(book.trades.len() == 2);
        assert!(book.trades[0].price == 18);
        assert!(book.trades[0].vol == 101);
        assert!(book.trades[1].price == 20);
        assert!(book.trades[1].vol == 1);
        assert!(book.get_trade_vol() == 102);

        book.create_order(Side::Ask, 204, 101, Some(14)).unwrap();
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
        let mut book: OrderBook = OrderBook::new(0, 1, false);

        book.create_and_place_order(Side::Bid, 101, 101, None)
            .unwrap();

        assert!(book.bid_ask() == (0, Price::MAX));
        assert!(book.bid_vol() == 0);
        assert!(book.ask_vol() == 0);
        assert!(book.orders[0].order.status == Status::Rejected);
    }

    #[test]
    fn test_unfilled_market_order() {
        let mut book: OrderBook = OrderBook::new(0, 1, true);

        book.create_and_place_order(Side::Ask, 10, 101, Some(50))
            .unwrap();
        book.create_and_place_order(Side::Bid, 20, 101, None)
            .unwrap();

        assert!(book.bid_ask() == (0, Price::MAX));
        assert!(book.bid_vol() == 0);
        assert!(book.ask_vol() == 0);
        assert!(book.orders[1].order.status == Status::Cancelled);
    }

    #[test]
    fn test_incorrect_price_err() {
        let mut book: OrderBook = OrderBook::new(0, 2, true);

        let res = book.create_order(Side::Ask, 100, 101, Some(51));

        assert!(res.is_err_and(|e| matches!(
            e,
            OrderError::PriceError {
                price: 51,
                tick_size: 2
            }
        )));
    }

    #[test]
    fn test_no_double_place() {
        let mut book: OrderBook = OrderBook::new(0, 2, true);

        let id = book.create_order(Side::Ask, 100, 101, Some(50)).unwrap();

        book.place_order(id);

        assert!(book.bid_ask() == (0, 50));
        assert!(book.ask_best_vol_and_orders() == (100, 1));

        book.place_order(id);

        assert!(book.bid_ask() == (0, 50));
        assert!(book.ask_best_vol_and_orders() == (100, 1));
    }

    #[test]
    fn test_serialisation() {
        use rand::{seq::SliceRandom, Rng};
        use rand_xoshiro::rand_core::SeedableRng;
        use rand_xoshiro::Xoroshiro128Plus;

        let mut book: OrderBook = OrderBook::new(0, 1, true);

        let mut rng = Xoroshiro128Plus::seed_from_u64(101);

        for i in (0..200).into_iter() {
            let side = [Side::Bid, Side::Ask].choose(&mut rng).unwrap();
            let price = rng.gen_range(20..40);
            let vol = rng.gen_range(5..20);
            book.create_and_place_order(*side, vol, 0, Some(price))
                .unwrap();
            book.set_time(i);
        }

        let book_snapshot = serde_json::to_string(&book).unwrap();
        let loaded_book = serde_json::from_str::<OrderBook>(book_snapshot.as_str()).unwrap();

        assert!(book.trading == loaded_book.trading);
        assert!(book.trade_vol == loaded_book.trade_vol);

        assert!(book.bid_ask() == loaded_book.bid_ask());

        assert!(book.bid_best_vol_and_orders() == loaded_book.bid_best_vol_and_orders());
        assert!(book.bid_vol() == loaded_book.bid_vol());

        assert!(book.ask_best_vol_and_orders() == loaded_book.ask_best_vol_and_orders());
        assert!(book.bid_vol() == loaded_book.bid_vol());

        assert!(book.current_order_id() == loaded_book.current_order_id());

        assert!(book.bid_side.best_order_idx() == loaded_book.bid_side.best_order_idx());
        assert!(book.ask_side.best_order_idx() == loaded_book.ask_side.best_order_idx());
    }
}
