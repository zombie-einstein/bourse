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
//! ## Initialisation and Updating an Order Book
//!
//! ```
//! use bourse_book;
//! use bourse_book::{types, OrderBook};
//!
//! let mut book: OrderBook = OrderBook::new(0, 1, true);
//!
//! // Create a new order
//! let order_id = book.create_order(
//!     types::Side::Bid, 50, 101, Some(50)
//! ).unwrap();
//!
//! // Place the order on the market
//! book.place_order(order_id);
//!
//! // Get the current touch prices
//! let (bid, ask) = book.bid_ask();
//!
//! // Set the time of the orderbook
//! book.set_time(1000);
//!
//! // Cancel the order
//! book.cancel_order(order_id);
//! ```
//!
//! [OrderBook] also implements functionality
//! to modify orders and retrieve market data.
//! See [OrderBook] for full details of the API.
//!
//! ## Order & Trade Histories
//!
//! [OrderBook] tracks any orders created and any trades executed
//! over the course of the order books existence. These can
//! be retrieved using:
//!
//! ```
//! # use bourse_book::OrderBook;
//! # use bourse_book::types::{Order, Trade};
//! # let book: OrderBook = OrderBook::new(0, 1, true);
//! // Get references to all the orders created
//! let order_history: Vec<&Order> = book.get_orders();
//! // Get a reference to trade records
//! let trade_history: &Vec<Trade> = book.get_trades();
//! ```
//!
//! ## Persisting State
//!
//! OrderBook implements [serde::Serialize] and
//! [serde::Deserialize] traits to allow the state of
//! the order book to be persisted, this can be done
//! manually, for example:
//!
//! ```
//! # use bourse_book::OrderBook;
//! # let book: OrderBook = OrderBook::new(0, 1, true);
//! let state = serde_json::to_string(&book).unwrap();
//! let book = serde_json::from_str::<OrderBook>(state.as_str()).unwrap();
//! ```
//! or the methods [OrderBook::save_json] and [OrderBook::load_json] can
//! be used to save/load the OrderBook state to/from a JSON file:
//!
//! ```no_run
//! # use bourse_book::OrderBook;
//! # let book: OrderBook = OrderBook::new(0, 1, true);
//! book.save_json("foo.json", true);
//! let loaded_book: OrderBook = OrderBook::load_json("foo.json").unwrap();
//! ```
//!
//! # Notes
//!
//! - Orders are sorted by price-time priority. To
//!   reduce any ambiguity in ordering, the simulated
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

pub use orderbook::{OrderBook, OrderError};
