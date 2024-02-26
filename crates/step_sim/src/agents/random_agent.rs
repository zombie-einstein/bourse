use super::Agent;
use crate::types::{OrderId, Price, Side, Status, TraderId, Vol};
use crate::Env;

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
/// use bourse_de::agents::{Agent, AgentSet, RandomAgents};
/// use bourse_de::{sim_runner, Env};
/// use bourse_macros::Agents;
/// use fastrand::Rng;
///
/// #[derive(Agents)]
/// struct SimAgents {
///     pub a: RandomAgents,
/// }
///
/// let mut env = Env::new(0, 1_000_000, true);
///
/// let mut agents = SimAgents {
///     a: RandomAgents::new(10, (40, 60), (10, 20), 0.8),
/// };
///
/// sim_runner(&mut env, &mut agents, 101, 10);
/// ```
pub struct RandomAgents {
    orders: Vec<Option<OrderId>>,
    price_range: (Price, Price),
    vol_range: (Vol, Vol),
    activity_rate: f32,
}

impl RandomAgents {
    pub fn new(
        n_agents: usize,
        price_range: (Price, Price),
        vol_range: (Vol, Vol),
        activity_rate: f32,
    ) -> Self {
        Self {
            orders: vec![None; n_agents],
            price_range,
            vol_range,
            activity_rate,
        }
    }

    pub fn sample_order(&mut self, rng: &mut fastrand::Rng) -> (Side, Price, Vol) {
        let side = rng.choice([Side::Ask, Side::Bid]).unwrap();
        let price = rng.u32(self.price_range.0..self.price_range.1);
        let vol = rng.u32(self.vol_range.0..self.vol_range.1);
        (side, price, vol)
    }
}

impl Agent for RandomAgents {
    fn update(&mut self, env: &mut Env, rng: &mut fastrand::Rng) {
        let new_orders: Vec<Option<OrderId>> = self
            .orders
            .iter_mut()
            .enumerate()
            .map(|(n, i)| {
                let p = rng.f32();

                match p < self.activity_rate {
                    true => {
                        if (i.is_some()) && (env.order_status(i.unwrap()) == Status::Active) {
                            env.cancel_order(i.unwrap());
                            None
                        } else {
                            let side = rng.choice([Side::Ask, Side::Bid]).unwrap();
                            let price = rng.u32(self.price_range.0..self.price_range.1);
                            let vol = rng.u32(self.vol_range.0..self.vol_range.1);
                            Some(env.place_order(
                                side,
                                vol,
                                TraderId::try_from(n).unwrap(),
                                Some(price),
                            ))
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
    use fastrand::Rng;

    #[test]
    fn test_activity_rate() {
        let mut env = Env::new(0, 1000, true);
        let mut rng = Rng::with_seed(101);

        let mut agents = RandomAgents::new(2, (10, 20), (20, 30), 0.0);

        agents.update(&mut env, &mut rng);
        assert!(env.get_transactions().len() == 0);

        agents.activity_rate = 1.0;
        agents.update(&mut env, &mut rng);
        assert!(env.get_transactions().len() == 2);
    }

    #[test]
    fn test_order_place_then_cancel() {
        let mut env = Env::new(0, 1000, true);
        let mut rng = Rng::with_seed(101);

        let mut agents = RandomAgents::new(1, (10, 20), (20, 30), 1.0);

        agents.update(&mut env, &mut rng);
        assert!(env.get_transactions().len() == 1);
        matches!(env.get_transactions()[0], Event::New { .. });
        assert!(agents.orders == vec![Some(0)]);

        env.step(&mut rng);

        agents.update(&mut env, &mut rng);
        assert!(env.get_transactions().len() == 1);
        matches!(env.get_transactions()[0], Event::Cancellation { .. });

        env.step(&mut rng);

        agents.update(&mut env, &mut rng);
        assert!(env.get_transactions().len() == 1);
        matches!(env.get_transactions()[0], Event::New { .. });
        assert!(agents.orders == vec![Some(1)]);
    }
}
