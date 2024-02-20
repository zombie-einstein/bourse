use bourse_book::types::{OrderId, Price, Side, TraderId, Vol};
use bourse_de::agents::AgentSet;
use bourse_de::{sim_runner, Env};

struct RandomAgentSet {
    orders: Vec<Option<OrderId>>,
    price_range: (Price, Price),
    trade_vol: Vol,
}

impl RandomAgentSet {
    pub fn new(n_agents: usize, max_price: Price, min_price: Price, trade_vol: Vol) -> Self {
        Self {
            orders: vec![None; n_agents],
            price_range: (min_price, max_price),
            trade_vol,
        }
    }
}

impl AgentSet for RandomAgentSet {
    fn update(&mut self, env: &mut Env, rng: &mut fastrand::Rng) {
        let new_orders: Vec<Option<OrderId>> = self
            .orders
            .iter_mut()
            .enumerate()
            .map(|(n, i)| {
                let p = rng.bool();
                match p {
                    true => match i {
                        Some(id) => {
                            env.cancel_order(*id);
                            None
                        }
                        None => {
                            let side = rng.choice([Side::Ask, Side::Bid]).unwrap();
                            let price = rng.u32(self.price_range.0..self.price_range.1);
                            Some(env.place_order(
                                side,
                                self.trade_vol,
                                TraderId::try_from(n).unwrap(),
                                Some(price),
                            ))
                        }
                    },
                    false => *i,
                }
            })
            .collect();

        self.orders = new_orders;
    }
}

pub fn main() {
    let mut env = Env::new(0, 1_000_000, true);
    let mut agents = RandomAgentSet::new(100, 100, 10, 100);

    sim_runner(&mut env, &mut agents, 101, 100);

    println!("{} trades", env.get_trades().len());
}
