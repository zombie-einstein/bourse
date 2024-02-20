/// Simulation execution functionality
use super::agents::AgentSet;
use super::env::Env;
use kdam::tqdm;

/// Run a simulation for a fixed number of steps
///
/// Each step updates the state of the agents (who
/// in turn can submit instructions to the environment
/// and then update the environment state)
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
