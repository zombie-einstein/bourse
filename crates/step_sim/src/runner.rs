//! Simulation execution functionality
use super::agents::{AgentSet, MarketAgentSet};
use super::env::Env;
use super::market_env::MarketEnv;
use kdam::tqdm;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoroshiro128StarStar;

/// Run a simulation for a fixed number of steps
///
/// Each step updates the state of the agents (who
/// in turn can submit instructions to the environment
/// and then update the environment state)
///
/// # Examples
///
/// ```
/// use bourse_de::{Env, sim_runner};
/// use bourse_de::agents::AgentSet;
/// use rand::RngCore;
///
/// // Dummy agent-type
/// struct Agents{}
///
/// impl AgentSet for Agents {
///     fn update<R: RngCore>(
///         &mut self, env: &mut Env, _rng: &mut R
///     ) {}
/// }
///
/// let mut env = bourse_de::Env::new(0, 1, 1_000, true);
/// let mut agents = Agents{};
///
/// // Run for 100 steps from seed 101
/// sim_runner(&mut env, &mut agents, 101, 100, true)
/// ```
///
/// # Arguments
///
/// - `env` - Simulation environment
/// - `agents` - Agent(s) implementing the [AgentSet] trait
/// - `seed` - Random seed
/// - `n_steps` - Number of simulation steps
/// - `show_progress` - Show progress bar
///
pub fn sim_runner<A: AgentSet>(
    env: &mut Env,
    agents: &mut A,
    seed: u64,
    n_steps: u64,
    show_progress: bool,
) {
    let mut rng = Xoroshiro128StarStar::seed_from_u64(seed);

    match show_progress {
        true => {
            for _ in tqdm!(0..n_steps) {
                agents.update(env, &mut rng);
                env.step(&mut rng);
            }
        }
        false => {
            for _ in 0..n_steps {
                agents.update(env, &mut rng);
                env.step(&mut rng);
            }
        }
    }
}

/// Run a simulation for a fixed number of steps
///
/// Each step updates the state of the agents (who
/// in turn can submit instructions to the environment
/// and then update the environment state)
///
/// # Examples
///
/// ```
/// use bourse_de::{MarketEnv, market_sim_runner};
/// use bourse_de::agents::MarketAgentSet;
/// use rand::RngCore;
///
/// // Dummy agent-type
/// struct Agents{}
///
/// impl MarketAgentSet for Agents {
///     fn update<R: RngCore, const M: usize, const N: usize>(
///         &mut self, _env: &mut MarketEnv<M, N>, _rng: &mut R
///     ) {}
/// }
///
/// let mut env = bourse_de::MarketEnv::<2>::new(0, [1, 1], 1_000, true);
/// let mut agents = Agents{};
///
/// // Run for 100 steps from seed 101
/// market_sim_runner(&mut env, &mut agents, 101, 100, true)
/// ```
///
/// # Arguments
///
/// - `env` - Simulation environment
/// - `agents` - Agent(s) implementing the [MarketAgentSet] trait
/// - `seed` - Random seed
/// - `n_steps` - Number of simulation steps
/// - `show_progress` - Show progress bar
///
pub fn market_sim_runner<A: MarketAgentSet, const M: usize, const N: usize>(
    env: &mut MarketEnv<M, N>,
    agents: &mut A,
    seed: u64,
    n_steps: u64,
    show_progress: bool,
) {
    let mut rng = Xoroshiro128StarStar::seed_from_u64(seed);

    match show_progress {
        true => {
            for _ in tqdm!(0..n_steps) {
                agents.update(env, &mut rng);
                env.step(&mut rng);
            }
        }
        false => {
            for _ in 0..n_steps {
                agents.update(env, &mut rng);
                env.step(&mut rng);
            }
        }
    }
}
