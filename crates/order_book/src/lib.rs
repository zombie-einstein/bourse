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
//!
//! // Create a new order
//! let order_id = book.create_order(
//!     types::Side::Bid, 50, 101, Some(50)
//! );
//!
//! // Place the order on the market
//! book.place_order(order_id);
//!
//! // Get the current touch prices
//! let (bid, ask) = book.bid_ask();
//!
//! // Cancel the order
//! book.cancel_order(order_id);
//! ```
//! # Notes
//!
//! - Orders are sorted by price-time priority. To
//!   reduce any ambiguity in ordering the simulated
//!   time of the market should be updated in
//!   between placing orders on the market.
//! - For accuracy prices are stored as unsigned
//!   integers (as opposed to a float type), hence
//!   prices from data should be scaled based on
//!   market tick-size
//! - Simulated orders are intended to be owned by
//!   the order book, from which agents/users can
//!   retrieve order data. Creating an order with
//!   [OrderBook::create_order] initialises a new
//!   order entry, but does not immediately place
//!   the order on the market.
//!
mod orderbook;
mod side;
pub mod types;

pub use orderbook::OrderBook;
