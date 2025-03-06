use crate::economy::economy_types::{EnergyUnit, Money};
use crate::simulation::SimInt;

pub const PP_INIT_MONEY: Money = Money::new(10000.0);
pub const PP_INIT_FUEL: SimInt = 0;
pub const PP_INIT_PRODUCTION_CAP: EnergyUnit = EnergyUnit::new(100);
pub const PP_INIT_FUEL_CAP: SimInt = 50;
pub const PP_FUEL_CAPACITY_INCREASE_COST: Money = Money::new(10000.0);
pub const PP_FUEL_CAPACITY_INCREASE: SimInt = 100;
pub const PP_PRODUCTION_CAPACITY_INCREASE_COST: Money = Money::new(50000.0);
pub const PP_PRODUCTION_CAPACITY_INCREASE: EnergyUnit = EnergyUnit::new(200);
pub const PP_ENERGY_PER_FUEL: EnergyUnit = EnergyUnit::new(20);

pub const FACTORY_INIT_MONEY: Money = Money::new(10000.0);