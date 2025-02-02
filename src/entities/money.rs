use crate::simulation::SimFlo;

#[derive(Debug)]
pub struct Money(SimFlo);

impl Money {
    pub const fn new(amount: SimFlo) -> Self {
        Self(amount)
    }
}

impl Money {
    pub fn get(&self) -> SimFlo {
        self.0
    }

    pub fn set_amount(&mut self, amount: SimFlo) {
        self.0 = amount;
    }
}
