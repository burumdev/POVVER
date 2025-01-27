use rand::Rng;
use crate::simulation::SimFlo;

#[derive(Debug, Default)]
pub struct Economy {
    growth_rate: SimFlo,
}

impl Economy {
    pub fn new(rng: &mut impl Rng) -> Self {
        Self {
            growth_rate: rng.gen_range(-1.2..1.2)
        }
    }
}