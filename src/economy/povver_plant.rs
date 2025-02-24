use std::{
    thread,
    sync::{Arc, RwLock},
};
use crate::{
    app_state::PovverPlantStateData,
    economy::economy_types::{Money, EnergyUnit},
    simulation::SimInt,
    utils_data::SlidingWindow,
};

pub const FUEL_CAPACITY_INCREASE_COST: Money = Money::new(10000.0);
pub const FUEL_CAPACITY_INCREASE: SimInt = 100;
pub const PRODUCTION_CAPACITY_INCREASE_COST: Money = Money::new(50000.0);
pub const PRODUCTION_CAPACITY_INCREASE: EnergyUnit = EnergyUnit::new(200);

pub struct PovverPlant {
    last_ten_sales: SlidingWindow<Money>,
}

impl PovverPlant {
    pub fn new() -> Self {
        Self {
            last_ten_sales: SlidingWindow::new(10),
        }
    }
}

impl PovverPlant {
    pub fn start(&self, state: Arc<RwLock<PovverPlantStateData>>) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            loop {
            }
        })
    }
}