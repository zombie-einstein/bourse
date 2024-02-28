use super::Agent;
use rand_xoshiro::Xoroshiro128StarStar as Rng;

pub struct MomentumAgent {}

impl Agent for MomentumAgent {
    fn update(&mut self, _env: &mut crate::Env, _rng: &mut Rng) {
        todo!()
    }
}
