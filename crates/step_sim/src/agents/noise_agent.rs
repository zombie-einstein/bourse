use super::stats::LogNormal;
use super::Agent;
use crate::types::{OrderId, Price, Side, Status, TraderId, Vol};
use crate::Env;

pub struct NoiseAgent {
    _tick_size: Price,
    p_limit: f32,
    p_market: f32,
    p_cancel: f32,
    trade_vol: Vol,
    price_dist: LogNormal,
    orders: Vec<OrderId>,
    trader_ids: Vec<TraderId>,
}

impl NoiseAgent {
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
            _tick_size: tick_size,
            p_limit,
            p_market,
            p_cancel,
            trade_vol,
            price_dist: LogNormal::new(price_dist_mu, price_dist_sigma),
            orders: Vec::new(),
            trader_ids,
        }
    }
}

impl Agent for NoiseAgent {
    fn update(&mut self, env: &mut Env, rng: &mut fastrand::Rng) {
        let live_orders = self
            .orders
            .iter()
            .filter(|x| env.order_status(**x) == Status::Active);

        let (mut live_orders, to_cancel): (Vec<OrderId>, Vec<OrderId>) = live_orders
            .into_iter()
            .partition(|_| rng.f32() < self.p_cancel);

        for order_id in to_cancel.into_iter() {
            env.cancel_order(order_id)
        }

        let (bid, ask) = env.get_orderbook().bid_ask();

        for trader_id in self.trader_ids.iter() {
            if rng.f32() < self.p_limit {
                let side = rng.choice([Side::Bid, Side::Ask]).unwrap();
                // TODO: Convert log-normal sample into price from touch
                let _dist = self.price_dist.sample(rng);
                let price_off: Price = 10;
                let order_id = match side {
                    Side::Bid => env.place_order(
                        Side::Bid,
                        self.trade_vol,
                        *trader_id,
                        Some(bid - price_off),
                    ),
                    Side::Ask => env.place_order(
                        Side::Ask,
                        self.trade_vol,
                        *trader_id,
                        Some(ask + price_off),
                    ),
                };
                live_orders.push(order_id);
            }

            if rng.f32() < self.p_market {
                let side = rng.choice([Side::Bid, Side::Ask]).unwrap();
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
