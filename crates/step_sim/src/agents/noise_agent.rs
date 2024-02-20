use super::Agent;

pub struct NoiseAgent {}

impl Agent for NoiseAgent {
    fn update(&mut self, _env: &mut crate::Env, _rng: &mut fastrand::Rng) {
        todo!()
    }
}
