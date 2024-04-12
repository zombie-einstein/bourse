//! Multi-asset discrete event simulation environment
//!
//! Wraps a [Market] and provides
//! functionality to process instructions
//! submitted by agents and to track market data
//!
use super::data::Level2DataRecords;
use crate::types::{
    AssetIdx, Level2Data, MarketEvent, Nanos, Order, OrderCount, Price, Side, Status, Trade,
    TraderId, Vol,
};
use bourse_book::{types::MarketOrderId, Market, OrderError};
use rand::seq::SliceRandom;
use rand::RngCore;
use std::{array, mem};

/// Multi-asset discrete event simulation environment
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
pub struct MarketEnv<const ASSETS: usize, const LEVELS: usize = 10> {
    /// Time-length of each simulation step
    step_size: Nanos,
    /// Simulated market
    market: Market<ASSETS, LEVELS>,
    /// Per step trade volume histories
    trade_vols: [Vec<Vol>; ASSETS],
    /// Transaction queue
    transactions: Vec<MarketEvent>,
    /// Current level 2 market data
    level_2_data: [Level2Data<LEVELS>; ASSETS],
    /// Level 2 data history
    level_2_data_records: [Level2DataRecords<LEVELS>; ASSETS],
}

impl<const ASSETS: usize, const LEVELS: usize> MarketEnv<ASSETS, LEVELS> {
    /// Initialise an empty market environment
    ///
    /// # Arguments
    ///
    /// - `start_time` - Simulation start time
    /// - `tick_sizes` - Array of market tick sizes per asset
    /// - `step_size` - Simulated step time-length
    /// - `trading` - Flag if `true` orders will be matched,
    ///   otherwise no trades will take place
    ///
    pub fn new(
        start_time: Nanos,
        tick_sizes: [Price; ASSETS],
        step_size: Nanos,
        trading: bool,
    ) -> Self {
        let market = Market::<ASSETS, LEVELS>::new(start_time, tick_sizes, trading);
        let level_2_data = market.level_2_data();
        Self {
            step_size,
            market,
            trade_vols: array::from_fn(|_| Vec::new()),
            transactions: Vec::new(),
            level_2_data,
            level_2_data_records: array::from_fn(|_| Level2DataRecords::new()),
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
        let start_time = self.market.get_time();
        self.market.reset_trade_vols();

        let mut transactions = mem::take(&mut self.transactions);
        transactions.shuffle(rng);

        for (i, t) in transactions.into_iter().enumerate() {
            self.market
                .set_time(start_time + Nanos::try_from(i).unwrap());
            self.market.process_event(t);
        }

        self.market.set_time(start_time + self.step_size);

        self.level_2_data = self.market.level_2_data();
        let trade_vols = self.market.get_trade_vols();

        for (i, tv) in trade_vols.into_iter().enumerate().take(ASSETS) {
            self.level_2_data_records[i].append_record(&self.level_2_data[i]);
            self.trade_vols[i].push(tv);
        }
    }

    /// Enable trading
    pub fn enable_trading(&mut self) {
        self.market.enable_trading();
    }

    /// Disable trading
    pub fn disable_trading(&mut self) {
        self.market.disable_trading();
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
        asset: AssetIdx,
        side: Side,
        vol: Vol,
        trader_id: TraderId,
        price: Option<Price>,
    ) -> Result<MarketOrderId, OrderError> {
        let order_id = self
            .market
            .create_order(asset, side, vol, trader_id, price)?;
        self.transactions.push(MarketEvent::New { order_id });
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
    pub fn cancel_order(&mut self, order_id: MarketOrderId) {
        self.transactions
            .push(MarketEvent::Cancellation { order_id })
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
        order_id: MarketOrderId,
        new_price: Option<Price>,
        new_vol: Option<Vol>,
    ) {
        self.transactions.push(MarketEvent::Modify {
            order_id,
            new_price,
            new_vol,
        })
    }

    /// Get reference to bid-ask price histories of an asset
    ///
    /// # Arguments
    ///
    /// - `asset` - Index of asset
    ///
    pub fn get_prices(&self, asset: AssetIdx) -> &(Vec<Price>, Vec<Price>) {
        &self.level_2_data_records[asset].prices
    }

    /// Get bid-ask volume histories of an asset
    ///
    /// # Arguments
    ///
    /// - `asset` - Index of asset
    ///
    pub fn get_volumes(&self, asset: AssetIdx) -> &(Vec<Vol>, Vec<Vol>) {
        &self.level_2_data_records[asset].volumes
    }

    /// Get bid-ask touch histories of an asset
    ///
    /// # Arguments
    ///
    /// - `asset` - Index of asset
    ///
    pub fn get_touch_volumes(&self, asset: AssetIdx) -> (&Vec<Vol>, &Vec<Vol>) {
        (
            &self.level_2_data_records[asset].volumes_at_levels.0[0],
            &self.level_2_data_records[asset].volumes_at_levels.1[0],
        )
    }

    /// Get bid-ask order_count histories of an asset
    ///
    /// # Arguments
    ///
    /// - `asset` - Index of asset
    ///
    pub fn get_touch_order_counts(&self, asset: AssetIdx) -> (&Vec<OrderCount>, &Vec<OrderCount>) {
        (
            &self.level_2_data_records[asset].orders_at_levels.0[0],
            &self.level_2_data_records[asset].orders_at_levels.1[0],
        )
    }

    /// Get per step trade volume histories of an asset
    ///
    /// # Arguments
    ///
    /// - `asset` - Index of asset
    ///
    pub fn get_trade_vols(&self, asset: AssetIdx) -> &Vec<Vol> {
        &self.trade_vols[asset]
    }

    /// Get references to order data for an asset
    ///
    /// # Arguments
    ///
    /// - `asset` - Index of asset
    ///
    pub fn get_orders(&self, asset: AssetIdx) -> Vec<&Order> {
        self.market.get_orders(asset)
    }

    /// Get a reference to the underlying market
    pub fn get_market(&self) -> &Market<ASSETS, LEVELS> {
        &self.market
    }

    /// Get level 2 data history for an asset
    ///
    /// # Arguments
    ///
    /// - `asset` - Index of asset
    ///
    pub fn get_level_2_data_history(&self, asset: AssetIdx) -> &Level2DataRecords<LEVELS> {
        &self.level_2_data_records[asset]
    }

    /// Get reference to trade data for an asset
    ///
    /// # Arguments
    ///
    /// - `asset` - Index of asset
    ///
    pub fn get_trades(&self, asset: AssetIdx) -> &Vec<Trade> {
        self.market.get_order_book(asset).get_trades()
    }

    /// Get a reference to an order by id
    ///
    /// # Arguments
    ///
    /// - `order_id` - Id of an order
    ///
    pub fn order(&self, order_id: MarketOrderId) -> &Order {
        self.market.order(order_id)
    }

    /// Get the status of an order
    ///
    /// # Arguments
    ///
    /// - `order_id` - Id of an order
    ///
    pub fn order_status(&self, order_id: MarketOrderId) -> Status {
        self.market.order(order_id).status
    }

    /// Reference to current level-2 market data
    pub fn level_2_data(&self) -> &[Level2Data<LEVELS>; ASSETS] {
        &self.level_2_data
    }

    #[cfg(test)]
    pub fn get_transactions(&self) -> &Vec<MarketEvent> {
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
        let mut env: MarketEnv<2> = MarketEnv::new(0, [1, 1], step_size, true);
        let mut rng = Rng::seed_from_u64(101);

        env.place_order(0, Side::Bid, 10, 101, Some(10)).unwrap();
        env.place_order(0, Side::Ask, 20, 101, Some(20)).unwrap();

        env.step(&mut rng);

        assert!(env.transactions.len() == 0);
        assert!(env.get_market().bid_asks() == [(10, 20), (0, Price::MAX)]);
        assert!(env.get_orders(0).len() == 2);
        assert!(env.get_orders(0)[0].status == Status::Active);
        assert!(env.get_orders(0)[1].status == Status::Active);
        assert!(env.get_market().get_time() == step_size);

        env.place_order(0, Side::Bid, 10, 101, Some(11)).unwrap();
        env.place_order(0, Side::Ask, 20, 101, Some(21)).unwrap();

        env.step(&mut rng);

        assert!(env.get_market().bid_asks() == [(11, 20), (0, Price::MAX)]);
        assert!(env.get_orders(0).len() == 4);
        assert!(env.get_market().get_time() == 2 * step_size);

        env.place_order(0, Side::Bid, 30, 101, None).unwrap();

        env.step(&mut rng);

        assert!(env.get_market().bid_asks() == [(11, 21), (0, Price::MAX)]);
        assert!(env.get_market().ask_vols() == [10, 0]);
        assert!(env.get_orders(0).len() == 5);
        assert!(env.get_orders(0)[1].status == Status::Filled);
        assert!(env.get_orders(0)[4].status == Status::Filled);
        assert!(env.get_trades(0).len() == 2);
        assert!(env.get_market().get_time() == 3 * step_size);

        let prices = env.get_prices(0);
        assert!(prices.0 == vec![10, 11, 11]);
        assert!(prices.1 == vec![20, 20, 21]);

        let volumes = env.get_volumes(0);
        assert!(volumes.0 == vec![10, 20, 20]);
        assert!(volumes.1 == vec![20, 40, 10]);

        let touch_volumes = env.get_touch_volumes(0);
        assert!(*touch_volumes.0 == vec![10, 10, 10]);
        assert!(*touch_volumes.1 == vec![20, 20, 10]);

        let touch_order_counts = env.get_touch_order_counts(0);
        assert!(*touch_order_counts.0 == vec![1, 1, 1]);
        assert!(*touch_order_counts.1 == vec![1, 1, 1]);

        let trade_vols = env.get_trade_vols(0);
        assert!(*trade_vols == vec![0, 0, 30]);
    }
}
