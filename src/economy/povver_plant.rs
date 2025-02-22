use super::{
    economy_types::Money,
    EnergyUnit,
};

use crate::{
    simulation::SimInt,
    utils_data::SlidingWindow,
};

pub const FUEL_CAPACITY_INCREASE_COST: Money = Money::new(10000.0);
pub const FUEL_CAPACITY_INCREASE: SimInt = 100;
pub const PRODUCTION_CAPACITY_INCREASE_COST: Money = Money::new(50000.0);
pub const PRODUCTION_CAPACITY_INCREASE: EnergyUnit = EnergyUnit::new(200);

pub struct PovverPlant {
    fuel: SimInt,
    fuel_capacity: SimInt,
    production_capacity: EnergyUnit,
    balance: Money,
    last_ten_sales: SlidingWindow<Money>,
}

impl PovverPlant {
    pub fn new() -> Self {
        Self {
            fuel: 0,
            fuel_capacity: 200,
            production_capacity: EnergyUnit::new(400),
            balance: Money::new(10000.0),
            last_ten_sales: SlidingWindow::new(10),
        }
    }
}

impl PovverPlant {
    pub fn start(&mut self) {

    }
}