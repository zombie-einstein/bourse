//! Simulation agent traits and common implementations
//!
//! The simulation agents must implement an `update` function
//! that is called each step of the simulation.
//!
use crate::{Env, MarketEnv};

use rand::RngCore;
pub mod common;
mod momentum_agent;
mod noise_agent;
mod random_agent;

pub use bourse_macros::{AgentSet, MarketAgentSet};
pub use momentum_agent::{MomentumAgent, MomentumMarketAgent, MomentumParams};
pub use noise_agent::{NoiseAgent, NoiseAgentParams, NoiseMarketAgent};
pub use random_agent::{RandomAgents, RandomMarketAgents};

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
/// use bourse_de::agents::{Agent, AgentSet};
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
/// #[derive(AgentSet)]
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
/// use bourse_de::agents::{Agent, AgentSet};
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
/// #[derive(AgentSet)]
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
/// # use bourse_de::agents::{Agent, AgentSet};
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

/// Homogeneous agent set functionality
///
/// A set of agents that implement this trait
/// can then be included in a struct using the
/// [MarketAgentSet] macro to combine multiple agent
/// types.
///
/// # Examples
///
/// ```
/// use bourse_de::MarketEnv;
/// use bourse_de::agents::{MarketAgent, MarketAgentSet};
/// use rand::RngCore;
///
/// struct AgentType{}
///
/// impl MarketAgent for AgentType {
///     fn update<R: RngCore, const M: usize, const N: usize>(
///         &mut self, env: &mut MarketEnv<M, N>, _rng: &mut R
///     ) {}
/// }
///
/// #[derive(MarketAgentSet)]
/// struct MixedAgents {
///     a: AgentType, b: AgentType
/// }
/// ```
pub trait MarketAgent {
    /// Update the state of the agent(s)
    ///
    /// # Argument
    ///
    /// - `env` - Reference to a [MarketEnv] simulation environment
    /// - `rng` - Random generator
    ///
    fn update<R: RngCore, const M: usize, const N: usize>(
        &mut self,
        env: &mut MarketEnv<M, N>,
        rng: &mut R,
    );
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
/// automatically implemented with the [MarketAgentSet] macro
/// as long as the agent types implement the [MarketAgent]
/// trait.
///
/// # Examples
///
/// ```
/// use bourse_de::MarketEnv;
/// use bourse_de::agents::{MarketAgent, MarketAgentSet};
/// use rand::RngCore;
///
/// struct AgentType{}
///
/// impl MarketAgent for AgentType {
///     fn update<R: RngCore, const M: usize, const N: usize>(
///         &mut self, env: &mut MarketEnv<M, N>, _rng: &mut R
///     ) {}
/// }
///
/// #[derive(MarketAgentSet)]
/// struct MixedAgents {
///     a: AgentType,
///     b: AgentType
/// }
/// ```
///
/// this is equivalent to
///
/// ```
/// # use bourse_de::MarketEnv;
/// # use bourse_de::agents::{MarketAgent, MarketAgentSet};
/// # use rand::RngCore;
/// # struct AgentType{}
/// # impl MarketAgent for AgentType {
/// #    fn update<R: RngCore, const M: usize, const N: usize>(
/// #        &mut self, env: &mut MarketEnv<M, N>, _rng: &mut R
/// #     ) {}
/// # }
/// struct MixedAgents {
///     a: AgentType,
///     b: AgentType
/// }
///
/// impl MarketAgentSet for MixedAgents {
///     fn update<R: RngCore, const M: usize, const N: usize>(
///         &mut self, env: &mut MarketEnv<M, N>, rng: &mut R
///     ){
///         self.a.update(env, rng);
///         self.b.update(env, rng);
///     }
/// }
/// ```
pub trait MarketAgentSet {
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
    fn update<R: RngCore, const M: usize, const N: usize>(
        &mut self,
        env: &mut MarketEnv<M, N>,
        rng: &mut R,
    );
}
