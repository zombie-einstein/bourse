//! Simulated order book
//!
//! Simulated order book designed for
//! market simulations. Acts as
//! matching engine, but also data
//! structure tracking simulated orders
//! and historical data.
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

mod orderbook;
mod side;
pub mod types;

pub use orderbook::OrderBook;
