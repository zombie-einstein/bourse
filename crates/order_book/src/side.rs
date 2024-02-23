//! Data structure tracking order on each side of the book
//!
//! Tracks the price-time priority of orders, but also
//! volume at price levels, and total volume on each side
//!
use std::collections::BTreeMap;

use super::types::{Nanos, OrderCount, OrderId, OrderKey, Price, Side, Vol};

/// Common side side functionality
pub trait SideFunctionality {
    /// Initialise a side
    fn new() -> Self;
    /// Insert an order
    fn insert_order(&mut self, key: OrderKey, idx: OrderId, vol: Vol);
    /// Remove an order
    fn remove_order(&mut self, key: OrderKey, vol: Vol);
    /// Remove volume from a price level
    fn remove_vol(&mut self, price: Price, vol: Vol);
    /// Get best price
    fn best_price(&self) -> Price;
    /// Get volume at best price
    fn best_vol(&self) -> Vol;
    /// Get the volume and number at the best_price
    fn best_vol_and_orders(&self) -> (Vol, OrderCount);
    /// Get total volume
    fn vol(&self) -> Vol;
    /// Get the id of the highest priority order
    fn best_order_idx(&self) -> Option<OrderId>;
}

/// Order book side data structure
pub struct OrderBookSide {
    /// Total volume
    vol: Vol,
    /// Volume at price levels
    volumes: BTreeMap<Price, (Vol, OrderCount)>,
    /// Order map and price-time priority queue
    orders: BTreeMap<(Price, Nanos), OrderId>,
}

impl OrderBookSide {
    /// Initialise an empty side
    fn new() -> Self {
        Self {
            vol: 0,
            volumes: BTreeMap::new(),
            orders: BTreeMap::new(),
        }
    }

    /// Insert an order and update volume tracking
    ///
    /// # Arguments
    ///
    /// - `key` - Key of the order
    /// - `idx` - Id of the order
    /// - `vol` - Volume of the order
    ///
    fn insert_order(&mut self, key: OrderKey, idx: OrderId, vol: Vol) {
        self.orders.insert((key.1, key.2), idx);
        match self.volumes.get_mut(&key.1) {
            Some(v) => {
                v.0 += vol;
                v.1 += 1;
            }
            None => {
                self.volumes.insert(key.1, (vol, 1));
            }
        };
        self.vol += vol;
    }

    /// Remove an order and update volume tracking
    ///
    /// # Arguments
    ///
    /// - `key` - Key of the order
    /// - `vol` - Remaining volume of the order to remove
    ///
    fn remove_order(&mut self, key: OrderKey, vol: Vol) {
        self.orders.remove(&(key.1, key.2));
        let vol_at_price = self.volumes.get_mut(&key.1).unwrap();
        vol_at_price.0 -= vol;
        vol_at_price.1 -= 1;
        if vol_at_price.1 == 0 {
            self.volumes.remove(&key.1);
        }
        self.vol -= vol;
    }

    /// Manually remove volume at a price level
    ///
    /// # Arguments
    ///
    /// - `price` - Price level
    /// - `vol` - Volume to remove
    ///
    fn remove_vol(&mut self, price: Price, vol: Vol) {
        self.volumes.get_mut(&price).unwrap().0 -= vol;
        self.vol -= vol;
    }

    /// Get best price of this side
    fn best_price(&self) -> Price {
        match self.orders.first_key_value() {
            Some((k, _)) => k.0,
            None => Price::MAX,
        }
    }

    /// Get the volume at the best price of this side
    fn best_vol_and_orders(&self) -> (Vol, OrderCount) {
        match self.volumes.first_key_value() {
            Some((_, v)) => *v,
            None => (0, 0),
        }
    }

    /// Get the volume at the best price of this side
    fn best_vol(&self) -> Vol {
        match self.volumes.first_key_value() {
            Some((_, v)) => v.0,
            None => 0,
        }
    }

    /// Get the total volume on this side
    fn vol(&self) -> Vol {
        self.vol
    }

    /// Get the id of the highest priority order
    fn best_order_idx(&self) -> Option<OrderId> {
        self.orders.first_key_value().map(|(_, v)| *v)
    }
}

/// Bid-side specific functionality
pub struct BidSide(OrderBookSide);

/// Ask-side specific functionality
pub struct AskSide(OrderBookSide);

impl SideFunctionality for BidSide {
    /// Initialise a new empty bid-side
    fn new() -> Self {
        Self(OrderBookSide::new())
    }

    /// Insert a bid order and update volume tracking
    ///
    /// # Arguments
    ///
    /// - `key` - Key of the order
    /// - `idx` - Id of the order
    /// - `vol` - Volume of the ord
    fn insert_order(&mut self, key: OrderKey, idx: OrderId, vol: Vol) {
        self.0.insert_order(key, idx, vol)
    }

    /// Remove an order and update volume tracking
    ///
    /// # Arguments
    ///
    /// - `key` - Key of the order
    /// - `vol` - Remaining volume of the order to remove
    ///
    fn remove_order(&mut self, key: OrderKey, vol: Vol) {
        self.0.remove_order(key, vol)
    }

    /// Manually remove volume at a price level
    ///
    /// # Arguments
    ///
    /// - `price` - Price level
    /// - `vol` - Volume to remove
    ///
    fn remove_vol(&mut self, price: Price, vol: Vol) {
        self.0.remove_vol(price, vol)
    }

    /// Get best bid price
    fn best_price(&self) -> Price {
        Price::MAX - self.0.best_price()
    }

    /// Get the volume at the best price of this side
    fn best_vol_and_orders(&self) -> (Vol, OrderCount) {
        self.0.best_vol_and_orders()
    }

    /// Get the best bid volume
    fn best_vol(&self) -> Vol {
        self.0.best_vol()
    }

    /// Get the total bid volume
    fn vol(&self) -> Vol {
        self.0.vol()
    }

    /// Get the id of the best bid order
    fn best_order_idx(&self) -> Option<OrderId> {
        self.0.best_order_idx()
    }
}

impl SideFunctionality for AskSide {
    /// Initialise a new empty ask-side
    fn new() -> Self {
        Self(OrderBookSide::new())
    }

    /// Insert a ask order and update volume tracking
    ///
    /// # Arguments
    ///
    /// - `key` - Key of the order
    /// - `idx` - Id of the order
    /// - `vol` - Volume of the ord
    fn insert_order(&mut self, key: OrderKey, idx: OrderId, vol: Vol) {
        self.0.insert_order(key, idx, vol)
    }

    /// Remove an order and update volume tracking
    ///
    /// # Arguments
    ///
    /// - `key` - Key of the order
    /// - `vol` - Remaining volume of the order to remove
    ///
    fn remove_order(&mut self, key: OrderKey, vol: Vol) {
        self.0.remove_order(key, vol)
    }

    /// Manually remove volume at a price level
    ///
    /// # Arguments
    ///
    /// - `price` - Price level
    /// - `vol` - Volume to remove
    ///
    fn remove_vol(&mut self, price: Price, vol: Vol) {
        self.0.remove_vol(price, vol)
    }

    /// Get best ask price
    fn best_price(&self) -> Price {
        self.0.best_price()
    }

    /// Get the volume at the best price of this side
    fn best_vol_and_orders(&self) -> (Vol, OrderCount) {
        self.0.best_vol_and_orders()
    }

    /// Get best ask volume
    fn best_vol(&self) -> Vol {
        self.0.best_vol()
    }

    /// Get total ask volume
    fn vol(&self) -> Vol {
        self.0.vol()
    }

    /// Get the index of the best ask order
    fn best_order_idx(&self) -> Option<OrderId> {
        self.0.best_order_idx()
    }
}

/// Generate a lookup key for a bid-order
///
/// # Arguments
///
/// - `t` - Order arrival time
/// - `price` - Price of the order
///
pub fn get_bid_key(t: Nanos, price: Price) -> OrderKey {
    (Side::Bid, Price::MAX - price, t)
}

/// Generate a lookup key for a ask-order
///
/// # Arguments
///
/// - `t` - Order arrival time
/// - `price` - Price of the order
///
pub fn get_ask_key(t: Nanos, price: Price) -> OrderKey {
    (Side::Ask, price, t)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_ask_init() {
        let side = AskSide::new();

        assert!(side.vol() == 0);
        assert!(side.best_vol() == 0);
        assert!(side.best_price() == Price::MAX);
        assert!(side.best_order_idx().is_none());
    }

    #[test]
    fn test_bid_init() {
        let side = BidSide::new();

        assert!(side.vol() == 0);
        assert!(side.best_vol() == 0);
        assert!(side.best_vol_and_orders() == (0, 0));
        assert!(side.best_price() == 0);
        assert!(side.best_order_idx().is_none());
    }

    #[test]
    fn test_insert_order() {
        let mut side = OrderBookSide::new();

        side.insert_order((Side::Ask, 100, 10), 1, 10);

        assert!(side.vol() == 10);
        assert!(side.best_vol() == 10);
        assert!(side.best_vol_and_orders() == (10, 1));
        assert!(side.best_price() == 100);
        assert!(side.best_order_idx() == Some(1));

        // Insert order at the same level
        side.insert_order((Side::Ask, 100, 11), 2, 11);

        assert!(side.vol() == 21);
        assert!(side.best_vol() == 21);
        assert!(side.best_vol_and_orders() == (21, 2));
        assert!(side.best_price() == 100);
        assert!(side.best_order_idx() == Some(1));

        // Insert higher price
        side.insert_order((Side::Ask, 101, 12), 3, 12);

        assert!(side.vol() == 33);
        assert!(side.best_vol() == 21);
        assert!(side.best_vol_and_orders() == (21, 2));
        assert!(side.best_price() == 100);
        assert!(side.best_order_idx() == Some(1));

        // Insert lower price
        side.insert_order((Side::Ask, 99, 13), 4, 2);

        assert!(side.vol() == 35);
        assert!(side.best_vol() == 2);
        assert!(side.best_vol_and_orders() == (2, 1));
        assert!(side.best_price() == 99);
        assert!(side.best_order_idx() == Some(4));

        let x: u8 = 255 - 10;
        assert!(x == 245);
        let y: u8 = 255 - x;
        assert!(y == 10);
    }

    #[test]
    fn test_best_bid_price() {
        let mut side = BidSide::new();

        side.insert_order(get_bid_key(0, 100), 1, 10);

        assert!(side.best_price() == 100);
    }

    #[test]
    fn test_best_ask_price() {
        let mut side = AskSide::new();

        side.insert_order(get_ask_key(0, 100), 1, 10);

        assert!(side.best_price() == 100);
    }

    #[test]
    fn test_remove_order() {
        let mut side = AskSide::new();

        side.insert_order(get_ask_key(0, 100), 1, 10);
        side.insert_order(get_ask_key(1, 99), 2, 10);

        assert!(side.best_price() == 99);
        assert!(side.vol() == 20);
        assert!(side.best_vol_and_orders() == (10, 1));
        assert!(side.best_order_idx() == Some(2));

        side.remove_order(get_ask_key(1, 99), 10);

        assert!(side.best_price() == 100);
        assert!(side.vol() == 10);
        assert!(side.best_vol_and_orders() == (10, 1));
        assert!(side.best_order_idx() == Some(1));

        side.insert_order(get_ask_key(3, 100), 3, 15);

        assert!(side.best_price() == 100);
        assert!(side.vol() == 25);
        assert!(side.best_vol_and_orders() == (25, 2));
        assert!(side.best_order_idx() == Some(1));

        side.remove_order(get_ask_key(3, 100), 15);

        assert!(side.best_price() == 100);
        assert!(side.vol() == 10);
        assert!(side.best_vol_and_orders() == (10, 1));
        assert!(side.best_order_idx() == Some(1));

        side.remove_order(get_ask_key(0, 100), 10);

        assert!(side.best_price() == Price::MAX);
        assert!(side.vol() == 0);
        assert!(side.best_vol_and_orders() == (0, 0));
        assert!(side.best_order_idx() == None);
    }

    #[test]
    fn test_remove_vol() {
        let mut side = AskSide::new();

        side.insert_order(get_ask_key(0, 100), 1, 10);

        side.remove_vol(100, 5);

        assert!(side.best_vol() == 5);
        assert!(side.best_vol_and_orders() == (5, 1));
        assert!(side.vol() == 5);
    }
}
