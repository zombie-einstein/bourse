use fastrand::Rng;

pub struct LogNormal {
    mu: f32,
    sigma: f32,
}

impl LogNormal {
    pub fn new(mu: f32, sigma: f32) -> Self {
        LogNormal { mu, sigma }
    }
    // TODO: Implement log-normal sample
    pub fn sample(&self, _rng: &mut Rng) -> f32 {
        self.mu + self.sigma
    }
}
