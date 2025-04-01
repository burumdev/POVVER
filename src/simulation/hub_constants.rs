use crate::economy::economy_types::{EnergyUnit, Money};
use crate::simulation::SimInt;

pub const PP_INIT_MONEY: Money = Money::new(66000.0);
pub const PP_INIT_FUEL_CAPACITY: SimInt = 25;
pub const PP_INIT_PRODUCTION_CAP: EnergyUnit = EnergyUnit::new(24000);
pub const PP_INIT_FUEL_BUY_THRESHOLD: SimInt = 15;
pub const PP_FUEL_CAPACITY_INCREASE_COST: Money = Money::new(10000.0);
pub const PP_FUEL_CAPACITY_INCREASE: SimInt = 25;
pub const PP_PRODUCTION_CAPACITY_INCREASE_COST: Money = Money::new(25000.0);
pub const PP_PRODUCTION_CAPACITY_INCREASE: EnergyUnit = EnergyUnit::new(12000);
pub const PP_ENERGY_PER_FUEL: EnergyUnit = EnergyUnit::new(1000);

pub const FACTORY_INIT_MONEY: Money = Money::new(33000.0);
