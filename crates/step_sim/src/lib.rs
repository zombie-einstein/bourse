//! Discrete event market simulation library
//!
//! Implements a discrete event simulation
//! environment ([Env]) and utilities for writing
//! discrete event market simulations.
//!
//! # Model
//!
//! In the discrete event model the state of
//! the market is updated in fixed size
//! time-steps. Inside each step agents can
//! submit updates/instructions to the
//! the simulated market, which are then processed
//! at the end of the step.
//!
//! Each simulation update performs the following
//! steps:
//!
//! - Iterate over all the agents, updating their
//!   state, and market instructions submitted to
//!   a queue.
//! - The instructions are randomly shuffled, and
//!   then processed in sequence, updating the state
//!   of the market.
//! - Record the state of the market.
//!
//! Hence agents can only observe the state of the
//! market at the end of the previous step when
//! updating, and have no guarantee of the ordering
//! of transactions.
//!
//! # Examples
//!
//! ```
//! use bourse_de::types::{Price, Side, Vol};
//! use bourse_de::agents::AgentSet;
//! use bourse_de::{sim_runner, Env};
//! use rand::{RngCore, Rng};
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
//!     fn update<R: RngCore>(
//!         &mut self, env: &mut Env, rng: &mut R
//!     ) {
//!         let bid = env.level_2_data().bid_price;
//!         let ask = env.level_2_data().ask_price;
//!         let mid = (ask - bid) / 2;
//!         let mid_price = bid + mid;
//!         for _ in (0..self.n_agents) {
//!             let side = rng.gen_bool(0.5);
//!             match side {
//!                 true => {
//!                     let p = mid_price - self.offset;
//!                     env.place_order(Side::Ask, self.vol, 101, Some(p));
//!                 }
//!                 false => {
//!                     let p = mid_price + self.offset;
//!                     env.place_order(Side::Bid, self.vol, 101, Some(p));
//!                 }
//!             }
//!         }
//!     }
//! }
//!
//! // Initialise the environment and agents
//! let mut env = Env::new(0, 1, 1_000_000, true);
//! let mut agents = Agents{offset: 6, vol: 50, n_agents: 10};
//!
//! // Run the simulation
//! sim_runner(&mut env, &mut agents, 101, 50);
//!
//! // Get history of prices over the course of the simulation
//! let price_data = env.get_prices();
//! ```
//!
//! # Implementing Your Own Agents
//!
//! For use in [sim_runner] simulation agents should implement the [agents::AgentSet]
//! trait. For a set of homogeneous agents (i.e. all the agents are the
//! same type) this can be implemented directly as in the above example.
//!
//! For a mixture of agent types, the [agents::Agents] macro can be used
//! to automatically implement [agents::AgentSet] for a struct of agents
//! all implementing [agents::Agent]. For examples
//!
//! ```
//! use bourse_de::{Env, sim_runner};
//! use bourse_de::agents::{Agent, AgentSet, Agents};
//! use rand::RngCore;
//!
//! struct AgentTypeA{}
//!
//! impl Agent for AgentTypeA{
//!     fn update<R: RngCore>(
//!         &mut self, env: &mut bourse_de::Env, rng: &mut R
//!     ) {}
//! }
//!
//! struct AgentTypeB{}
//!
//! impl Agent for AgentTypeB{
//!     fn update<R: RngCore>(
//!         &mut self, env: &mut bourse_de::Env, rng: &mut R
//!     ) {}
//! }
//!
//! #[derive(Agents)]
//! struct SimAgents {
//!     pub a: AgentTypeA,
//!     pub b: AgentTypeB,
//! }
//!
//! let mut env = bourse_de::Env::new(0, 1, 1_000_000, true);
//! let mut agents = SimAgents{a: AgentTypeA{}, b: AgentTypeB{}};
//!
//! sim_runner(&mut env, &mut agents, 101, 50);
//! ```
//!
//! # Randomness
//!
//! To ensure simulations are deterministic (given a random seed)
//! random generators (that implement the [rand::RngCore] trait) are
//! passed to agents during the simulation. The [rand_distr] crate
//! can be used to sample from common distributions.
//!

pub mod agents;
mod env;
mod runner;

pub use bourse_book::{types, OrderError};
pub use env::Env;
pub use runner::sim_runner;
