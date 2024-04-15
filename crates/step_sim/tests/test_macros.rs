use bourse_book::types::{Price, Side};
use bourse_de::agents::{Agent, AgentSet, MarketAgent, MarketAgentSet};
use bourse_de::types::AssetIdx;
use bourse_de::{Env, MarketEnv};
use rand::RngCore;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoroshiro128StarStar;

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
    fn update<R: RngCore>(&mut self, env: &mut Env, _rng: &mut R) {
        env.place_order(self.side, 10, 101, Some(self.price))
            .unwrap();
    }
}

#[test]
fn test_agent_macro() {
    #[derive(AgentSet)]
    struct TestAgents {
        pub a: TestAgent,
        pub b: TestAgent,
    }

    let mut env = Env::new(0, 1, 1000, true);
    let mut rng = Xoroshiro128StarStar::seed_from_u64(101);

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

struct MarketTestAgent {
    asset: AssetIdx,
    side: Side,
    price: Price,
}

impl MarketTestAgent {
    pub fn new(asset: AssetIdx, side: Side, price: Price) -> Self {
        Self { asset, side, price }
    }
}

impl MarketAgent for MarketTestAgent {
    fn update<R: RngCore, const M: usize, const N: usize>(
        &mut self,
        env: &mut MarketEnv<M, N>,
        _rng: &mut R,
    ) {
        env.place_order(self.asset, self.side, 10, 101, Some(self.price))
            .unwrap();
    }
}

#[test]
fn test_market_agent_macro() {
    #[derive(MarketAgentSet)]
    struct TestAgents {
        pub a: MarketTestAgent,
        pub b: MarketTestAgent,
        pub c: MarketTestAgent,
        pub d: MarketTestAgent,
    }

    let mut env = MarketEnv::<2>::new(0, [1, 1], 1000, true);
    let mut rng = Xoroshiro128StarStar::seed_from_u64(101);

    let mut test_agents = TestAgents {
        a: MarketTestAgent::new(0, Side::Bid, 20),
        b: MarketTestAgent::new(0, Side::Ask, 40),
        c: MarketTestAgent::new(1, Side::Bid, 60),
        d: MarketTestAgent::new(1, Side::Ask, 80),
    };

    test_agents.update(&mut env, &mut rng);
    env.step(&mut rng);

    assert!(env.get_market().ask_vols() == [10, 10]);
    assert!(env.get_market().bid_vols() == [10, 10]);
    assert!(env.get_market().bid_asks() == [(20, 40), (60, 80)]);

    test_agents.update(&mut env, &mut rng);
    env.step(&mut rng);

    assert!(env.get_market().ask_vols() == [20, 20]);
    assert!(env.get_market().bid_vols() == [20, 20]);
    assert!(env.get_market().bid_asks() == [(20, 40), (60, 80)]);
}
