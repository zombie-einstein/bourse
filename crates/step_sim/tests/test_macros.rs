use bourse_book::types::{Price, Side};
use bourse_de::agents::{Agent, AgentSet, Agents};
use bourse_de::Env;
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
    #[derive(Agents)]
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
