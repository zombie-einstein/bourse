use divan::{black_box, Bencher};

use bourse_de::agents::{Agent, RandomAgents};
use bourse_de::{sim_runner, Env};
use bourse_macros::Agents;

#[derive(Agents)]
struct SimAgents {
    pub a: RandomAgents,
    pub b: RandomAgents,
}

#[divan::bench]
fn random_agents_simulation(bencher: Bencher) {
    let mut env = Env::new(0, 1, 1_000_000, true);

    let mut agents = SimAgents {
        a: RandomAgents::new(200, (40, 60), (10, 20), 2, 0.8),
        b: RandomAgents::new(200, (10, 90), (50, 70), 2, 0.2),
    };

    bencher.bench_local(move || {
        sim_runner(black_box(&mut env), black_box(&mut agents), 101, 200, false);
    });
}

fn main() {
    divan::main();
}
