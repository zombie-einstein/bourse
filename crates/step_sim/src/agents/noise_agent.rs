//! Agent that randomly places and cancels limit and market orders
use super::common;
use super::Agent;
use crate::types::{OrderId, Price, Side, TraderId, Vol};
use crate::Env;
use rand::Rng;
use rand::RngCore;
use rand_distr::LogNormal;

/// Noise agent parameters
pub struct NoiseAgentParams {
    /// Tick-size of the market
    pub tick_size: Price,
    /// Probability of placing a limit order
    pub p_limit: f32,
    /// Probability of placing a market order
    pub p_market: f32,
    /// Probability of cancelling a live order
    pub p_cancel: f32,
    /// Size of trades that are placed
    pub trade_vol: Vol,
    /// Log-normal price distribution mean
    pub price_dist_mu: f64,
    /// Log-normal price distribution width
    pub price_dist_sigma: f64,
}

/// Agent(s) that randomly place and cancel limit and market orders
///
/// Represents a group of agents that randomly place and cancel
/// orders at each step of the simulation. Each step:
///
/// - Any currently live orders are randomly selected for cancellation
/// - Each agent randomly chooses to place a limit order, if so they
///   place an order on a random side with a price above/below the
///   mid-price by a distance sampled from a log-normal distribution
/// - Each agent randomly chooses to place a market order, if so they
///   place an order on a random side
///
/// In both cases orders are placed at a fixed size.
///
/// # Examples
///
/// ```
/// use bourse_de::agents::{Agent, AgentSet, NoiseAgent, NoiseAgentParams};
/// use bourse_de::{sim_runner, Env};
/// use bourse_macros::Agents;
///
/// #[derive(Agents)]
/// struct SimAgents {
///     pub a: NoiseAgent,
/// }
///
/// let mut env = Env::new(0, 1, 1_000_000, true);
///
/// let params = NoiseAgentParams{
///     tick_size: 2,
///     p_limit: 0.2,
///     p_market: 0.2,
///     p_cancel: 0.1,
///     trade_vol: 100,
///     price_dist_mu: 0.0,
///     price_dist_sigma: 1.0,
/// };
/// let mut agents = SimAgents {
///     a: NoiseAgent::new(0, 5, params),
/// };
///
/// sim_runner(&mut env, &mut agents, 101, 10, false);
/// ```
///
/// # References
///
/// 1. <https://arxiv.org/abs/2208.13654>
///
pub struct NoiseAgent {
    tick_size: f64,
    price_dist: LogNormal<f64>,
    orders: Vec<OrderId>,
    trader_ids: Vec<TraderId>,
    params: NoiseAgentParams,
}

impl NoiseAgent {
    /// Initialise a group of noise-agents
    ///
    /// # Parameters
    ///
    /// - `agent_id_start` - Starting id for
    ///   agents in this set
    /// - `n_agents` - Number of agents
    /// - `p_limit` - Probability each agent places
    ///   a new limit order each step
    /// - `p_market` - Probability each agent places
    ///   a new market order each step
    /// - `p_cancel` - Probability of cancelling a
    ///   live order
    /// - `trade_vol` - Size of orders placed by
    ///   the agents
    /// - `tick_size` - Integer tick-size of the
    ///   market
    /// - `price_dist_mu` - Mean parameter of the
    ///   log-normal distribution that limit-order
    ///   prices are sampled from
    /// - `price_dist_sigma` - Width parameter of the
    ///   log-normal distribution that limit-order
    ///   prices are sampled from
    ///
    pub fn new(agent_id_start: TraderId, n_agents: u16, params: NoiseAgentParams) -> Self {
        let trader_ids = (agent_id_start..agent_id_start + TraderId::from(n_agents)).collect();

        Self {
            tick_size: params.tick_size.into(),

            price_dist: LogNormal::<f64>::new(params.price_dist_mu, params.price_dist_sigma)
                .unwrap(),
            orders: Vec::new(),
            trader_ids,
            params,
        }
    }
}

impl Agent for NoiseAgent {
    fn update<R: RngCore>(&mut self, env: &mut Env, rng: &mut R) {
        let mut live_orders =
            common::cancel_live_orders(env, rng, &self.orders, self.params.p_cancel);

        let mid_price = env.get_orderbook().mid_price();

        for trader_id in self.trader_ids.iter() {
            if rng.gen::<f32>() < self.params.p_limit {
                let side = rng.gen_bool(0.5);

                let order_id = match side {
                    true => common::place_buy_limit_order(
                        env,
                        rng,
                        self.price_dist,
                        mid_price,
                        self.tick_size,
                        self.params.trade_vol,
                        *trader_id,
                    )
                    .unwrap(),
                    false => common::place_sell_limit_order(
                        env,
                        rng,
                        self.price_dist,
                        mid_price,
                        self.tick_size,
                        self.params.trade_vol,
                        *trader_id,
                    )
                    .unwrap(),
                };
                live_orders.push(order_id);
            }

            if rng.gen::<f32>() < self.params.p_market {
                let side = rng.gen_bool(0.5);
                match side {
                    true => env
                        .place_order(Side::Bid, self.params.trade_vol, *trader_id, None)
                        .unwrap(),
                    false => env
                        .place_order(Side::Ask, self.params.trade_vol, *trader_id, None)
                        .unwrap(),
                };
            }
        }

        self.orders = live_orders;
    }
}

#[cfg(test)]
mod tests {
    use crate::types::Status;
    use bourse_book::types::Event;
    use rand::SeedableRng;
    use rand_xoshiro::Xoroshiro128StarStar;

    use super::*;

    #[test]
    fn test_init() {
        let params = NoiseAgentParams {
            tick_size: 2,
            p_limit: 0.5,
            p_market: 0.5,
            p_cancel: 0.1,
            trade_vol: 100,
            price_dist_mu: 0.0,
            price_dist_sigma: 1.0,
        };
        let agents = NoiseAgent::new(10, 4, params);

        assert!(agents.trader_ids == vec![10, 11, 12, 13])
    }

    #[test]
    fn test_place_and_cancel_limit_orders() {
        let mut env = Env::new(0, 1, 1_000_000, true);
        let mut rng = Xoroshiro128StarStar::seed_from_u64(101);

        let params = NoiseAgentParams {
            tick_size: 2,
            p_limit: 1.0,
            p_market: 0.0,
            p_cancel: 1.0,
            trade_vol: 100,
            price_dist_mu: 0.0,
            price_dist_sigma: 10.0,
        };
        let mut agents = NoiseAgent::new(10, 10, params);

        agents.update(&mut env, &mut rng);

        assert!(agents.orders.len() == 10);
        assert!(env.get_transactions().len() == 10);

        let mid_price = env.get_orderbook().mid_price();

        for event in env.get_transactions().iter() {
            match event {
                Event::New { order_id } => {
                    let order = env.order(*order_id);
                    match order.side {
                        Side::Bid => assert!(f64::from(order.price) <= mid_price),
                        Side::Ask => assert!(f64::from(order.price) >= mid_price),
                    }
                }
                _ => panic!("Only new orders should have been placed"),
            }
        }

        env.step(&mut rng);

        agents.params.p_limit = 0.0;

        agents.update(&mut env, &mut rng);

        assert!(agents.orders.len() == 0);

        env.step(&mut rng);

        for i in (0..10).into_iter() {
            assert!(env.order(i).status == Status::Cancelled);
        }
    }
}
