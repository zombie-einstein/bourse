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
//! See [bourse_book] for details of the limit
//! order-book used in this environment.
//!
//! # Examples
//!
//! ```
//! use bourse_de::types::{Price, Side, Vol};
//! use bourse_de::agents;
//! use bourse_de::agents::Agent;
//! use bourse_de::{sim_runner, Env};
//! use rand::{RngCore, Rng};
//!
//! // Define a set of agents using built
//! // in definitions
//! #[derive(agents::Agents)]
//! struct SimAgents {
//!     pub a: agents::MomentumAgent,
//!     pub b: agents::NoiseAgent,
//! }
//!
//! // Initialise agent parameters
//! let m_params = agents::MomentumParams {
//!     tick_size: 2,
//!     p_cancel: 0.1,
//!     trade_vol: 100,
//!     decay: 1.0,
//!     demand: 5.0,
//!     scale: 0.5,
//!     order_ratio: 1.0,
//!     price_dist_mu: 0.0,
//!     price_dist_sigma: 10.0,
//! };
//!
//! let n_params = agents::NoiseAgentParams{
//!     tick_size: 2,
//!     p_limit: 0.2,
//!     p_market: 0.2,
//!     p_cancel: 0.1,
//!     trade_vol: 100,
//!     price_dist_mu: 0.0,
//!     price_dist_sigma: 1.0,
//! };
//!
//! let mut agents = SimAgents {
//!     a: agents::MomentumAgent::new(0, 10, m_params),
//!     b: agents::NoiseAgent::new(10, 20, n_params),
//! };
//!
//! // Initialise the environment and agents
//! let mut env = Env::new(0, 1, 1_000_000, true);
//!
//! // Run the simulation
//! sim_runner(&mut env, &mut agents, 101, 50, true);
//!
//! // Get history of level 2 data over the course of the simulation
//! let data = env.level_2_data();
//! ```
//!
//! # Implementing Your Own Agents
//!
//! For use in [sim_runner] simulation agents should implement the [agents::AgentSet]
//! trait. For a set of homogeneous agents (i.e. all the agents are the
//! same type) this can be implemented directly.
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
//! sim_runner(&mut env, &mut agents, 101, 50, true);
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
