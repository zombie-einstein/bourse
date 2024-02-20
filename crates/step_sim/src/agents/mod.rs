//! Agent traits for agents to be included in a simulation
//!
//! The simulation agents must implement an `update` function
//! that is called each step of the simulation.
//!
use super::env::Env;
use fastrand::Rng;
mod momentum_agent;
mod noise_agent;

pub use bourse_macros::Agents;
pub use momentum_agent::MomentumAgent;
pub use noise_agent::NoiseAgent;

pub trait Agent {
    fn update(&mut self, env: &mut Env, rng: &mut Rng);
}

/// Functionality required for simulation agents
pub trait AgentSet {
    /// Update function called each simulated step
    ///
    /// This function should update the state of the
    /// agent(s) and submit transactions to the
    /// simulation environment
    ///
    /// The implementing struct is flexible in what
    /// it represent, from a single agent to a group
    /// of multiple agent types
    ///
    /// # Arguments
    ///
    /// - `env` - Simulation environment
    /// - `rng` - Fastrand random generator
    ///
    fn update(&mut self, env: &mut Env, rng: &mut Rng);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bourse_book::types::{Price, Side};

    struct TestAgent {
        side: Side,
        price: Price,
    }

    impl TestAgent {
        pub fn new(side: Side, price: Price) -> Self {
            Self { side, price }
        }
    }

    impl Agent for TestAgent {
        fn update(&mut self, env: &mut Env, _rng: &mut Rng) {
            env.place_order(self.side, 10, 101, Some(self.price));
        }
    }

    #[test]
    fn test_agent_macro() {
        #[derive(Agents)]
        struct TestAgents {
            pub a: TestAgent,
            pub b: TestAgent,
        }

        let mut env = Env::new(0, 1000, true);
        let mut rng = Rng::with_seed(101);

        let mut test_agents = TestAgents {
            a: TestAgent::new(Side::Bid, 20),
            b: TestAgent::new(Side::Ask, 40),
        };

        test_agents.update(&mut env, &mut rng);
        env.step(&mut rng);

        assert!(env.get_orderbook().ask_vol() == 10);
        assert!(env.get_orderbook().bid_vol() == 10);
        assert!(env.get_orderbook().bid_ask() == (20, 40));

        test_agents.update(&mut env, &mut rng);
        env.step(&mut rng);

        assert!(env.get_orderbook().ask_vol() == 20);
        assert!(env.get_orderbook().bid_vol() == 20);
        assert!(env.get_orderbook().bid_ask() == (20, 40));
    }
}
