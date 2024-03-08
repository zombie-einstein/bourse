use bourse_de::agents::{Agent, RandomAgents};
use bourse_de::{sim_runner, Env};
use bourse_macros::Agents;

#[derive(Agents)]
struct SimAgents {
    pub a: RandomAgents,
    pub b: RandomAgents,
}

pub fn main() {
    let mut env = Env::new(0, 1, 1_000_000, true);

    let mut agents = SimAgents {
        a: RandomAgents::new(50, (40, 60), (10, 20), 2, 0.8),
        b: RandomAgents::new(50, (10, 90), (50, 70), 2, 0.2),
    };

    sim_runner(&mut env, &mut agents, 101, 100, true);

    println!("{} trades", env.get_trades().len());
}
