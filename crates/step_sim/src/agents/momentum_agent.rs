use super::Agent;
use rand::RngCore;

pub struct MomentumAgent {}

impl Agent for MomentumAgent {
    fn update<R: RngCore>(&mut self, _env: &mut crate::Env, _rng: &mut R) {
        todo!()
    }
}
