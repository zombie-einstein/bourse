//! Simulation agent traits and common implementations
//!
//! The simulation agents must implement an `update` function
//! that is called each step of the simulation.
//!
use super::env::Env;
use rand::RngCore;
pub mod common;
mod momentum_agent;
mod noise_agent;
mod random_agent;

pub use bourse_macros::Agents;
pub use momentum_agent::{MomentumAgent, MomentumParams};
pub use noise_agent::{NoiseAgent, NoiseAgentParams};
pub use random_agent::RandomAgents;

/// Homogeneous agent set functionality
///
/// A set of agents that implement this trait
/// can then be included in a struct using the
/// [Agents] macro to combine multiple agent
/// types.
///
/// # Examples
///
/// ```
/// use bourse_de::Env;
/// use bourse_de::agents::{Agent, Agents, AgentSet};
/// use rand::RngCore;
///
/// struct AgentType{}
///
/// impl Agent for AgentType {
///     fn update<R: RngCore>(
///         &mut self, env: &mut Env, _rng: &mut R
///     ) {}
/// }
///
/// #[derive(Agents)]
/// struct MixedAgents {
///     a: AgentType, b: AgentType
/// }
/// ```
pub trait Agent {
    /// Update the state of the agent(s)
    ///
    /// # Argument
    ///
    /// - `env` - Reference to a [Env] simulation environment
    /// - `rng` - Random generator
    ///
    fn update<R: RngCore>(&mut self, env: &mut Env, rng: &mut R);
}

/// Functionality required for simulation agents
///
/// Simulation agents provided as an argument to
/// [crate::sim_runner] must implement this trait,
/// but the details of the implementation are
/// left to the user.
///
/// It's a common case that we want update to update
/// a heterogeneous set of agents which can be
/// automatically implemented with the [Agents] macro
/// as long as the agent types implement the [Agent]
/// trait.
///
/// # Examples
///
/// ```
/// use bourse_de::Env;
/// use bourse_de::agents::{Agent, Agents, AgentSet};
/// use rand::RngCore;
///
/// struct AgentType{}
///
/// impl Agent for AgentType {
///     fn update<R: RngCore>(
///         &mut self, env: &mut Env, _rng: &mut R
///     ) {}
/// }
///
/// #[derive(Agents)]
/// struct MixedAgents {
///     a: AgentType,
///     b: AgentType
/// }
/// ```
///
/// this is equivalent to
///
/// ```
/// # use bourse_de::Env;
/// # use bourse_de::agents::{Agent, Agents, AgentSet};
/// # use rand::RngCore;
/// # struct AgentType{}
/// # impl Agent for AgentType {
/// #    fn update<R: RngCore>(
/// #        &mut self, env: &mut Env, _rng: &mut R
/// #     ) {}
/// # }
/// struct MixedAgents {
///     a: AgentType,
///     b: AgentType
/// }
///
/// impl AgentSet for MixedAgents {
///     fn update<R: RngCore>(&mut self, env: &mut Env, rng: &mut R){
///         self.a.update(env, rng);
///         self.b.update(env, rng);
///     }
/// }
/// ```
pub trait AgentSet {
    /// Update function called each simulated step
    ///
    /// This function should update the state of the
    /// agent(s) and submit transactions to the
    /// simulation environment
    ///
    /// The implementing struct is flexible in what
    /// it represent, from a single agent to a group
    /// of multiple agent types.
    ///
    /// # Arguments
    ///
    /// - `env` - Simulation environment
    /// - `rng` - Random generator
    ///
    fn update<R: RngCore>(&mut self, env: &mut Env, rng: &mut R);
}
