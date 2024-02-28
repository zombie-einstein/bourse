//! Agent that randomly places and cancels limit and market orders
use super::Agent;
use crate::types::{OrderId, Price, Side, Status, TraderId, Vol};
use crate::Env;
use rand::seq::SliceRandom;
use rand::Rng;
use rand_distr::{Distribution, LogNormal};
use rand_xoshiro::Xoroshiro128StarStar as RngGen;

/// Agent that randomly places and cancels limit and market orders
///
/// Represents a group of agents that randomly place and cancel
/// orders at each step of the simulation. Each step:
///
/// - Any currently live orders are randomly selected for cancellation
/// - Each agent randomly chooses to place a limit order, if so they
///   place an order on a random side with price above/below the
///   mid-price sampled from a log-normal distribution
/// - Each agent randomly chooses to place a market order, if so they
///   place an order on a random side
///
/// # Examples
///
/// ```
/// use bourse_de::agents::{Agent, AgentSet, NoiseAgent};
/// use bourse_de::{sim_runner, Env};
/// use bourse_macros::Agents;
///
/// #[derive(Agents)]
/// struct SimAgents {
///     pub a: NoiseAgent,
/// }
///
/// let mut env = Env::new(0, 1_000_000, true);
///
/// let mut agents = SimAgents {
///     a: NoiseAgent::new(0, 5, 2.0, 0.5, 0.25, 100, 2, 0.0, 1.0),
/// };
///
/// sim_runner(&mut env, &mut agents, 101, 10);
/// ```
///
/// # References
///
/// <https://arxiv.org/abs/2208.13654>
///
pub struct NoiseAgent {
    tick_size: f64,
    p_limit: f32,
    p_market: f32,
    p_cancel: f32,
    trade_vol: Vol,
    price_dist: LogNormal<f64>,
    orders: Vec<OrderId>,
    trader_ids: Vec<TraderId>,
}

impl NoiseAgent {
    /// Initialise a group of noise-agents
    ///
    /// # Parameters
    ///
    /// - `agent_id_start` - Starting id for
    ///   agents in this set
    /// - `n_agents` - Number of agents
    /// - `demand` - Represents aggregate demand of
    ///   these agents. The probability of agents
    ///   placing limit orders is given by
    ///   `demand/n_agents`
    /// - `order_ratio` - Ratio of probability of
    ///   of placing limit orders to market orders
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
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        agent_id_start: TraderId,
        n_agents: u16,
        demand: f32,
        order_ratio: f32,
        p_cancel: f32,
        trade_vol: Vol,
        tick_size: Price,
        price_dist_mu: f32,
        price_dist_sigma: f32,
    ) -> Self {
        let p_limit = demand / f32::from(n_agents);
        let p_market = order_ratio * p_limit;
        let trader_ids = (agent_id_start..agent_id_start + TraderId::from(n_agents)).collect();

        Self {
            tick_size: tick_size.into(),
            p_limit,
            p_market,
            p_cancel,
            trade_vol,
            price_dist: LogNormal::<f64>::new(price_dist_mu.into(), price_dist_sigma.into())
                .unwrap(),
            orders: Vec::new(),
            trader_ids,
        }
    }
}

fn round_price(p: f64, tick_size: f64) -> Price {
    let p = (p / tick_size).round() * tick_size;
    let p = p.clamp(0.0, Price::MAX.into());
    p as Price
}

impl Agent for NoiseAgent {
    fn update(&mut self, env: &mut Env, rng: &mut RngGen) {
        let live_orders = self
            .orders
            .iter()
            .filter(|x| env.order_status(**x) == Status::Active);

        let (mut live_orders, to_cancel): (Vec<OrderId>, Vec<OrderId>) = live_orders
            .into_iter()
            .partition(|_| rng.gen::<f32>() < self.p_cancel);

        for order_id in to_cancel.into_iter() {
            env.cancel_order(order_id)
        }

        let mid_price = env.get_orderbook().mid_price();

        for trader_id in self.trader_ids.iter() {
            if rng.gen::<f32>() < self.p_limit {
                let side = [Side::Bid, Side::Ask].choose(rng).unwrap();
                let dist = self.price_dist.sample(rng).abs();

                let order_id = match side {
                    Side::Bid => {
                        let price = mid_price - dist;
                        let price = round_price(price, self.tick_size);
                        env.place_order(Side::Bid, self.trade_vol, *trader_id, Some(price))
                    }
                    Side::Ask => {
                        let price = mid_price + dist;
                        let price = round_price(price, self.tick_size);
                        env.place_order(Side::Ask, self.trade_vol, *trader_id, Some(price))
                    }
                };
                live_orders.push(order_id);
            }

            if rng.gen::<f32>() < self.p_market {
                let side = [Side::Bid, Side::Ask].choose(rng).unwrap();
                match side {
                    Side::Bid => env.place_order(Side::Bid, self.trade_vol, *trader_id, None),
                    Side::Ask => env.place_order(Side::Ask, self.trade_vol, *trader_id, None),
                };
            }
        }

        self.orders = live_orders;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let agents = NoiseAgent::new(10, 4, 2.0, 0.5, 0.1, 100, 2, 0.0, 1.0);

        assert!(agents.trader_ids == vec![10, 11, 12, 13])
    }
}
