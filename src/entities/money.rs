use crate::simulation::SimFlo;

#[derive(Debug)]
pub struct Money(SimFlo);

impl Money {
    pub const fn new(amount: SimFlo) -> Self {
        Self(amount)
    }
}