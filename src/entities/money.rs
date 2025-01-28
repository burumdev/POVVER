use crate::simulation::SimFlo;

pub struct Credit(SimFlo);

impl Credit {
    pub const fn new(credit: SimFlo) -> Self {
        Self(credit)
    }
}