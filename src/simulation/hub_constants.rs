use crate::economy::economy_types::{EnergyUnit, Money};
use crate::simulation::SimInt;

pub const PP_FUEL_CAPACITY_INCREASE_COST: Money = Money::new(10000.0);
pub const PP_FUEL_CAPACITY_INCREASE: SimInt = 100;
pub const PP_PRODUCTION_CAPACITY_INCREASE_COST: Money = Money::new(50000.0);
pub const PP_PRODUCTION_CAPACITY_INCREASE: EnergyUnit = EnergyUnit::new(200);
