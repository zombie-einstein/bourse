use super::common;
use super::Agent;
use crate::types::{OrderId, Price, Side, TraderId, Vol};
use crate::Env;
use rand::{Rng, RngCore};
use rand_distr::LogNormal;

/// Momentum agent parameters
///
/// See [MomentumAgent] for details of the
/// these parameters are used.
pub struct MomentumParams {
    /// Integer market tick-size
    pub tick_size: Price,
    /// Probability of limit-order cancellation
    pub p_cancel: f32,
    /// Size of orders placed by agents
    pub trade_vol: Vol,
    /// Momentum decay factor
    pub decay: f64,
    /// Agent demand
    pub demand: f64,
    /// Momentum tanh scale
    pub scale: f64,
    /// Ratio of limit to market orders probabilities
    pub order_ratio: f64,
    /// Log-normal price distribution mean
    pub price_dist_mu: f64,
    /// Log-normal price distribution width
    pub price_dist_sigma: f64,
}

/// Agents that place trades conditioned on price history
///
/// A group of agents that track trends in price movements.
///
/// The momentum of the price, `M`, is updated each step
///
/// ```notrust
/// M = m * (1 - decay) + decay * (P - p)
/// ```
/// where `M` and `P` are the current momentum and price,
/// and `m` and `p` are the values at the previous step.
///
/// The probability an agent places a market order is then
/// given by
///
/// ```notrust
/// p_market = demand * tanh(scale * M) / n
/// ```
/// where `n` is the number of agents. The probability of
/// placing a limit order is then given by
///
/// ```notrust
/// p_limit = p_market * order_ratio
/// ```
///
/// Agents will then place a buy/sell order if `M` is
/// greater/less than 0.0 respectively.
///
/// Each step the agent(s)
///
/// - Randomly select existing limit orders for cancellation
/// - Calculate updated values for `M` and `p_market`.
/// - Place limit orders with probability `p_limit`
///   conditional on `M`
/// - Place market orders with probability `p_market`
///   conditional on `M`
///
/// # Examples
///
/// ```
/// use bourse_de::agents::{Agent, AgentSet, MomentumAgent, MomentumParams};
/// use bourse_de::{sim_runner, Env};
/// use bourse_macros::Agents;
///
/// #[derive(Agents)]
/// struct SimAgents {
///     pub a: MomentumAgent,
/// }
///
/// let mut env = Env::new(0, 1, 1_000_000, true);
///
/// let params = MomentumParams {
///     tick_size: 2,
///     p_cancel: 0.1,
///     trade_vol: 100,
///     decay: 1.0,
///     demand: 5.0,
///     scale: 0.5,
///     order_ratio: 1.0,
///     price_dist_mu: 0.0,
///     price_dist_sigma: 10.0,
/// };
/// let mut agents = SimAgents {
///     a: MomentumAgent::new(0, 5, params),
/// };
///
/// sim_runner(&mut env, &mut agents, 101, 10, false);
/// ```
/// # References
///
/// 1. <https://arxiv.org/abs/2208.13654>
///
pub struct MomentumAgent {
    price_dist: LogNormal<f64>,
    orders: Vec<OrderId>,
    trader_ids: Vec<TraderId>,
    last_price: Option<f64>,
    momentum: f64,
    n: f64,
    tick_size: f64,
    params: MomentumParams,
}

impl MomentumAgent {
    /// Initialise a set of momentum traders
    ///
    /// # Arguments
    ///
    /// - `agent_id_start` = Starting id number of these agents
    /// - `n_agents` - Number of agents in this set
    /// - `params` - Algorithm parameters, see [MomentumParams]
    ///
    pub fn new(agent_id_start: TraderId, n_agents: u16, params: MomentumParams) -> Self {
        let trader_ids = (agent_id_start..agent_id_start + TraderId::from(n_agents)).collect();

        Self {
            price_dist: LogNormal::<f64>::new(params.price_dist_mu, params.price_dist_sigma)
                .unwrap(),
            orders: Vec::new(),
            trader_ids,
            last_price: None,
            momentum: 0.0,
            n: n_agents.into(),
            tick_size: params.tick_size.into(),
            params,
        }
    }
}

impl Agent for MomentumAgent {
    fn update<R: RngCore>(&mut self, env: &mut Env, rng: &mut R) {
        let mut live_orders =
            common::cancel_live_orders(env, rng, &self.orders, self.params.p_cancel);

        let mid_price = env.get_orderbook().mid_price();

        let (m, p_market) = match self.last_price {
            Some(p) => {
                let m =
                    self.momentum * (1.0 - self.params.decay) + self.params.decay * (mid_price - p);
                let p = self.params.demand * f64::tanh(self.params.scale * m) / self.n;
                (m, p)
            }
            None => (0.0, 0.0),
        };

        let p_limit = self.params.order_ratio * p_market;

        for trader_id in self.trader_ids.iter() {
            if rng.gen::<f64>() < p_limit {
                if m > 0.0 {
                    let order_id = common::place_buy_limit_order(
                        env,
                        rng,
                        self.price_dist,
                        mid_price,
                        self.tick_size,
                        self.params.trade_vol,
                        *trader_id,
                    )
                    .unwrap();
                    live_orders.push(order_id);
                } else if m < 0.0 {
                    let order_id = common::place_sell_limit_order(
                        env,
                        rng,
                        self.price_dist,
                        mid_price,
                        self.tick_size,
                        self.params.trade_vol,
                        *trader_id,
                    )
                    .unwrap();
                    live_orders.push(order_id);
                }
            }

            if rng.gen::<f64>() < p_market {
                if m > 0.0 {
                    env.place_order(Side::Bid, self.params.trade_vol, *trader_id, None)
                        .unwrap();
                } else if m < 0.0 {
                    env.place_order(Side::Ask, self.params.trade_vol, *trader_id, None)
                        .unwrap();
                }
            }
        }

        self.momentum = m;
        self.last_price = Some(mid_price);

        self.orders = live_orders;
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_xoshiro::Xoroshiro128StarStar;

    use super::*;

    #[test]
    fn test_init_and_no_order() {
        let mut env = Env::new(0, 1, 1_000_000, true);
        let mut rng = Xoroshiro128StarStar::seed_from_u64(101);

        env.place_order(Side::Bid, 100, 0, Some(1000)).unwrap();
        env.place_order(Side::Ask, 100, 0, Some(1020)).unwrap();
        env.step(&mut rng);

        let params = MomentumParams {
            tick_size: 2,
            p_cancel: 0.1,
            trade_vol: 100,
            decay: 1.0,
            demand: 5.0,
            scale: 0.5,
            order_ratio: 1.0,
            price_dist_mu: 0.0,
            price_dist_sigma: 10.0,
        };
        let mut agents = MomentumAgent::new(10, 100, params);

        agents.update(&mut env, &mut rng);

        assert!(env.get_transactions().is_empty());
    }
}
