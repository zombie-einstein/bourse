//! Market data recording
use crate::types::{Level2Data, OrderCount, Price, Vol};
use std::array;

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

impl<const N: usize> Default for Level2DataRecords<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Level2DataRecords<N> {
    /// Initialise an empty set of records
    pub fn new() -> Self {
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
    pub fn append_record(&mut self, record: &Level2Data<N>) {
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
