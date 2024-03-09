use super::Agent;
use crate::types::{OrderId, Price, Side, Status, TraderId, Vol};
use crate::Env;
use rand::seq::SliceRandom;
use rand::Rng;
use rand::RngCore;

/// Agents that place orders with uniformly sampled parameters
///
/// A set of agents that place orders on a random side with
/// random volume and price within a given range. Sides,
/// volumes and prices are uniformly sampled.
///
/// Each step each agent:
///
/// - Are randomly activated with a given activity rate
/// - If active and they have an active order on the
///   market they try to cancel that order
/// - If they don't have an active order then they
///   place a new order on a random side, with a
///   random volume and price
///
/// <div id="doc-warning-1" class="warning">
/// The behaviour of this agent is not based on anything
/// 'realistic' and should probably only be used for
/// testing and benchmarking.
/// </div>
///
/// # Examples
///
/// ```
/// use bourse_de::agents::{Agent, RandomAgents, Agents};
/// use bourse_de::{sim_runner, Env};
///
/// #[derive(Agents)]
/// struct SimAgents {
///     pub a: RandomAgents,
/// }
///
/// let mut env = Env::new(0, 1, 1_000_000, true);
///
/// let mut agents = SimAgents {
///     a: RandomAgents::new(10, (40, 60), (10, 20), 2, 0.8),
/// };
///
/// sim_runner(&mut env, &mut agents, 101, 10, false);
/// ```
pub struct RandomAgents {
    orders: Vec<Option<OrderId>>,
    tick_range: (Price, Price),
    vol_range: (Vol, Vol),
    tick_size: Price,
    activity_rate: f32,
}

impl RandomAgents {
    /// Initialise a set of random agents
    ///
    /// # Arguments
    ///
    /// - `n_agents` - Number of agents in the set
    /// - `tick_range` - Range of ticks to place orders over
    /// - `vol_range` - Order volume range to sample from
    /// - `tick_size` - Market tick size
    /// - `activity_rate` - Agent activity rate
    ///
    pub fn new(
        n_agents: usize,
        tick_range: (Price, Price),
        vol_range: (Vol, Vol),
        tick_size: Price,
        activity_rate: f32,
    ) -> Self {
        Self {
            orders: vec![None; n_agents],
            tick_range,
            vol_range,
            tick_size,
            activity_rate,
        }
    }
}

impl Agent for RandomAgents {
    fn update<R: RngCore>(&mut self, env: &mut Env, rng: &mut R) {
        let new_orders: Vec<Option<OrderId>> = self
            .orders
            .iter_mut()
            .enumerate()
            .map(|(n, i)| {
                let p = rng.gen::<f32>();

                match p < self.activity_rate {
                    true => {
                        if (i.is_some()) && (env.order_status(i.unwrap()) == Status::Active) {
                            env.cancel_order(i.unwrap());
                            None
                        } else {
                            let side = [Side::Ask, Side::Bid].choose(rng).unwrap();
                            let tick = rng.gen_range(self.tick_range.0..self.tick_range.1);
                            let vol = rng.gen_range(self.vol_range.0..self.vol_range.1);
                            Some(
                                env.place_order(
                                    *side,
                                    vol,
                                    TraderId::try_from(n).unwrap(),
                                    Some(tick * self.tick_size),
                                )
                                .unwrap(),
                            )
                        }
                    }
                    false => *i,
                }
            })
            .collect();

        self.orders = new_orders;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bourse_book::types::Event;
    use rand::SeedableRng;
    use rand_xoshiro::Xoroshiro128StarStar;

    #[test]
    fn test_activity_rate() {
        let mut env = Env::new(0, 1, 1000, true);
        let mut rng = Xoroshiro128StarStar::seed_from_u64(101);

        let mut agents = RandomAgents::new(2, (10, 20), (20, 30), 1, 0.0);

        agents.update(&mut env, &mut rng);
        assert!(env.get_transactions().len() == 0);

        agents.activity_rate = 1.0;
        agents.update(&mut env, &mut rng);
        assert!(env.get_transactions().len() == 2);
    }

    #[test]
    fn test_order_place_then_cancel() {
        let mut env = Env::new(0, 1, 1000, true);
        let mut rng = Xoroshiro128StarStar::seed_from_u64(101);

        let mut agents = RandomAgents::new(1, (10, 20), (20, 30), 1, 1.0);

        agents.update(&mut env, &mut rng);
        assert!(env.get_transactions().len() == 1);
        assert!(matches!(env.get_transactions()[0], Event::New { .. }));
        assert!(agents.orders == vec![Some(0)]);

        env.step(&mut rng);

        agents.update(&mut env, &mut rng);
        assert!(env.get_transactions().len() == 1);
        assert!(matches!(
            env.get_transactions()[0],
            Event::Cancellation { .. }
        ));

        env.step(&mut rng);

        agents.update(&mut env, &mut rng);
        assert!(env.get_transactions().len() == 1);
        assert!(matches!(env.get_transactions()[0], Event::New { .. }));
        assert!(agents.orders == vec![Some(1)]);
    }
}
