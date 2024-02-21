//! Discrete event market simulation library
//!
//! Implements a discrete event simulation
//! environment and utilities for writing
//! market simulations.
//!
//! In the discrete event model the state of
//! the market is updated in fixed size
//! time-steps. Inside each step agents
//! submit instructions to the market, which
//! are shuffled then applied at the end of
//! the step.
//!
//! # Examples
//!
//! ```
//! use bourse_book::types::{Price, Side, Vol};
//! use bourse_de::agents::AgentSet;
//! use bourse_de::{sim_runner, Env};
//! use fastrand::Rng;
//!
//! struct Agents {
//!     pub offset: Price,
//!     pub vol: Vol,
//!     pub n_agents: usize,
//! }
//!
//! impl AgentSet for Agents {
//!     // Agents place an order on a random side
//!     // a fixed distance above/below the mid
//!     fn update(
//!         &mut self, env: &mut Env, rng: &mut Rng
//!     ) {
//!         let (bid, ask) = env.get_orderbook().bid_ask();
//!         let mid = (ask - bid) / 2;
//!         let mid_price = bid + mid;
//!         println!("{}", mid_price);
//!         for _ in (0..self.n_agents) {
//!             let side = rng.choice([Side::Bid, Side::Ask]).unwrap();
//!             match side {
//!                 Side::Ask => {
//!                     let p = mid_price - self.offset;
//!                     println!("==> {}", p);
//!                     env.place_order(side, self.vol, 101, Some(p));
//!                 }
//!                 Side::Bid => {
//!                     let p = mid_price + self.offset;
//!                     println!("==< {}", p);
//!                     env.place_order(side, self.vol, 101, Some(p));
//!                 }
//!             }
//!         }
//!     }
//! }
//!
//! let mut env = Env::new(0, 1_000_000, true);
//! let mut agents = Agents{offset: 5, vol: 50, n_agents: 10};
//!
//! sim_runner(&mut env, &mut agents, 101, 50);
//! ```

pub mod agents;
mod env;
mod runner;

pub use bourse_book::types;
pub use env::Env;
pub use runner::sim_runner;
