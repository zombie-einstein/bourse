//! Simulation execution functionality
use super::agents::AgentSet;
use super::env::Env;
use kdam::tqdm;

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
/// use fastrand::Rng;
///
/// // Dummy agent-type
/// struct Agents{}
///
/// impl AgentSet for Agents {
///     fn update(
///         &mut self, env: &mut Env, _rng: &mut Rng
///     ) {}
/// }
///
/// let mut env = bourse_de::Env::new(0, 1_000, true);
/// let mut agents = Agents{};
///
/// // Run for 100 steps from seed 101
/// sim_runner(&mut env, &mut agents, 101, 100)
/// ```
///
/// # Arguments
///
/// - `env` - Simulation environment
/// - `agents` - Agent(s) implementing the [AgentSet] trait
/// - `seed` - Random seed
/// - `n_steps` - Number of simulation steps
///
pub fn sim_runner<A: AgentSet>(env: &mut Env, agents: &mut A, seed: u64, n_steps: u64) {
    let mut rng = fastrand::Rng::with_seed(seed);

    for _ in tqdm!(0..n_steps) {
        agents.update(env, &mut rng);
        env.step(&mut rng);
    }
}
