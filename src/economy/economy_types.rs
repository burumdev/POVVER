use crate::simulation::{SimFlo, SimInt};
use crate::utils_traits::Flippable;

#[derive(Debug)]
pub struct Money(SimFlo);

pub const INFLATION_MAX: SimFlo = 10000.0;
pub const INFLATION_MIN: SimFlo = -10.0;

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

#[derive(Debug, PartialEq)]
pub enum UpDown {
    Up,
    Down,
}
impl Flippable for UpDown {
    fn flip(&mut self) -> Self {
        match self {
            Self::Up => {
                *self = Self::Down;

                Self::Down
            },
            Self::Down => {
                *self = UpDown::Up;

                Self::Up
            },
        }
    }
}

pub struct EnergyUnit(SimInt);

impl EnergyUnit {
    pub const fn new(unit: SimInt) -> Self {
        Self(unit)
    }
}
