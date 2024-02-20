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
//! use bourse_de;
//! use bourse_de::types;
//! use fastrand::Rng;
//!
//! let mut env = bourse_de::Env::new(0, 1_000, true);
//! let mut rng = Rng::with_seed(101);
//! let order_id = env.place_order(
//!     types::Side::Ask,
//!     100,
//!     101,
//!     Some(50),
//! );
//! env.step(&mut rng)
//! ```

pub mod agents;
mod env;
mod runner;

pub use bourse_book::types;
pub use env::Env;
pub use runner::sim_runner;
