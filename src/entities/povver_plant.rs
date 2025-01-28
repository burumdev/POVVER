use crate::simulation::SimInt;
use crate::entities::energy::EnergyUnit;

pub struct PovverPlant {
    fuel: SimInt,
    production_capacity: EnergyUnit,
}