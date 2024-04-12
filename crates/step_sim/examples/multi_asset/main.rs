use bourse_de::agents::{MarketAgent, MarketAgentSet, RandomMarketAgents};
use bourse_de::{market_sim_runner, MarketEnv};

#[derive(MarketAgentSet)]
struct Agents {
    pub a: RandomMarketAgents,
    pub b: RandomMarketAgents,
    pub c: RandomMarketAgents,
    pub d: RandomMarketAgents,
}

pub fn main() {
    let mut env = MarketEnv::<2>::new(0, [1, 1], 1_000_000, true);

    let mut agents = Agents {
        a: RandomMarketAgents::new(0, 50, (40, 60), (10, 20), 2, 0.8),
        b: RandomMarketAgents::new(0, 50, (10, 90), (50, 70), 2, 0.2),
        c: RandomMarketAgents::new(1, 50, (40, 60), (10, 20), 2, 0.8),
        d: RandomMarketAgents::new(1, 50, (10, 90), (50, 70), 2, 0.2),
    };

    market_sim_runner(&mut env, &mut agents, 101, 100, true);

    println!("{} trades of asset 0", env.get_trades(0).len());
    println!("{} trades of asset 1", env.get_trades(1).len());
}
