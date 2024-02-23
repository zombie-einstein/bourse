//! Discrete event simulation environment
//!
//! Wraps an order book and provides
//! functionality to process instructions
//! submitted by agents and to track market data
//!
use bourse_book::types::{
    Event, Nanos, Order, OrderCount, OrderId, Price, Side, Trade, TraderId, Vol,
};
use bourse_book::OrderBook;
use fastrand::Rng;
use std::mem;

/// Discrete event simulation environment
///
/// Simulation environment designed for use in a
/// discrete event simulation. Allows agents/users
/// to submit order instructions, update
/// the state of the simulation, and record the
/// market data.
///
/// # Examples
///
/// ```
/// use bourse_de;
/// use bourse_de::types;
/// use fastrand::Rng;
///
/// let mut env = bourse_de::Env::new(0, 1_000, true);
/// let mut rng = Rng::with_seed(101);
///
/// // Submit a new order instruction
/// let order_id = env.place_order(
///     types::Side::Ask,
///     100,
///     101,
///     Some(50),
/// );
///
/// // Update the state of the market
/// env.step(&mut rng)
/// ```
pub struct Env {
    /// Time-length of each simulation step
    step_size: Nanos,
    /// Simulated order book
    order_book: OrderBook,
    /// Bid-ask price history
    prices: (Vec<Price>, Vec<Price>),
    /// Bid-ask volume histories
    volumes: (Vec<Vol>, Vec<Vol>),
    /// Number of touch order histories
    touch_order_counts: (Vec<OrderCount>, Vec<OrderCount>),
    /// Bid-ask touch volume histories
    touch_volumes: (Vec<Vol>, Vec<Vol>),
    /// Per step trade volume histories
    trade_vols: Vec<Vol>,
    /// Transaction queue
    transactions: Vec<Event>,
}

impl Env {
    /// Initialise an empty environment
    ///
    /// # Arguments
    ///
    /// - `start_time` - Simulation start time
    /// - `step_size` - Simulated step time-length
    /// - `trading` - Flag if `true` orders will be matched,
    ///   otherwise no trades will take place
    ///
    pub fn new(start_time: Nanos, step_size: Nanos, trading: bool) -> Self {
        Self {
            step_size,
            order_book: OrderBook::new(start_time, trading),
            prices: (Vec::new(), Vec::new()),
            volumes: (Vec::new(), Vec::new()),
            touch_order_counts: (Vec::new(), Vec::new()),
            touch_volumes: (Vec::new(), Vec::new()),
            trade_vols: Vec::new(),
            transactions: Vec::new(),
        }
    }

    /// Update the state of the simulation
    ///
    /// Each step of the simulation:
    ///
    /// - The cumulative trade volume is reset
    /// - The transaction queue is shuffled
    /// - The transactions are processed, updating
    ///   the state of the market
    /// - Time is jumped forward to the next step
    /// - Market data for the step is recorded
    ///
    /// Note that when each event is processed time
    /// is incremented by 1 time unit (to ensure
    /// orders have a unique index).
    ///
    /// # Arguments
    ///
    /// - `rng` - Fastrand random generator
    ///
    pub fn step(&mut self, rng: &mut Rng) {
        let start_time = self.order_book.get_time();
        self.order_book.reset_trade_vol();

        let mut transactions = mem::take(&mut self.transactions);
        rng.shuffle(transactions.as_mut_slice());

        for (i, t) in transactions.into_iter().enumerate() {
            self.order_book
                .set_time(start_time + Nanos::try_from(i).unwrap());
            self.order_book.process_event(t);
        }

        self.order_book.set_time(start_time + self.step_size);

        let bid_ask = self.order_book.bid_ask();
        self.prices.0.push(bid_ask.0);
        self.prices.1.push(bid_ask.1);
        self.volumes.0.push(self.order_book.bid_vol());
        self.volumes.1.push(self.order_book.ask_vol());

        let (bid_touch_vol, bid_touch_order_count) = self.order_book.bid_best_vol_and_orders();
        self.touch_volumes.0.push(bid_touch_vol);
        self.touch_order_counts.0.push(bid_touch_order_count);

        let (ask_touch_vol, ask_touch_order_count) = self.order_book.ask_best_vol_and_orders();
        self.touch_volumes.1.push(ask_touch_vol);
        self.touch_order_counts.1.push(ask_touch_order_count);

        self.trade_vols.push(self.order_book.get_trade_vol());
    }

    /// Enable trading
    pub fn enable_trading(&mut self) {
        self.order_book.enable_trading();
    }

    /// Disable tradeing
    pub fn disable_trading(&mut self) {
        self.order_book.disable_trading();
    }

    /// Create a new order
    ///
    /// Note that this creates an order but does not
    /// immediately place the order on the market,
    /// rather it submits an instruction to place
    /// the order on the market that will be executed
    /// during the next update.
    ///
    /// Returns the id of the newly create order.
    ///
    /// # Arguments
    ///
    /// - `side` - Side to place order
    /// - `vol` - Volume of the order
    /// - `trader_id` - Id of the trader/agent
    ///   placing the order
    /// - `price` - Order price, if None the
    ///   order will be treated as a market order
    ///
    pub fn place_order(
        &mut self,
        side: Side,
        vol: Vol,
        trader_id: TraderId,
        price: Option<Price>,
    ) -> OrderId {
        let order_id = self.order_book.create_order(side, vol, trader_id, price);
        self.transactions.push(Event::New { order_id });
        order_id
    }

    /// Submit an instruction to cancel an order
    ///
    /// Note that this does not immediately delete
    /// the order but submits an instruction to cancel
    /// the order that will be processed during the
    /// next update
    ///
    /// # Arguments
    ///
    /// - `order_id` - Id of the order to cancel
    ///
    pub fn cancel_order(&mut self, order_id: OrderId) {
        self.transactions.push(Event::Cancellation { order_id })
    }

    /// Submit an instruction to modify an order
    ///
    /// Note that this does not immediately modify
    /// the order but submits an instruction to modify
    /// the order that will be processed during the
    /// next update
    ///
    /// # Arguments
    ///
    /// - `order_id` - Id of the order to modify
    /// - `new_price` - New price of the order,
    ///   if `None` the original price will be kept
    /// - `new_vol` - New volume of the order,
    ///   if `None` the original price will be kept
    ///
    pub fn modify_order(
        &mut self,
        order_id: OrderId,
        new_price: Option<Price>,
        new_vol: Option<Vol>,
    ) {
        self.transactions.push(Event::Modify {
            order_id,
            new_price,
            new_vol,
        })
    }

    /// Get reference to bid-ask price histories
    pub fn get_prices(&self) -> &(Vec<Price>, Vec<Price>) {
        &self.prices
    }

    /// Get bid-ask volume histories
    pub fn get_volumes(&self) -> &(Vec<Vol>, Vec<Vol>) {
        &self.volumes
    }

    /// Get bid-ask touch histories
    pub fn get_touch_volumes(&self) -> &(Vec<Vol>, Vec<Vol>) {
        &self.touch_volumes
    }

    /// Get bid-ask order_count histories
    pub fn get_touch_order_counts(&self) -> &(Vec<OrderCount>, Vec<OrderCount>) {
        &self.touch_order_counts
    }

    /// Get per step trade volume histories
    pub fn get_trade_vols(&self) -> &Vec<Vol> {
        &self.trade_vols
    }

    /// Get references to order data
    pub fn get_orders(&self) -> Vec<&Order> {
        self.order_book.get_orders()
    }

    /// Get reference to the underlying orderbook
    pub fn get_orderbook(&self) -> &OrderBook {
        &self.order_book
    }

    /// Get reference to trade data
    pub fn get_trades(&self) -> &Vec<Trade> {
        self.order_book.get_trades()
    }
}

#[cfg(test)]
mod tests {
    use bourse_book::types::Status;

    use super::*;

    #[test]
    fn test_env() {
        let step_size: Nanos = 1000;
        let mut env = Env::new(0, step_size, true);
        let mut rng = Rng::with_seed(101);

        env.place_order(Side::Bid, 10, 101, Some(10));
        env.place_order(Side::Ask, 20, 101, Some(20));

        env.step(&mut rng);

        assert!(env.transactions.len() == 0);
        assert!(env.get_orderbook().bid_ask() == (10, 20));
        assert!(env.get_orderbook().get_orders().len() == 2);
        assert!(env.get_orderbook().get_orders()[0].status == Status::Active);
        assert!(env.get_orderbook().get_orders()[1].status == Status::Active);
        assert!(env.get_orderbook().get_time() == step_size);

        env.place_order(Side::Bid, 10, 101, Some(11));
        env.place_order(Side::Ask, 20, 101, Some(21));

        env.step(&mut rng);

        assert!(env.get_orderbook().bid_ask() == (11, 20));
        assert!(env.get_orderbook().get_orders().len() == 4);
        assert!(env.get_orderbook().get_time() == 2 * step_size);

        env.place_order(Side::Bid, 30, 101, None);

        env.step(&mut rng);

        assert!(env.get_orderbook().bid_ask() == (11, 21));
        assert!(env.get_orderbook().ask_vol() == 10);
        assert!(env.get_orderbook().get_orders().len() == 5);
        assert!(env.get_orderbook().get_orders()[1].status == Status::Filled);
        assert!(env.get_orderbook().get_orders()[4].status == Status::Filled);
        assert!(env.get_orderbook().get_trades().len() == 2);
        assert!(env.get_orderbook().get_time() == 3 * step_size);

        let prices = env.get_prices();
        assert!(prices.0 == vec![10, 11, 11]);
        assert!(prices.1 == vec![20, 20, 21]);

        let volumes = env.get_volumes();
        assert!(volumes.0 == vec![10, 20, 20]);
        assert!(volumes.1 == vec![20, 40, 10]);

        let touch_volumes = env.get_touch_volumes();
        assert!(touch_volumes.0 == vec![10, 10, 10]);
        assert!(touch_volumes.1 == vec![20, 20, 10]);

        let touch_order_counts = env.get_touch_order_counts();
        assert!(touch_order_counts.0 == vec![1, 1, 1]);
        assert!(touch_order_counts.1 == vec![1, 1, 1]);

        let trade_vols = env.get_trade_vols();
        assert!(*trade_vols == vec![0, 0, 30]);
    }
}
