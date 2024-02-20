use super::Agent;

pub struct MomentumAgent {}

impl Agent for MomentumAgent {
    fn update(&mut self, _env: &mut crate::Env, _rng: &mut fastrand::Rng) {
        todo!()
    }
}
