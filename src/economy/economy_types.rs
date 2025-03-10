use slint::ToSharedString;
use crate::{
    economy::products::Product,
    simulation::{SimFlo, SimInt},
    ui_controller::{UpDown as UIUpDown, ProductDemand as UIProductDemand },
    utils_traits::Flippable,
};

pub const INFLATION_MAX: SimFlo = 50.0;
pub const INFLATION_MIN: SimFlo = -10.0;
pub const FUEL_PRICE_MIN: SimFlo = 100.0;
pub const FUEL_PRICE_MAX: SimFlo = 1000.0;
pub const FUEL_PRICE_MODIFIER: SimFlo = 20.00;

#[derive(Debug, Copy, Clone)]
pub struct Money(SimFlo);

impl Money {
    pub const fn new(amount: SimFlo) -> Self {
        Self(amount)
    }
}

impl Money {
    pub fn val(&self) -> SimFlo {
        self.0
    }

    pub fn set(&mut self, amount: SimFlo) {
        self.0 = amount.clamp(0.0, SimFlo::MAX);
    }

    pub fn dec(&mut self, amount: SimFlo) -> bool {
        if self.0 - amount < 0.0 {
            return false;
        }

        self.0 -= amount;

        true
    }

    pub fn inc(&mut self, amount: SimFlo) {
        self.0 += amount.clamp(0.0, SimFlo::MAX);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum UpDown {
    Up,
    Down,
}
impl From<UpDown> for UIUpDown {
    fn from(other: UpDown) -> Self {
        match other {
            UpDown::Up => Self::Up,
            UpDown::Down => Self::Down,
        }
    }
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

#[derive(Debug)]
pub struct EnergyUnit(SimInt);

impl EnergyUnit {
    pub const fn new(unit: SimInt) -> Self {
        Self(unit)
    }
}

#[derive(Debug, Clone)]
pub struct ProductDemand {
    pub product: &'static Product,
    pub percent: SimFlo,
    pub age: SimInt,
    pub demand_meet_percent: SimFlo,
}
impl ProductDemand {
    pub fn new(product: &'static Product, percent: SimFlo) -> Self {
        Self {
            product,
            percent,
            age: 0,
            demand_meet_percent: 0.0
        }
    }
}
impl From<&ProductDemand> for UIProductDemand {
    fn from(other: &ProductDemand) -> Self {
        Self {
            product_name: other.product.name.to_shared_string(),
            age: other.age,
            demand_met: other.demand_meet_percent,
            percent: other.percent,
        }
    }
}