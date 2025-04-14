use slint::ToSharedString;

use crate::{
    economy::products::Product,
    simulation::{SimFlo, SimInt, Percentage},
    ui_controller::{UpDown as UIUpDown, ProductDemand as UIProductDemand },
    utils_traits::{Flippable, AsFactor}
};



#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Money(SimFlo);

impl Money {
    pub const fn new(amount: SimFlo) -> Self {
        Self(amount)
    }
}
impl Default for Money {
    fn default() -> Self {
        Self(0.0)
    }
}
impl PartialOrd for Money {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}
impl From<SimFlo> for Money {
    fn from(val: SimFlo) -> Self {
        Self::new(val)
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
        self.0 = (self.0 + amount).clamp(0.0, SimFlo::MAX);
    }
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct EnergyUnit(SimInt);
impl EnergyUnit {
    pub const fn new(unit: SimInt) -> Self {
        Self(unit)
    }
}
impl Default for EnergyUnit {
    fn default() -> Self {
        Self(0)
    }
}
impl EnergyUnit {
    pub fn val(&self) -> SimInt {
        self.0
    }
    pub fn set(&mut self, amount: SimInt) {
        self.0 = amount.clamp(0, SimInt::MAX);
    }
    pub fn dec(&mut self, unit: SimInt) {
        self.0 = (self.0 - unit).clamp(0, SimInt::MAX);
    }
    pub fn inc(&mut self, unit: SimInt) {
        self.0 = (self.0 + unit).clamp(0, SimInt::MAX);
    }
    pub fn zero(&mut self) {
        self.0 = 0;
    }
}
impl PartialOrd for EnergyUnit {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}
impl From<EnergyUnit> for SimInt {
    fn from(other: EnergyUnit) -> Self {
        other.0
    }
}
impl From<EnergyUnit> for SimFlo {
    fn from(other: EnergyUnit) -> Self {
        other.0 as SimFlo
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ProductDemand {
    pub product: &'static Product,
    pub percent: Percentage,
    pub age: SimInt,
    pub demand_meet_percent: Percentage,
    pub units: SimInt,
}
impl ProductDemand {
    pub fn new(product: &'static Product, percent: Percentage) -> Self {
        let units = (percent.val() * product.demand_info.unit_per_percent as SimFlo) as SimInt;
        Self {
            product,
            percent,
            age: 0,
            demand_meet_percent: Percentage::default(),
            units,
        }
    }
    pub fn as_units(&self) -> SimInt {
        (self.percent.val() * self.product.demand_info.unit_per_percent as SimFlo) as SimInt
    }
    pub fn calculate_energy_need(&self) -> SimInt {
        self.as_units() * self.product.unit_production_cost.energy
    }
}
impl From<&ProductDemand> for UIProductDemand {
    fn from(other: &ProductDemand) -> Self {
        Self {
            product_name: other.product.name.to_shared_string(),
            age: other.age,
            demand_met: other.demand_meet_percent.val(),
            percent: other.percent.val(),
        }
    }
}