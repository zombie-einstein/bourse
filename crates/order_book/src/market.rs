//! Multi asset market combining several order-books
//!
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::{array, path::Path};

use super::{OrderBook, OrderError};
use crate::types::{
    AssetIdx, Event, MarketEvent, MarketOrderId, Nanos, Order, OrderCount, Price, Side, TraderId,
    Vol,
};

/// Multi asset market combining several [OrderBook]
///
/// Market combining several order-books allowing
/// orders and instructions to be placed for multiple
/// assets. Market data across all assets can then
/// be retrieved as an array of values.
///
/// For efficiency assets are indexed  by an
/// [AssetIdx] (as opposed to a ticker). Order ids
/// are then also extended with an asset index,
/// defined by [MarketOrderId].
///
/// A Market type is parametrised by 2 constants:
///
/// - `N` - The number of assets
/// - `M` - The number of price levels tracked
///   by each order-book (default is 10)
///
/// # Examples
///
/// ```
/// use bourse_book::{types, Market};
///
/// // Initialise a market with 4 assets
/// let mut market = Market::<4>::new(0, [1, 1, 1, 1], true);
///
/// // Place a buy order for asset 0
/// market.create_and_place_order(
///     0, types::Side::Bid, 10, 0, Some(100)
/// ).unwrap();
///
/// // Place a sell order for asset 2
/// market.create_and_place_order(
///     2, types::Side::Ask, 10, 0, Some(100)
/// ).unwrap();
///
/// // Get current prices across assets
/// let prices = market.bid_asks();
///
/// // Individual order-books can be accessed by index
/// // for example to retrieve trade data
/// let trades_0 = market.get_order_book(0).get_trades();
/// ```
///
#[serde_as]
#[derive(Deserialize, Serialize)]
pub struct Market<const N: usize, const M: usize = 10> {
    #[serde_as(as = "[_; N]")]
    order_books: [OrderBook<M>; N],
}

impl<const N: usize, const M: usize> Market<N, M> {
    /// Initialise a market
    ///
    /// # Arguments
    ///
    /// - `start_time` - Initial time to assign to the market
    ///   and contained order-books.
    /// - `tick_size` - Array of integer tick sizes for each asset.
    /// - `trading` - If `False` no orders will be matched.
    ///
    pub fn new(start_time: Nanos, tick_size: [Price; N], trading: bool) -> Self {
        Self {
            order_books: array::from_fn(|i| OrderBook::<M>::new(start_time, tick_size[i], trading)),
        }
    }

    /// Get a reference to an orderbook
    ///
    /// # Arguments
    ///
    /// - `asset` - Index of the asset
    ///
    pub fn get_order_book(&self, asset: AssetIdx) -> &OrderBook<M> {
        &self.order_books[asset]
    }

    /// Get a mutable reference to an orderbook
    ///
    /// # Arguments
    ///
    /// - `asset` - Index of the asset
    ///
    pub fn get_order_book_mut(&mut self, asset: AssetIdx) -> &mut OrderBook<M> {
        &mut self.order_books[asset]
    }

    /// Get the market time
    pub fn get_time(&self) -> Nanos {
        self.order_books[0].get_time()
    }

    /// Manually set the time of the market
    ///
    /// # Arguments
    ///
    /// - `t` - Time to set
    ///
    pub fn set_time(&mut self, t: Nanos) {
        for book in self.order_books.iter_mut() {
            book.set_time(t)
        }
    }

    /// Enable trade execution for all assets
    pub fn enable_trading(&mut self) {
        for book in self.order_books.iter_mut() {
            book.enable_trading()
        }
    }

    /// Disable trade execution for all assets
    ///
    /// > **_NOTE:_** Currently there is not
    ///   a un-crossing algorithm implemented
    pub fn disable_trading(&mut self) {
        for book in self.order_books.iter_mut() {
            book.disable_trading()
        }
    }

    /// Get the current cumulative trade_volume across assets
    pub fn get_trade_vols(&self) -> [Vol; N] {
        array::from_fn(|i| self.order_books[i].get_trade_vol())
    }

    /// Reset cumulative trade vol to 0 for all assets
    pub fn reset_trade_vols(&mut self) {
        for book in self.order_books.iter_mut() {
            book.reset_trade_vol();
        }
    }

    /// Get the current total ask volume for all assets
    pub fn bid_vols(&self) -> [Vol; N] {
        array::from_fn(|i| self.order_books[i].bid_vol())
    }

    /// Get the current touch ask volume for all assets
    pub fn bid_best_vols(&self) -> [Vol; N] {
        array::from_fn(|i| self.order_books[i].bid_best_vol())
    }

    /// Get the current touch ask volume and order count for all assets
    pub fn bid_best_vol_and_orders(&self) -> [(Vol, OrderCount); N] {
        array::from_fn(|i| self.order_books[i].bid_best_vol_and_orders())
    }

    /// Get 2d array of orders and volumes at top levels across assets
    pub fn bid_levels(&self) -> [[(Vol, OrderCount); M]; N] {
        array::from_fn(|i| self.order_books[i].bid_levels())
    }

    /// Get the current total ask volume for all assets
    pub fn ask_vols(&self) -> [Vol; N] {
        array::from_fn(|i| self.order_books[i].ask_vol())
    }

    /// Get the current touch ask volume for all assets
    pub fn ask_best_vols(&self) -> [Vol; N] {
        array::from_fn(|i| self.order_books[i].ask_best_vol())
    }

    /// Get the current touch ask volume and order count for all assets
    pub fn ask_best_vol_and_orders(&self) -> [(Vol, OrderCount); N] {
        array::from_fn(|i| self.order_books[i].ask_best_vol_and_orders())
    }

    /// Get 2d array of orders and volumes at top levels across assets
    pub fn ask_levels(&self) -> [[(Vol, OrderCount); M]; N] {
        array::from_fn(|i| self.order_books[i].ask_levels())
    }

    /// Get current bid-ask price for all assets
    pub fn bid_asks(&self) -> [(Price, Price); N] {
        array::from_fn(|i| self.order_books[i].bid_ask())
    }

    /// Get a reference to the order data stored at the id
    ///
    /// # Arguments
    ///
    /// - `order_id` - Asset index and id of the order
    ///
    pub fn order(&self, order_id: MarketOrderId) -> &Order {
        self.order_books[order_id.0].order(order_id.1)
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
    /// - `asset` - Id of the asset to create the order for
    /// - `side` - Order side
    /// - `vol` - Order volume
    /// - `trader_id` - Id of the trader placing the order
    /// - `price` -  Price of the order, if `None` the
    ///   order is treated as a market order
    ///
    pub fn create_order(
        &mut self,
        asset: AssetIdx,
        side: Side,
        vol: Vol,
        trader_id: TraderId,
        price: Option<Price>,
    ) -> Result<MarketOrderId, OrderError> {
        let id = self.order_books[asset].create_order(side, vol, trader_id, price)?;
        Ok((asset, id))
    }

    /// Convenience function to create and immediately place an order
    ///
    /// Create a new order in the order list and place it on the market.
    /// Returns the id of the newly created order.
    ///
    /// # Arguments
    ///
    /// - `asset` - Id of the asset to create and place the order for
    /// - `side` - Order side
    /// - `vol` - Order volume
    /// - `trader_id` - Id of the trader placing the order
    /// - `price` -  Price of the order, if `None` the
    ///   order is treated as a market order
    ///
    pub fn create_and_place_order(
        &mut self,
        asset: AssetIdx,
        side: Side,
        vol: Vol,
        trader_id: TraderId,
        price: Option<Price>,
    ) -> Result<MarketOrderId, OrderError> {
        let id = self.order_books[asset].create_and_place_order(side, vol, trader_id, price)?;
        Ok((asset, id))
    }

    /// Place an order on the market
    ///
    /// Place an order that has been created on the market
    ///
    /// # Arguments
    ///
    /// - `order_id` - Asset index and id of the order to place
    ///
    pub fn place_order(&mut self, order_id: MarketOrderId) {
        self.order_books[order_id.0].place_order(order_id.1)
    }

    /// Cancel an order
    ///
    /// Attempts to cancel an order, if the order is
    /// already filled or rejected then no change is made
    ///
    /// # Arguments
    ///
    /// - `order_id` - Asset index and id of the order to place
    ///
    pub fn cancel_order(&mut self, order_id: MarketOrderId) {
        self.order_books[order_id.0].cancel_order(order_id.1)
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
    /// - `order_id` - Asset index and id of the order to place
    /// - `new_price` - New price of the order, `None``
    ///   keeps the same price
    /// - `new_vol` - New volume of the order, `None``
    ///   keeps the same volume
    ///
    pub fn modify_order(
        &mut self,
        order_id: MarketOrderId,
        new_price: Option<Price>,
        new_vol: Option<Price>,
    ) {
        self.order_books[order_id.0].modify_order(order_id.1, new_price, new_vol)
    }

    /// Process an [MarketEvent] order instruction
    ///
    /// Processes an order instruction to place, cancel
    /// or modify an order
    ///
    /// # Arguments
    ///
    /// - `event` - Order instruction with asset id
    ///
    pub fn process_event(&mut self, event: MarketEvent) {
        let (asset, event) = (event.asset, event.event);
        match event {
            Event::New { order_id } => self.place_order((asset, order_id)),
            Event::Cancellation { order_id } => self.cancel_order((asset, order_id)),
            Event::Modify {
                order_id,
                new_price,
                new_vol,
            } => self.modify_order((asset, order_id), new_price, new_vol),
        }
    }

    /// Save a snapshot of the market to JSON
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

    /// Load a market from a JSON snapshot
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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::types::Status;

    #[test]
    fn test_init() {
        let market: Market<2> = Market::new(101, [1, 2], true);

        assert!(market.get_time() == 101);
        assert!(market.bid_vols() == [0, 0]);
        assert!(market.ask_vols() == [0, 0]);
        assert!(market.bid_best_vols() == [0, 0]);
        assert!(market.bid_best_vol_and_orders() == [(0, 0), (0, 0)]);
        assert!(market.bid_best_vols() == [0, 0]);
        assert!(market.ask_best_vol_and_orders() == [(0, 0), (0, 0)]);
        assert!(market.bid_asks() == [(0, Price::MAX), (0, Price::MAX)]);
    }

    #[test]
    fn test_insert_order() {
        let mut market: Market<2> = Market::new(101, [1, 2], true);

        market
            .create_and_place_order(0, Side::Ask, 10, 0, Some(100))
            .unwrap();
        market
            .create_and_place_order(0, Side::Bid, 10, 0, Some(50))
            .unwrap();

        assert!(market.bid_asks() == [(50, 100), (0, Price::MAX)]);
        assert!(market.ask_vols() == [10, 0]);
        assert!(market.bid_vols() == [10, 0]);
        assert!(market.bid_best_vols() == [10, 0]);
        assert!(market.bid_best_vol_and_orders() == [(10, 1), (0, 0)]);
        assert!(market.ask_best_vols() == [10, 0]);
        assert!(market.ask_best_vol_and_orders() == [(10, 1), (0, 0)]);

        market
            .create_and_place_order(1, Side::Ask, 20, 0, Some(20))
            .unwrap();
        market
            .create_and_place_order(1, Side::Bid, 20, 0, Some(10))
            .unwrap();

        assert!(market.bid_asks() == [(50, 100), (10, 20)]);
        assert!(market.ask_vols() == [10, 20]);
        assert!(market.bid_vols() == [10, 20]);
        assert!(market.bid_best_vols() == [10, 20]);
        assert!(market.bid_best_vol_and_orders() == [(10, 1), (20, 1)]);
        assert!(market.ask_best_vols() == [10, 20]);
        assert!(market.ask_best_vol_and_orders() == [(10, 1), (20, 1)]);

        market
            .create_and_place_order(0, Side::Ask, 10, 0, Some(90))
            .unwrap();
        market
            .create_and_place_order(0, Side::Bid, 10, 0, Some(60))
            .unwrap();

        assert!(market.bid_asks() == [(60, 90), (10, 20)]);
        assert!(market.ask_vols() == [20, 20]);
        assert!(market.bid_vols() == [20, 20]);
        assert!(market.bid_best_vols() == [10, 20]);
        assert!(market.bid_best_vol_and_orders() == [(10, 1), (20, 1)]);
        assert!(market.ask_best_vols() == [10, 20]);
        assert!(market.ask_best_vol_and_orders() == [(10, 1), (20, 1)]);

        market
            .create_and_place_order(1, Side::Ask, 10, 0, Some(20))
            .unwrap();
        market
            .create_and_place_order(1, Side::Bid, 10, 0, Some(12))
            .unwrap();

        assert!(market.bid_asks() == [(60, 90), (12, 20)]);
        assert!(market.ask_vols() == [20, 30]);
        assert!(market.bid_vols() == [20, 30]);
        assert!(market.bid_best_vols() == [10, 10]);
        assert!(market.bid_best_vol_and_orders() == [(10, 1), (10, 1)]);
        assert!(market.ask_best_vols() == [10, 30]);
        assert!(market.ask_best_vol_and_orders() == [(10, 1), (30, 2)]);

        market
            .create_and_place_order(0, Side::Ask, 10, 0, Some(110))
            .unwrap();
        market
            .create_and_place_order(0, Side::Bid, 10, 0, Some(40))
            .unwrap();

        assert!(market.bid_asks() == [(60, 90), (12, 20)]);
        assert!(market.ask_vols() == [30, 30]);
        assert!(market.bid_vols() == [30, 30]);
        assert!(market.bid_best_vols() == [10, 10]);
        assert!(market.bid_best_vol_and_orders() == [(10, 1), (10, 1)]);
        assert!(market.ask_best_vols() == [10, 30]);
        assert!(market.ask_best_vol_and_orders() == [(10, 1), (30, 2)]);
    }

    #[test]
    fn test_cancel_order() {
        let mut market: Market<2> = Market::new(0, [1, 2], true);

        market
            .create_and_place_order(0, Side::Ask, 10, 0, Some(100))
            .unwrap();
        market
            .create_and_place_order(0, Side::Bid, 10, 0, Some(50))
            .unwrap();
        market
            .create_and_place_order(0, Side::Ask, 10, 0, Some(90))
            .unwrap();
        market
            .create_and_place_order(0, Side::Bid, 10, 0, Some(60))
            .unwrap();

        market
            .create_and_place_order(1, Side::Ask, 50, 0, Some(20))
            .unwrap();
        market
            .create_and_place_order(1, Side::Bid, 50, 0, Some(10))
            .unwrap();

        assert!(market.bid_asks() == [(60, 90), (10, 20)]);
        assert!(market.ask_vols() == [20, 50]);
        assert!(market.bid_vols() == [20, 50]);
        assert!(market.bid_best_vols() == [10, 50]);
        assert!(market.ask_best_vols() == [10, 50]);
        assert!(market.bid_best_vol_and_orders() == [(10, 1), (50, 1)]);
        assert!(market.ask_best_vol_and_orders() == [(10, 1), (50, 1)]);

        market.cancel_order((0, 0));
        market.cancel_order((0, 3));

        assert!(market.bid_asks() == [(50, 90), (10, 20)]);
        assert!(market.ask_vols() == [10, 50]);
        assert!(market.bid_vols() == [10, 50]);
        assert!(market.bid_best_vols() == [10, 50]);
        assert!(market.ask_best_vols() == [10, 50]);
        assert!(market.bid_best_vol_and_orders() == [(10, 1), (50, 1)]);
        assert!(market.ask_best_vol_and_orders() == [(10, 1), (50, 1)]);

        market.cancel_order((0, 1));
        market.cancel_order((0, 2));

        assert!(market.bid_asks() == [(0, Price::MAX), (10, 20)]);
        assert!(market.ask_vols() == [0, 50]);
        assert!(market.bid_vols() == [0, 50]);
        assert!(market.bid_best_vols() == [0, 50]);
        assert!(market.ask_best_vols() == [0, 50]);
        assert!(market.bid_best_vol_and_orders() == [(0, 0), (50, 1)]);
        assert!(market.ask_best_vol_and_orders() == [(0, 0), (50, 1)]);

        assert!(matches!(market.order((0, 0)).status, Status::Cancelled));
        assert!(matches!(market.order((0, 1)).status, Status::Cancelled));
        assert!(matches!(market.order((0, 2)).status, Status::Cancelled));
        assert!(matches!(market.order((0, 3)).status, Status::Cancelled));
    }

    #[test]
    fn test_mod_order_vol() {
        let mut market: Market<2> = Market::new(0, [1, 2], true);

        market
            .create_and_place_order(0, Side::Ask, 10, 0, Some(100))
            .unwrap();
        market
            .create_and_place_order(0, Side::Bid, 10, 0, Some(50))
            .unwrap();

        market.modify_order((0, 0), None, Some(8));
        market.modify_order((0, 1), None, Some(5));

        assert!(market.ask_vols() == [8, 0]);
        assert!(market.ask_best_vols() == [8, 0]);
        assert!(market.ask_best_vol_and_orders() == [(8, 1), (0, 0)]);
        assert!(market.bid_vols() == [5, 0]);
        assert!(market.bid_best_vols() == [5, 0]);
        assert!(market.bid_best_vol_and_orders() == [(5, 1), (0, 0)]);

        assert!(market.order((0, 0)).vol == 8);
        assert!(market.order((0, 1)).vol == 5);
    }

    #[test]
    fn test_serialisation() {
        use rand::{seq::SliceRandom, Rng};
        use rand_xoshiro::rand_core::SeedableRng;
        use rand_xoshiro::Xoroshiro128Plus;

        let mut market: Market<4> = Market::new(0, [1, 1, 1, 1], true);

        let mut rng = Xoroshiro128Plus::seed_from_u64(101);

        for i in (0..500).into_iter() {
            let asset = rng.gen_range(0..4);
            let side = [Side::Bid, Side::Ask].choose(&mut rng).unwrap();
            let price = rng.gen_range(20..40);
            let vol = rng.gen_range(5..20);
            market
                .create_and_place_order(asset, *side, vol, 0, Some(price))
                .unwrap();
            market.set_time(i);
        }

        let market_snapshot = serde_json::to_string(&market).unwrap();
        let loaded_market = serde_json::from_str::<Market<4>>(market_snapshot.as_str()).unwrap();

        assert!(market.get_trade_vols() == loaded_market.get_trade_vols());

        assert!(market.bid_asks() == loaded_market.bid_asks());

        assert!(market.bid_best_vol_and_orders() == loaded_market.bid_best_vol_and_orders());
        assert!(market.bid_vols() == loaded_market.bid_vols());

        assert!(market.ask_best_vol_and_orders() == loaded_market.ask_best_vol_and_orders());
        assert!(market.ask_vols() == loaded_market.ask_vols());
    }
}
