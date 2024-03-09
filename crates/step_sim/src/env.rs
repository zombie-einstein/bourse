//! Discrete event simulation environment
//!
//! Wraps an order book and provides
//! functionality to process instructions
//! submitted by agents and to track market data
//!
use crate::types::{
    Event, Level2Data, Nanos, Order, OrderCount, OrderId, Price, Side, Status, Trade, TraderId, Vol,
};
use bourse_book::{OrderBook, OrderError};
use rand::seq::SliceRandom;
use rand::RngCore;
use std::{array, mem};

/// Market data history recording
///
/// History of level 2 data over the course of
/// the existence of this environment.
pub struct Level2DataRecords<const N: usize> {
    /// Touch price histories
    pub prices: (Vec<Price>, Vec<Price>),
    /// Bid-ask volume histories
    pub volumes: (Vec<Vol>, Vec<Vol>),
    /// Volumes at price levels
    pub volumes_at_levels: ([Vec<Vol>; N], [Vec<Vol>; N]),
    /// numbers of orders at price levels
    pub orders_at_levels: ([Vec<OrderCount>; N], [Vec<OrderCount>; N]),
}

impl<const N: usize> Level2DataRecords<N> {
    /// Initialise an empty set of records
    fn new() -> Self {
        Self {
            prices: (Vec::new(), Vec::new()),
            volumes: (Vec::new(), Vec::new()),
            volumes_at_levels: (
                array::from_fn(|_| Vec::new()),
                array::from_fn(|_| Vec::new()),
            ),
            orders_at_levels: (
                array::from_fn(|_| Vec::new()),
                array::from_fn(|_| Vec::new()),
            ),
        }
    }

    /// Append a record to the history
    fn append_record(&mut self, record: &Level2Data<N>) {
        self.prices.0.push(record.bid_price);
        self.prices.1.push(record.ask_price);
        self.volumes.0.push(record.bid_vol);
        self.volumes.1.push(record.ask_vol);
        for i in 0..N {
            self.volumes_at_levels.0[i].push(record.bid_price_levels[i].0);
            self.orders_at_levels.0[i].push(record.bid_price_levels[i].1);

            self.volumes_at_levels.1[i].push(record.ask_price_levels[i].0);
            self.orders_at_levels.1[i].push(record.ask_price_levels[i].1);
        }
    }
}

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
/// use bourse_de::{types, Env};
/// use rand_xoshiro::Xoroshiro128StarStar;
/// use rand_xoshiro::rand_core::SeedableRng;
///
/// let mut env: Env = Env::new(0, 1, 1_000, true);
/// let mut rng = Xoroshiro128StarStar::seed_from_u64(101);
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
pub struct Env<const N: usize = 10> {
    /// Time-length of each simulation step
    step_size: Nanos,
    /// Simulated order book
    order_book: OrderBook<N>,
    /// Per step trade volume histories
    trade_vols: Vec<Vol>,
    /// Transaction queue
    transactions: Vec<Event>,
    /// Current level 2 market data
    level_2_data: Level2Data<N>,
    /// Level 2 data history
    level_2_data_records: Level2DataRecords<N>,
}

impl<const N: usize> Env<N> {
    /// Number of price levels recorded during simulation
    pub const N_LEVELS: usize = N;

    /// Initialise an empty environment
    ///
    /// # Arguments
    ///
    /// - `start_time` - Simulation start time
    /// - `tick_size` - Market tick size
    /// - `step_size` - Simulated step time-length
    /// - `trading` - Flag if `true` orders will be matched,
    ///   otherwise no trades will take place
    ///
    pub fn new(start_time: Nanos, tick_size: Price, step_size: Nanos, trading: bool) -> Self {
        let order_book = OrderBook::new(start_time, tick_size, trading);
        let level_2_data = order_book.level_2_data();
        Self {
            step_size,
            order_book,
            trade_vols: Vec::new(),
            transactions: Vec::new(),
            level_2_data,
            level_2_data_records: Level2DataRecords::new(),
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
    /// - `rng` - Random generator
    ///
    pub fn step<R: RngCore>(&mut self, rng: &mut R) {
        let start_time = self.order_book.get_time();
        self.order_book.reset_trade_vol();

        let mut transactions = mem::take(&mut self.transactions);
        transactions.shuffle(rng);

        for (i, t) in transactions.into_iter().enumerate() {
            self.order_book
                .set_time(start_time + Nanos::try_from(i).unwrap());
            self.order_book.process_event(t);
        }

        self.order_book.set_time(start_time + self.step_size);

        // Update data records
        self.level_2_data = self.order_book.level_2_data();
        self.level_2_data_records.append_record(&self.level_2_data);
        self.trade_vols.push(self.order_book.get_trade_vol());
    }

    /// Enable trading
    pub fn enable_trading(&mut self) {
        self.order_book.enable_trading();
    }

    /// Disable trading
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
    ) -> Result<OrderId, OrderError> {
        let order_id = self.order_book.create_order(side, vol, trader_id, price)?;
        self.transactions.push(Event::New { order_id });
        Ok(order_id)
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
        &self.level_2_data_records.prices
    }

    /// Get bid-ask volume histories
    pub fn get_volumes(&self) -> &(Vec<Vol>, Vec<Vol>) {
        &self.level_2_data_records.volumes
    }

    /// Get bid-ask touch histories
    pub fn get_touch_volumes(&self) -> (&Vec<Vol>, &Vec<Vol>) {
        (
            &self.level_2_data_records.volumes_at_levels.0[0],
            &self.level_2_data_records.volumes_at_levels.1[0],
        )
    }

    /// Get bid-ask order_count histories
    pub fn get_touch_order_counts(&self) -> (&Vec<OrderCount>, &Vec<OrderCount>) {
        (
            &self.level_2_data_records.orders_at_levels.0[0],
            &self.level_2_data_records.orders_at_levels.1[0],
        )
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
    pub fn get_orderbook(&self) -> &OrderBook<N> {
        &self.order_book
    }

    /// Get level 2 data history
    pub fn get_level_2_data_history(&self) -> &Level2DataRecords<N> {
        &self.level_2_data_records
    }

    /// Get reference to trade data
    pub fn get_trades(&self) -> &Vec<Trade> {
        self.order_book.get_trades()
    }

    /// Get a reference to an order by id
    ///
    /// # Arguments
    ///
    /// - `order_id` - Id of an order
    ///
    pub fn order(&self, order_id: OrderId) -> &Order {
        self.order_book.order(order_id)
    }

    /// Get the status of an order
    ///
    /// # Arguments
    ///
    /// - `order_id` - Id of an order
    ///
    pub fn order_status(&self, order_id: OrderId) -> Status {
        self.order_book.order(order_id).status
    }

    /// Reference to current level-2 market data
    pub fn level_2_data(&self) -> &Level2Data<N> {
        &self.level_2_data
    }

    #[cfg(test)]
    pub fn get_transactions(&self) -> &Vec<Event> {
        &self.transactions
    }
}

#[cfg(test)]
mod tests {
    use bourse_book::types::Status;
    use rand_xoshiro::rand_core::SeedableRng;
    use rand_xoshiro::Xoroshiro128StarStar as Rng;

    use super::*;

    #[test]
    fn test_env() {
        let step_size: Nanos = 1000;
        let mut env: Env = Env::new(0, 1, step_size, true);
        let mut rng = Rng::seed_from_u64(101);

        env.place_order(Side::Bid, 10, 101, Some(10)).unwrap();
        env.place_order(Side::Ask, 20, 101, Some(20)).unwrap();

        env.step(&mut rng);

        assert!(env.transactions.len() == 0);
        assert!(env.get_orderbook().bid_ask() == (10, 20));
        assert!(env.get_orderbook().get_orders().len() == 2);
        assert!(env.get_orderbook().get_orders()[0].status == Status::Active);
        assert!(env.get_orderbook().get_orders()[1].status == Status::Active);
        assert!(env.get_orderbook().get_time() == step_size);

        env.place_order(Side::Bid, 10, 101, Some(11)).unwrap();
        env.place_order(Side::Ask, 20, 101, Some(21)).unwrap();

        env.step(&mut rng);

        assert!(env.get_orderbook().bid_ask() == (11, 20));
        assert!(env.get_orderbook().get_orders().len() == 4);
        assert!(env.get_orderbook().get_time() == 2 * step_size);

        env.place_order(Side::Bid, 30, 101, None).unwrap();

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
        assert!(*touch_volumes.0 == vec![10, 10, 10]);
        assert!(*touch_volumes.1 == vec![20, 20, 10]);

        let touch_order_counts = env.get_touch_order_counts();
        assert!(*touch_order_counts.0 == vec![1, 1, 1]);
        assert!(*touch_order_counts.1 == vec![1, 1, 1]);

        let trade_vols = env.get_trade_vols();
        assert!(*trade_vols == vec![0, 0, 30]);
    }
}
